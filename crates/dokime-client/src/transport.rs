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
use harmos::grpc::{Codec, DecodeSettings, Encode, EncodeSettings, InputEnvelope, OutputEnvelope, PreparedFrame, Transcoded, Wire};
use harmos::operations::{Direction, OperationDeclaration, Response, StreamItem};
use tonic::{
    metadata::{Ascii, MetadataValue},
    transport::Channel,
};

// Local Imports
use super::{Client, Error, ResponseStream, Result, request, stream::Input};

/* === Constants === */

const DECODE_RETAINED_BYTES: u64 = 67_108_864;
const ENCODE_BUFFER_BYTES: u64 = 8_192;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Client {
    pub(super) async fn download<O>(
        &self, path: &'static str, message: O::Request,
    ) -> Result<ResponseStream<<O::Output as Direction>::Item>>
    where
        O: OperationDeclaration,
        O::Request: Sync,
        O::Response: Portable + MemorySize,
        <O::Output as Direction>::Item: Portable + MemorySize,
    {
        self.require(path)?;
        let report = matches!(path, "/dokime.v1.Reports/DownloadCycleReport" | "/dokime.v1.Reports/DownloadExecutionReport");
        let response_bytes = if report { self.response_limit().min(2_097_152) } else { self.response_limit() };
        let codec = codec::<OutputEnvelope<<O::Output as Direction>::Item, O::Response>, Response<O::Request>>(
            response_bytes,
            self.request_limit(),
        )?;
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(response_bytes)
            .max_encoding_message_size(self.request_limit());
        let incoming = self
            .execute(async {
                client.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                // A Stream Carries No Total Deadline: Opening Is Bounded Here, Each Later Frame By The Idle Deadline.
                client
                    .server_streaming(
                        request(Response::new(message), self.inner.authorization.as_ref()),
                        tonic::codegen::http::uri::PathAndQuery::from_static(path),
                        codec,
                    )
                    .await
                    .map(tonic::Response::into_inner)
            })
            .await?;
        let idle = self.timeout;
        let frames = stream::unfold(Some(incoming), move |state| async move {
            let mut incoming = state?;
            let next = match idle_frame(idle, incoming.next()).await {
                Ok(next) => next,
                Err(status) => return Some((Err(status), None)),
            };
            match next {
                Some(Err(status)) => Some((Err(status), None)),
                None => Some((Err(tonic::Status::data_loss("operation stream has no completion")), None)),
                Some(Ok(frame)) => match OutputEnvelope::project(frame) {
                    Err(error) => Some((Err(tonic::Status::data_loss(error.to_string())), None)),
                    Ok(OutputEnvelope::Item(item)) => Some((value(item).map_err(status), Some(incoming))),
                    Ok(OutputEnvelope::Response(_)) => {
                        // Final Status Must Be Observed Before Reporting End Of Stream.
                        match idle_frame(idle, incoming.next()).await {
                            Err(status) => Some((Err(status), None)),
                            Ok(Some(Err(status))) => Some((Err(status), None)),
                            Ok(Some(Ok(_))) => Some((Err(tonic::Status::data_loss("operation stream continued after completion")), None)),
                            Ok(None) => None,
                        }
                    }
                },
            }
        });
        if report { ResponseStream::prefetched(self, frames, response_bytes) } else { ResponseStream::new(self, frames) }
    }

    pub(super) async fn upload<O, S>(&self, path: &'static str, message: O::Request, items: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        O::Request: Sync,
        O::Response: Portable + MemorySize,
        <O::Input as Direction>::Item: Portable + Sync,
        S: tokio_stream::Stream<Item = <O::Input as Direction>::Item> + Send + 'static,
    {
        self.require(path)?;
        let opening = stream::once(async move { Response::new(InputEnvelope::Request(message)) });
        let frames = opening.chain(items.map(|item| Response::new(InputEnvelope::Item(item))));
        self.upload_frames::<O, _, _>(path, frames).await
    }

    pub(super) async fn upload_prepared<O, S>(&self, path: &'static str, message: O::Request, items: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        S: tokio_stream::Stream<Item = PreparedFrame> + Send + 'static,
    {
        self.require(path)?;
        let opening = harmos::grpc::prepare_input_request::<O>(&message, self.request_limit()).map_err(Error::Rpc)?;
        drop(message);
        let frames = stream::once(async move { opening }).chain(items);
        self.upload_frames::<O, _, _>(path, frames).await
    }

    async fn upload_frames<O, E, S>(&self, path: &'static str, frames: S) -> Result<O::Response>
    where
        O: OperationDeclaration,
        E: Encode<Transcoded> + Sync,
        S: tokio_stream::Stream<Item = E> + Send + 'static,
    {
        let codec = codec::<O::Response, E>(self.response_limit(), self.request_limit())?;
        let cancellation = self.inner.cancellation.child_token();
        let _cancel = cancellation.clone().drop_guard();
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

pub(super) fn value<T>(reply: StreamItem<T>) -> Result<T> {
    // Transfer The Decoded Allocation; Never Clone A Payload To Drop Its Owner.
    reply.into_request().map(|(value, _claims)| value).map_err(|error| Error::Rpc(tonic::Status::data_loss(error.to_string())))
}

pub(super) fn codec<D, E>(response_bytes: usize, request_bytes: usize) -> Result<Codec<Transcoded, D, E>>
where
    Transcoded: Wire<D>,
    D: MemorySize + Send + 'static,
    E: Encode<Transcoded> + Sync,
{
    let response_bytes = NonZeroUsize::new(response_bytes).ok_or(Error::IncompatibleApi)?;
    let request_bytes = NonZeroUsize::new(request_bytes).ok_or(Error::IncompatibleApi)?;
    let retained_bytes =
        NonZeroUsize::new(usize::try_from(DECODE_RETAINED_BYTES).map_err(|_| Error::IncompatibleApi)?).ok_or(Error::IncompatibleApi)?;
    Codec::new(
        DecodeSettings {
            message_bytes: response_bytes,
            retained_bytes,
        },
        EncodeSettings {
            message_bytes: request_bytes,
            buffer_bytes: NonZeroUsize::new(
                request_bytes.get().min(usize::try_from(ENCODE_BUFFER_BYTES).map_err(|_| Error::IncompatibleApi)?),
            )
            .ok_or(Error::IncompatibleApi)?,
        },
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
    let codec = codec::<R, Response<Q>>(response_bytes, request_bytes)?;
    let mut client = tonic::client::Grpc::new(channel).max_decoding_message_size(response_bytes).max_encoding_message_size(request_bytes);
    client.ready().await.map_err(|_| Error::Rpc(tonic::Status::unavailable("channel is not ready")))?;
    client
        .unary(request(Response::new(message), authorization), tonic::codegen::http::uri::PathAndQuery::from_static(path), codec)
        .await
        .map(tonic::Response::into_inner)
        .map_err(Error::Rpc)
}

/* === Private Functions === */

// A Live Stream Fails Only When No Frame, Heartbeats Included, Arrives Within The Idle Deadline.
async fn idle_frame<T>(idle: Option<std::time::Duration>, next: impl Future<Output = T>) -> std::result::Result<T, tonic::Status> {
    match idle {
        Some(idle) => tokio::time::timeout(idle, next).await.map_err(|_| tonic::Status::deadline_exceeded("stream idle deadline elapsed")),
        None => Ok(next.await),
    }
}

fn status(error: Error) -> tonic::Status {
    match error {
        Error::Rpc(status) => status,
        error => tonic::Status::data_loss(error.to_string()),
    }
}

/* === Tests === */
