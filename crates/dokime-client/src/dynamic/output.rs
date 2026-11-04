/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Decode File Chunks Before Dynamic JSON Conversion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashSet, num::NonZeroUsize, path::PathBuf};

// Third Party Imports
use prost::bytes::Buf;
use prost_reflect::{Kind, MessageDescriptor, Value};
use tonic::codec::DecodeBuf;

// Local Imports
use super::{
    Client, JsonEvent, JsonMessage,
    json::{self, Messages, message},
};
use crate::{
    Download,
    download::{self, Frame, Policy},
    operations::{Error, Result},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Codec {
    messages: Messages,
}

struct Decoder(Codec);

/* === Enums === */

pub enum FileOutput {
    Download(Download<JsonMessage>),
    Json(crate::ResponseStream<JsonEvent>),
}

enum Output {
    Frame(Frame<JsonMessage>),
    Completion,
}

/* === Traits === */

/* === Implementations === */

impl Client {
    pub async fn call_operation_output(
        &self, identifier: &str, owner: Option<&str>, request: &[u8], inputs: Vec<Vec<u8>>, destination: PathBuf, overwrite: bool,
    ) -> Result<FileOutput> {
        super::validate_input(request, &inputs)?;
        let provider = self.select_operation(identifier, owner).await?;
        let binding = provider.contract.grpc().ok_or(Error::Catalog)?;
        let service = harmos::grpc::descriptor(&binding.package, &binding.service, &[(&binding.method, &provider.contract)])
            .map_err(|_| Error::Catalog)?;
        let method = service.methods().next().ok_or(Error::Catalog)?;
        let response = method.output();
        let download = method.is_server_streaming()
            && message_field(&response, "item")
                .filter(|item| !has_bytes_except(item, "payload"))
                .and_then(|item| message_field(&item, "payload"))
                .is_some_and(|payload| {
                    let chunk = message_field(&payload, "Chunk").and_then(|wrapper| message_field(&wrapper, "field_0"));
                    let complete = message_field(&payload, "Complete").and_then(|wrapper| message_field(&wrapper, "field_0"));
                    chunk.is_some_and(|chunk| {
                        chunk.get_field_by_name("data").is_some_and(|field| field.kind() == Kind::Bytes)
                            && chunk.get_field_by_name("offset").is_some_and(|field| field.kind() == Kind::Uint64)
                    }) && complete.is_some_and(|complete| {
                        complete.get_field_by_name("size_bytes").is_some_and(|field| field.kind() == Kind::Uint64)
                            && complete.get_field_by_name("sha256").is_some_and(|field| field.kind() == Kind::String)
                    })
                });
        let limit = NonZeroUsize::new(self.request_limit().min(self.response_limit())).ok_or(crate::Error::IncompatibleApi)?;
        let messages = Messages::new(&provider.contract, response.clone(), limit).map_err(crate::Error::Rpc)?;
        let operation = messages.operation.clone();
        if !download || message_field(&response, "response").is_some_and(has_bytes) {
            let codec = json::Codec(messages);
            let stream = self
                .invoke_operation(&provider, &operation, request, inputs, codec, |event| matches!(event, JsonEvent::Completion(_)))
                .await?;
            return Ok(FileOutput::Json(stream));
        }
        download::validate_destination(&destination, overwrite)?;
        let codec = Codec { messages };
        let stream =
            self.invoke_operation(&provider, &operation, request, inputs, codec, |output| matches!(output, Output::Completion)).await?;
        let result = download::receive_with(
            || async {
                match stream.message().await? {
                    Some(Output::Frame(frame)) => Ok(Some(frame)),
                    Some(Output::Completion) | None => Ok(None),
                }
            },
            destination,
            Ok,
            Policy { overwrite, maximum_bytes: u64::MAX },
        )
        .await;
        stream.close().await;
        Ok(FileOutput::Download(result?))
    }
}

impl tonic::codec::Codec for Codec {
    type Encode = harmos::grpc::dynamic::Payload;
    type Decode = Output;
    type Encoder = harmos::grpc::dynamic::Encoder;
    type Decoder = Decoder;

    fn encoder(&mut self) -> Self::Encoder {
        self.messages.operation.codec().encoder()
    }

    fn decoder(&mut self) -> Self::Decoder {
        Decoder(Codec { messages: self.messages.clone() })
    }
}

impl tonic::codec::Decoder for Decoder {
    type Item = Output;
    type Error = tonic::Status;

    fn decode(&mut self, source: &mut DecodeBuf<'_>) -> std::result::Result<Option<Output>, tonic::Status> {
        let bytes = source.copy_to_bytes(source.remaining());
        self.0.decode(&bytes).map(Some)
    }
}

impl Codec {
    fn decode(&self, bytes: &[u8]) -> std::result::Result<Output, tonic::Status> {
        let item = match self.messages.decode(bytes)? {
            JsonEvent::Item(item) => item,
            JsonEvent::Completion(_) => return Ok(Output::Completion),
        };
        let payload = message(&item, "payload").ok_or_else(|| invalid("download has no payload"))?;
        if let Some(chunk) = message(&payload, "Chunk").and_then(|wrapper| message(&wrapper, "field_0")) {
            let offset = match chunk.get_field_by_name("offset").as_deref() {
                Some(Value::U64(offset)) => *offset,
                _ => return Err(invalid("download chunk has no offset")),
            };
            let data = match chunk.get_field_by_name("data").as_deref() {
                Some(Value::Bytes(data)) => data.to_vec(),
                _ => return Err(invalid("download chunk has no bytes")),
            };
            return Ok(Output::Frame(Frame::Chunk(offset, data)));
        }
        if let Some(complete) = message(&payload, "Complete").and_then(|wrapper| message(&wrapper, "field_0")) {
            let size = match complete.get_field_by_name("size_bytes").as_deref() {
                Some(Value::U64(size)) => *size,
                _ => return Err(invalid("download completion has no size")),
            };
            let hash = match complete.get_field_by_name("sha256").as_deref() {
                Some(Value::String(hash)) => hash.clone(),
                _ => return Err(invalid("download completion has no SHA-256")),
            };
            return Ok(Output::Frame(Frame::Complete(size, hash)));
        }
        // Retain Bounded Header Metadata
        for name in ["Report", "Metadata", "Header"] {
            let Some(wrapper) = message(&payload, name) else {
                continue;
            };
            let header = message(&wrapper, "field_0").ok_or_else(|| invalid("download header has no metadata"))?;
            let expected = match (header.get_field_by_name("size_bytes").as_deref(), header.get_field_by_name("sha256").as_deref()) {
                (Some(Value::U64(size)), Some(Value::String(hash))) => Some((*size, hash.clone())),
                _ => None,
            };
            self.messages.check(&header)?;
            return Ok(Output::Frame(Frame::Header(JsonMessage(header), expected)));
        }
        Err(invalid("expected download header, chunk or completion"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn message_field(descriptor: &MessageDescriptor, name: &str) -> Option<MessageDescriptor> {
    match descriptor.get_field_by_name(name)?.kind() {
        Kind::Message(message) => Some(message),
        _ => None,
    }
}

fn has_bytes_except(message: &MessageDescriptor, except: &str) -> bool {
    message.fields().filter(|field| field.name() != except).any(|field| match field.kind() {
        Kind::Bytes => true,
        Kind::Message(message) => has_bytes(message),
        _ => false,
    })
}

fn has_bytes(root: MessageDescriptor) -> bool {
    let mut visited = HashSet::new();
    let mut pending = vec![root];
    while let Some(message) = pending.pop() {
        if !visited.insert(message.full_name().to_owned()) {
            continue;
        }
        for field in message.fields() {
            match field.kind() {
                Kind::Bytes => return true,
                Kind::Message(message) => pending.push(message),
                _ => {}
            }
        }
    }
    false
}

pub(super) fn invalid(message: impl Into<String>) -> tonic::Status {
    tonic::Status::data_loss(message.into())
}

/* === Tests === */
