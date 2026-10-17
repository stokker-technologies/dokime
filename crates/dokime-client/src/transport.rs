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

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use futures_util::{StreamExt, stream};
use harmos::MemorySize;
use harmos::domain::Portable;
use harmos::grpc::{Codec, DecodeSettings, EncodeSettings, InputEnvelope, OutputEnvelope, Transcoded, Wire};
use harmos::operations::{Direction, OperationDeclaration, Response, StreamItem};
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};

// Local Imports
use super::{Client, Error, ResponseStream, Result, request, stream::Input};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub(super) async fn download<O>(&self, path: &'static str, message: O) -> Result<ResponseStream<<O::Output as Direction>::Item>>
    where
        O: OperationDeclaration,
        O: Portable + Sync,
        O::Response: Portable + MemorySize,
        <O::Output as Direction>::Item: Portable + MemorySize,
    {
        self.require(path)?;
        let codec = codec::<OutputEnvelope<<O::Output as Direction>::Item, O::Response>, O>(self.response_limit(), self.request_limit())?;
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(self.response_limit())
            .max_encoding_message_size(self.request_limit());
        let incoming = self
            .execute(async {
                client.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                client
                    .server_streaming(
                        self.request(Response::new(message)),
                        tonic::codegen::http::uri::PathAndQuery::from_static(path),
                        codec,
                    )
                    .await
                    .map(tonic::Response::into_inner)
            })
            .await?;
        let frames = stream::unfold(Some(incoming), |state| async move {
            let mut incoming = state?;
            match incoming.next().await {
                Some(Err(status)) => Some((Err(status), None)),
                None => Some((Err(tonic::Status::data_loss("operation stream has no completion")), None)),
                Some(Ok(frame)) => match OutputEnvelope::project(frame) {
                    Err(error) => Some((Err(tonic::Status::data_loss(error.to_string())), None)),
                    Ok(OutputEnvelope::Item(item)) => Some((value(item).map_err(status), Some(incoming))),
                    Ok(OutputEnvelope::Response(_)) => {
                        // Final Status Must Be Observed Before Reporting End Of Stream.
                        match incoming.next().await {
                            Some(Err(status)) => Some((Err(status), None)),
                            Some(Ok(_)) => Some((Err(tonic::Status::data_loss("operation stream continued after completion")), None)),
                            None => None,
                        }
                    }
                },
            }
        });
        ResponseStream::new(self, frames)
    }

    pub(super) async fn upload<O, S>(&self, path: &'static str, message: O, items: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        O: Portable + Sync,
        O::Response: Portable + MemorySize,
        <O::Input as Direction>::Item: Portable + Sync,
        S: tokio_stream::Stream<Item = <O::Input as Direction>::Item> + Send + 'static,
    {
        self.require(path)?;
        let codec = codec::<O::Response, InputEnvelope<O, <O::Input as Direction>::Item>>(self.response_limit(), self.request_limit())?;
        let cancellation = self.inner.cancellation.child_token();
        let _cancel = cancellation.clone().drop_guard();
        let opening = stream::once(async move { Response::new(InputEnvelope::Request(message)) });
        let frames = opening.chain(items.map(|item| Response::new(InputEnvelope::Item(item))));
        let frames = Input::new(frames, cancellation);
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(self.response_limit())
            .max_encoding_message_size(self.request_limit());
        let reply = self
            .execute(async {
                client.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                client
                    .client_streaming(self.request(frames), tonic::codegen::http::uri::PathAndQuery::from_static(path), codec)
                    .await
                    .map(tonic::Response::into_inner)
            })
            .await?;
        value(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn value<T>(reply: StreamItem<T>) -> Result<T> {
    // Transfer The Decoded Allocation; Never Clone A Payload To Drop Its Owner.
    reply.into_request().map(|(value, _claims)| value).map_err(|error| Error::Rpc(tonic::Status::data_loss(error.to_string())))
}

pub(super) fn codec<D, E>(response_bytes: usize, request_bytes: usize) -> Result<Codec<Transcoded, D, Response<E>>>
where
    Transcoded: Wire<D> + Wire<E>,
    D: MemorySize + Send + 'static,
    E: Send + Sync + 'static,
{
    let response_bytes = NonZeroUsize::new(response_bytes).ok_or(Error::IncompatibleApi)?;
    let request_bytes = NonZeroUsize::new(request_bytes).ok_or(Error::IncompatibleApi)?;
    // Transcoding Decodes Through The Canonical Postcard Form, So A Message Retains Up To Four Times Its Wire Size.
    let retained_bytes = NonZeroUsize::new(response_bytes.get().saturating_mul(4).min(u32::MAX as usize)).unwrap();
    Codec::new(
        DecodeSettings { message_bytes: response_bytes, retained_bytes },
        EncodeSettings { message_bytes: request_bytes, buffer_bytes: NonZeroUsize::new(request_bytes.get().min(8192)).unwrap() },
    )
    .map_err(Error::Rpc)
}

pub(super) async fn unary<R, Q>(
    channel: Channel, authorization: Option<&MetadataValue<Ascii>>, path: &'static str, message: Q, response_bytes: usize,
    request_bytes: usize,
) -> Result<StreamItem<R>>
where
    R: Portable + MemorySize + 'static,
    Q: Portable + Send + Sync + 'static,
{
    let codec = codec::<R, Q>(response_bytes, request_bytes)?;
    let mut client = tonic::client::Grpc::new(channel).max_decoding_message_size(response_bytes).max_encoding_message_size(request_bytes);
    client.ready().await.map_err(|_| Error::Rpc(tonic::Status::unavailable("channel is not ready")))?;
    client
        .unary(request(Response::new(message), authorization), tonic::codegen::http::uri::PathAndQuery::from_static(path), codec)
        .await
        .map(tonic::Response::into_inner)
        .map_err(Error::Rpc)
}

fn status(error: Error) -> tonic::Status {
    match error {
        Error::Rpc(status) => status,
        error => tonic::Status::data_loss(error.to_string()),
    }
}

/* === Tests === */
