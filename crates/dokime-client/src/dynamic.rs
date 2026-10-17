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
use super::operations::{Error, MAX_INPUT_BYTES, Result};
use super::{Client, ResponseStream, stream::Input};

/* === Constants === */
const MAX_INPUT_FRAMES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    /// Calls a provider discovered from the connected backend's live catalog.
    ///
    /// `request` and each input frame use protobuf JSON for the discovered schema.
    /// The owner selects a provider when an identifier is shared. Inputs are bounded
    /// to 4096 frames and 16 MiB including the request; use typed file uploads for
    /// large content. Items preserve order, and completion is emitted only after
    /// a successful final gRPC status. Closing the result cancels transport work.
    ///
    /// ```no_run
    /// use dokime_client::{Client, Target, dynamic::Event};
    ///
    /// # async fn call() -> Result<(), Box<dyn std::error::Error>> {
    /// let target = Target::new("https://lab.example:50051", None)?;
    /// let client = Client::connect(target, Some("token"), None).await?;
    ///
    /// let stream = client
    ///     .call_operation("obc-plugin.get_status", Some("obc-plugin"), b"{}", Vec::new())
    ///     .await?;
    /// while let Some(event) = stream.message().await? {
    ///     match event {
    ///         Event::Item(value) => println!("item {value}"),
    ///         Event::Completion(value) => println!("completion {value}"),
    ///     }
    /// }
    /// stream.close().await;
    ///
    /// client.close_and_wait().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call_operation(
        &self, identifier: &str, owner: Option<&str>, request: &[u8], inputs: Vec<Vec<u8>>,
    ) -> Result<ResponseStream<Event>> {
        let bytes = inputs.iter().try_fold(request.len(), |total, frame| total.checked_add(frame.len()).ok_or(Error::InputTooLarge))?;
        if bytes > MAX_INPUT_BYTES || inputs.len() > MAX_INPUT_FRAMES {
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
