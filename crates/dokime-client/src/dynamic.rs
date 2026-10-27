/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

pub use harmos::grpc::dynamic::Event;

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use futures_util::{StreamExt, stream};
use harmos::grpc::dynamic::Operation;

// Local Imports
use super::operations::{Error, Result};
use super::{Client, ResponseStream, stream::Input};

/* === Constants === */

const INPUT_BYTES: u64 = 16_777_216;
const INPUT_FRAMES: u64 = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub async fn call_operation(
        &self, identifier: &str, owner: Option<&str>, request: &[u8], inputs: Vec<Vec<u8>>,
    ) -> Result<ResponseStream<Event>> {
        let bytes = inputs.iter().try_fold(request.len(), |total, frame| total.checked_add(frame.len()).ok_or(Error::InputTooLarge))?;
        if bytes > INPUT_BYTES as usize || inputs.len() > INPUT_FRAMES as usize {
            return Err(Error::InputTooLarge);
        }
        self.verify(true).await?;
        let providers: Vec<_> = self
            .operations()
            .await?
            .into_iter()
            .filter(|entry| entry.contract.operation().id.as_str() == identifier && owner.is_none_or(|owner| entry.owner.as_str() == owner))
            .collect();
        let mut providers = providers.into_iter();
        let provider = providers.next().ok_or_else(|| Error::Unknown(identifier.into()))?;
        if providers.next().is_some() {
            return Err(Error::Ambiguous(identifier.into()));
        }
        if !provider.grpc_available {
            return Err(super::Error::Rpc(tonic::Status::unimplemented("the selected provider does not expose gRPC")).into());
        }
        let limit = NonZeroUsize::new(self.request_limit().min(self.response_limit())).ok_or(super::Error::IncompatibleApi)?;
        let operation = Operation::new(&provider.contract, limit).map_err(super::Error::Rpc)?;
        if !operation.client_streaming() && !inputs.is_empty() {
            return Err(Error::InputStream(identifier.into()));
        }
        let opening = operation.request(request).map_err(super::Error::Rpc)?;
        let mut payloads = Vec::with_capacity(inputs.len() + 1);
        payloads.push(opening);
        for frame in inputs {
            let frame = operation.input(&frame).map_err(super::Error::Rpc)?;
            payloads.push(frame);
        }
        let path = tonic::codegen::http::uri::PathAndQuery::try_from(operation.path().to_owned()).map_err(|_| Error::Catalog)?;
        let mut grpc =
            tonic::client::Grpc::new(self.channel()?).max_decoding_message_size(limit.get()).max_encoding_message_size(limit.get());
        let codec = operation.codec();
        let cancellation = self.inner.cancellation.child_token();
        let cancel = cancellation.clone().drop_guard();
        if operation.server_streaming() {
            let incoming = if operation.client_streaming() {
                let frames = Input::new(stream::iter(payloads), cancellation);
                self.execute(async {
                    grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                    grpc.streaming(self.request(frames), path, codec).await.map(tonic::Response::into_inner)
                })
                .await?
            } else {
                let opening = payloads.pop().expect("the validated opening exists");
                self.execute(async {
                    grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                    grpc.server_streaming(self.request(opening), path, codec).await.map(tonic::Response::into_inner)
                })
                .await?
            };
            // The Completion Is Published Only After Its Successful Final Status.
            let frames = stream::unfold(Some((incoming, cancel)), |state| async move {
                let (mut incoming, cancel) = state?;
                match incoming.next().await {
                    Some(Ok(Event::Item(value))) => Some((Ok(Event::Item(value)), Some((incoming, cancel)))),
                    Some(Ok(completion @ Event::Completion(_))) => match incoming.next().await {
                        None => Some((Ok(completion), None)),
                        Some(Err(error)) => Some((Err(error), None)),
                        Some(Ok(_)) => Some((Err(tonic::Status::data_loss("operation continued after completion")), None)),
                    },
                    Some(Err(error)) => Some((Err(error), None)),
                    None => Some((Err(tonic::Status::data_loss("operation has no completion")), None)),
                }
            });
            Ok(ResponseStream::new(self, frames)?)
        } else {
            let completion = if operation.client_streaming() {
                let frames = Input::new(stream::iter(payloads), cancellation);
                self.execute(async {
                    grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                    grpc.client_streaming(self.request(frames), path, codec).await.map(tonic::Response::into_inner)
                })
                .await?
            } else {
                let opening = payloads.pop().expect("the validated opening exists");
                self.execute(async {
                    grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                    grpc.unary(self.request(opening), path, codec).await.map(tonic::Response::into_inner)
                })
                .await?
            };
            if !matches!(&completion, Event::Completion(_)) {
                return Err(Error::Catalog);
            }
            Ok(ResponseStream::new(self, stream::once(async move { Ok(completion) }))?)
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
