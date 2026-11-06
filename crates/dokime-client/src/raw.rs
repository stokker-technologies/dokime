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

pub use tonic::Streaming as RawStreaming;

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use prost::bytes::{Buf, BufMut};
use tokio_stream::Stream;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

// Local Imports
use super::{Client, Error, Result, stream::Input};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, Default)]
struct RawCodec;

#[derive(Clone, Copy, Debug, Default)]
struct RawEncoder;

#[derive(Clone, Copy, Debug, Default)]
struct RawDecoder;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Codec for RawCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;
    type Encoder = RawEncoder;
    type Decoder = RawDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        RawEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        RawDecoder
    }
}

impl Encoder for RawEncoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn encode(&mut self, item: Self::Item, buffer: &mut EncodeBuf<'_>) -> std::result::Result<(), Self::Error> {
        buffer.put_slice(&item);
        Ok(())
    }
}

impl Decoder for RawDecoder {
    type Item = Vec<u8>;
    type Error = tonic::Status;

    fn decode(&mut self, buffer: &mut DecodeBuf<'_>) -> std::result::Result<Option<Self::Item>, Self::Error> {
        let mut item = vec![0; buffer.remaining()];
        buffer.copy_to_slice(&mut item);
        Ok(Some(item))
    }
}

impl Client {
    pub async fn call_raw(&self, path: &str, request: Vec<u8>, deadline: Duration) -> Result<Vec<u8>> {
        self.authorize_raw(path).await?;
        let (grpc, route) = self.raw_call(path, request.len())?;
        self.with_timeout(deadline)
            .execute(async {
                super::transport::retry_unavailable(path, || {
                    let mut grpc = grpc.clone();
                    let route = route.clone();
                    let mut request = self.request(request.clone());
                    request.set_timeout(deadline);
                    async move {
                        grpc.ready().await.map_err(|_| Error::Rpc(tonic::Status::unavailable("channel is not ready")))?;
                        grpc.unary(request, route, RawCodec).await.map(tonic::Response::into_inner).map_err(Error::Rpc)
                    }
                })
                .await
                .map_err(|error| match error {
                    Error::Rpc(status) => status,
                    error => tonic::Status::internal(error.to_string()),
                })
            })
            .await
    }

    pub async fn stream_raw<S>(&self, path: &str, input: S, deadline: Duration) -> Result<RawStreaming<Vec<u8>>>
    where
        S: Stream<Item = Vec<u8>> + Send + 'static,
    {
        self.stream_raw_response(path, input, deadline).await.map(tonic::Response::into_inner)
    }

    /// Opens a raw stream while retaining its initial response metadata.
    pub async fn stream_raw_response<S>(&self, path: &str, input: S, deadline: Duration) -> Result<tonic::Response<RawStreaming<Vec<u8>>>>
    where
        S: Stream<Item = Vec<u8>> + Send + 'static,
    {
        self.authorize_raw(path).await?;
        let (mut grpc, path) = self.raw_call(path, 0)?;
        let input = Input::new(input, self.inner.cancellation.child_token());
        let mut request = self.request(input);
        request.set_timeout(deadline);
        self.with_timeout(deadline)
            .execute(async move {
                grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                grpc.streaming(request, path, RawCodec).await
            })
            .await
    }

    pub fn max_request_bytes(&self) -> usize {
        self.request_limit()
    }

    async fn authorize_raw(&self, path: &str) -> Result<()> {
        if self.backend().methods.iter().any(|method| method == path) || path.starts_with("/dokime.v1.") {
            return self.require(path);
        }
        // External Bindings Are Advertised By The Authorized Operation Catalog.
        self.verify(true).await?;
        let operations = self.operations().await.map_err(|error| match error {
            super::operations::Error::Client(error) => error,
            _ => Error::IncompatibleApi,
        })?;
        if operations.iter().any(|entry| entry.grpc_available && entry.contract.grpc().is_some_and(|binding| binding.path() == path)) {
            return Ok(());
        }
        Err(Error::Rpc(tonic::Status::permission_denied("operation is not available to this caller")))
    }

    fn raw_call(
        &self, path: &str, request_bytes: usize,
    ) -> Result<(tonic::client::Grpc<tonic::transport::Channel>, tonic::codegen::http::uri::PathAndQuery)> {
        if request_bytes > self.request_limit() {
            return Err(Error::Rpc(tonic::Status::out_of_range("request exceeds the negotiated message limit")));
        }
        let path = tonic::codegen::http::uri::PathAndQuery::try_from(path)
            .map_err(|_| Error::Rpc(tonic::Status::invalid_argument("invalid method path")))?;
        let grpc = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(self.response_limit())
            .max_encoding_message_size(self.request_limit());
        Ok((grpc, path))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
