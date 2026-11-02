/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate Reflected Responses And Serialize Byte Fields Without JSON Value Expansion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Write, num::NonZeroUsize};

// Third Party Imports
use harmos::grpc::dynamic::Operation;
use harmos_sdk::grpc::{DecodeInput, Validator};
use prost::bytes::Buf;
use prost_reflect::{DynamicMessage, MessageDescriptor, ReflectMessage, Value};
use tonic::{Status, codec::DecodeBuf};

// Local Imports
use super::output::invalid;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct JsonMessage(pub(super) DynamicMessage);

#[derive(Clone)]
pub(super) struct Messages {
    pub(super) operation: Operation,
    response: MessageDescriptor,
    response_validator: Validator,
    output_validator: Option<Validator>,
    limit: usize,
}

pub(super) struct Codec(pub(super) Messages);

pub(super) struct Decoder(Messages);

struct Ceiling(usize);

/* === Enums === */

#[derive(serde::Serialize)]
#[serde(tag = "event", content = "value", rename_all = "lowercase")]
pub enum JsonEvent {
    Item(DynamicMessage),
    Completion(DynamicMessage),
}

/* === Traits === */

/* === Implementations === */

impl Messages {
    pub(super) fn new(contract: &harmos::operations::Contract, response: MessageDescriptor, limit: NonZeroUsize) -> Result<Self, Status> {
        let validators = Validator::from_schema(contract.schema(), &Default::default()).map_err(|error| invalid(error.to_string()))?;
        Ok(Self {
            operation: Operation::new(contract, limit)?,
            response,
            response_validator: validators.at(contract.response().index()).map_err(|error| invalid(error.to_string()))?,
            output_validator: contract
                .output()
                .map(|view| validators.at(view.index()))
                .transpose()
                .map_err(|error| invalid(error.to_string()))?,
            limit: limit.get(),
        })
    }

    pub(super) fn check(&self, message: &DynamicMessage) -> Result<(), Status> {
        // Count Serialized Base64 Without Allocating A JSON Buffer Or Value Tree.
        serde_json::to_writer(Ceiling(self.limit), message).map_err(|error| {
            if error.is_io() {
                Status::resource_exhausted("Response JSON exceeds its byte ceiling (bytes use base64); use --output <file> for chunked downloads or a typed client for other byte responses")
            } else {
                invalid(error.to_string())
            }
        })?;
        Ok(())
    }

    pub(super) fn decode(&self, bytes: &[u8]) -> Result<JsonEvent, Status> {
        if bytes.len() > self.limit {
            return Err(Status::resource_exhausted("operation message exceeds byte ceiling"));
        }
        // Use The Shared Schema Validator Before Reflection Allocates A Response.
        if let Some(output) = &self.output_validator {
            DecodeInput::new(bytes)
                .fields(&mut |field| {
                    let validator = match field.tag {
                        1 => output,
                        2 => &self.response_validator,
                        _ => return Ok(()),
                    };
                    validator.validate(field.bytes()?).map_err(|error| harmos::MessageError::new(error.to_string()))
                })
                .map_err(|error| invalid(error.to_string()))?;
        } else {
            self.response_validator.validate(bytes).map_err(|error| invalid(error.to_string()))?;
        }
        let envelope = DynamicMessage::decode(self.response.clone(), bytes).map_err(|error| invalid(error.to_string()))?;
        if self.output_validator.is_none() {
            return Ok(JsonEvent::Completion(envelope));
        }
        if let Some(item) = message(&envelope, "item") {
            return Ok(JsonEvent::Item(item));
        }
        let response = message(&envelope, "response").ok_or_else(|| invalid("empty output envelope"))?;
        Ok(JsonEvent::Completion(response))
    }
}

impl tonic::codec::Codec for Codec {
    type Encode = harmos::grpc::dynamic::Payload;
    type Decode = JsonEvent;
    type Encoder = harmos::grpc::dynamic::Encoder;
    type Decoder = Decoder;

    fn encoder(&mut self) -> Self::Encoder {
        self.0.operation.codec().encoder()
    }

    fn decoder(&mut self) -> Self::Decoder {
        Decoder(self.0.clone())
    }
}

impl tonic::codec::Decoder for Decoder {
    type Item = JsonEvent;
    type Error = Status;

    fn decode(&mut self, source: &mut DecodeBuf<'_>) -> Result<Option<JsonEvent>, Status> {
        let bytes = source.copy_to_bytes(source.remaining());
        let event = self.0.decode(&bytes)?;
        let message = match &event {
            JsonEvent::Item(message) | JsonEvent::Completion(message) => message,
        };
        self.0.check(message)?;
        Ok(Some(event))
    }
}

impl Write for Ceiling {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.checked_sub(bytes.len()).ok_or_else(|| std::io::Error::other("JSON byte ceiling"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(super) fn message(parent: &DynamicMessage, name: &str) -> Option<DynamicMessage> {
    let field = parent.descriptor().get_field_by_name(name)?;
    if !parent.has_field(&field) {
        return None;
    }
    match parent.get_field(&field).as_ref() {
        Value::Message(message) => Some(message.clone()),
        _ => None,
    }
}

/* === Tests === */
