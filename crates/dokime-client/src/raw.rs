/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use tonic::Streaming as RawStreaming;

use std::time::Duration;

use prost::bytes::{Buf, BufMut};
use tokio_stream::Stream;
use tonic::codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder};

use super::{Client, Error, Result, stream::Input};

#[derive(Clone, Copy, Debug, Default)]
struct RawCodec;

#[derive(Clone, Copy, Debug, Default)]
struct RawEncoder;

#[derive(Clone, Copy, Debug, Default)]
struct RawDecoder;

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
        let (mut grpc, path) = self.raw_call(path, request.len())?;
        let mut request = self.request(request);
        request.set_timeout(deadline);
        self.with_timeout(deadline)
            .execute(async move {
                grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                grpc.unary(request, path, RawCodec).await.map(tonic::Response::into_inner)
            })
            .await
    }

    pub async fn stream_raw<S>(&self, path: &str, input: S, deadline: Duration) -> Result<RawStreaming<Vec<u8>>>
    where
        S: Stream<Item = Vec<u8>> + Send + 'static,
    {
        let (mut grpc, path) = self.raw_call(path, 0)?;
        let input = Input::new(input, self.inner.cancellation.child_token());
        let mut request = self.request(input);
        request.set_timeout(deadline);
        self.with_timeout(deadline)
            .execute(async move {
                grpc.ready().await.map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
                grpc.streaming(request, path, RawCodec).await.map(tonic::Response::into_inner)
            })
            .await
    }

    pub fn max_request_bytes(&self) -> usize {
        self.request_limit()
    }

    fn raw_call(
        &self, path: &str, request_bytes: usize,
    ) -> Result<(tonic::client::Grpc<tonic::transport::Channel>, tonic::codegen::http::uri::PathAndQuery)> {
        self.require(path)?;
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
