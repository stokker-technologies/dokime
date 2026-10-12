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
use std::{future::Future, num::NonZeroUsize, pin::Pin, sync::Arc};

// Third Party Imports
use harmos::catalog::{Catalog, Member, Projection, Visitor};
use harmos::grpc::{Codec, DecodeSettings, EncodeSettings};
use harmos::operations::{OperationDeclaration, StreamItem};
use harmos::{MemorySize, Protobuf};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

// Local Imports
use super::{Client, Error as ClientError, Result as ClientResult};
use crate::operations::runtime::{
    GetHealth, GetIdentity, Health, Identity, Request as RuntimeRequest,
};
use crate::operations::{Endpoint, Operations};

/* === Constants === */
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, Error>;
type Call = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;

/* === Structs === */
pub struct Request {
    path: &'static str,
    bytes: usize,
    call: Box<dyn FnOnce(Client) -> Call + Send>,
}

#[derive(Serialize)]
pub struct Description {
    pub path: &'static str,
    pub access: crate::operations::Access,
    pub contract: harmos::operations::Contract,
}

struct Invocations;

struct Entry {
    declaration: crate::operations::Declaration,
    prepare: fn(&[u8]) -> Result<Request>,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown operation: {0}")]
    Unknown(String),
    #[error("operation input exceeds its byte limit")]
    InputTooLarge,
    #[error("invalid operation input")]
    Input(#[source] serde_json::Error),
    #[error("operation response cannot be represented as JSON")]
    Output(#[source] serde_json::Error),
    #[error("invalid compiled operation contract")]
    Contract(#[from] harmos::operations::ContractError),
    #[error(transparent)]
    Client(#[from] ClientError),
}

/* === Traits === */

/* === Implementations === */
impl Visitor for Invocations {
    type Entry = Entry;
}

impl<O> Member<O> for Invocations
where
    O: Endpoint,
    O::Request: Protobuf + harmos::schema::Described + DeserializeOwned + Sync,
    O::Response: Protobuf + harmos::schema::Described + MemorySize + Serialize,
{
    const ENTRY: Entry = Entry {
        declaration: <crate::operations::Endpoints as Member<O>>::ENTRY,
        prepare: prepare::<O>,
    };
}

impl Request {
    pub fn from_json(operation: &str, input: &[u8]) -> Result<Self> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(Error::InputTooLarge);
        }
        let mut selected = None;
        <Operations as Catalog<Invocations>>::ENTRIES.try_for_each(|entry| {
            if entry.declaration.wire.operation.id == operation {
                selected = Some((entry.prepare)(input)?);
            }
            Ok::<_, Error>(())
        })?;
        selected.ok_or_else(|| Error::Unknown(operation.to_owned()))
    }
}

impl Client {
    pub fn operations(&self) -> Result<Vec<Description>> {
        if self.inner.cancellation.is_cancelled() {
            return Err(ClientError::Closed.into());
        }
        descriptions(&self.backend().methods)
    }

    pub async fn invoke(&self, request: Request) -> Result<Value> {
        self.require(request.path)?;
        if request.bytes > self.request_limit() {
            return Err(Error::InputTooLarge);
        }
        (request.call)(self.clone()).await
    }

    pub async fn health(&self) -> ClientResult<StreamItem<Health>> {
        self.unary::<GetHealth>(GetHealth::PATH, RuntimeRequest {})
            .await
    }

    pub async fn identity(&self) -> ClientResult<StreamItem<Identity>> {
        self.unary::<GetIdentity>(GetIdentity::PATH, RuntimeRequest {})
            .await
    }

    async fn unary<O>(
        &self,
        path: &'static str,
        message: O::Request,
    ) -> ClientResult<StreamItem<O::Response>>
    where
        O: OperationDeclaration,
        O::Request: Protobuf + Sync,
        O::Response: Protobuf + MemorySize,
    {
        self.require(path)?;
        let response_bytes =
            NonZeroUsize::new(self.response_limit()).ok_or(ClientError::IncompatibleApi)?;
        let request_bytes =
            NonZeroUsize::new(self.request_limit()).ok_or(ClientError::IncompatibleApi)?;
        let codec = Codec::<O::Response, Arc<O::Request>>::new(
            DecodeSettings {
                message_bytes: response_bytes,
                retained_bytes: response_bytes,
            },
            EncodeSettings {
                message_bytes: request_bytes,
                buffer_bytes: NonZeroUsize::new(request_bytes.get().min(8192)).unwrap(),
            },
        )
        .map_err(ClientError::Rpc)?;
        let mut client = tonic::client::Grpc::new(self.channel()?)
            .max_decoding_message_size(response_bytes.get())
            .max_encoding_message_size(request_bytes.get());
        self.execute(async {
            client
                .ready()
                .await
                .map_err(|_| tonic::Status::unavailable("channel is not ready"))?;
            client
                .unary(
                    self.request(Arc::new(message)),
                    tonic::codegen::http::uri::PathAndQuery::from_static(path),
                    codec,
                )
                .await
                .map(tonic::Response::into_inner)
        })
        .await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn descriptions(methods: &[String]) -> Result<Vec<Description>> {
    let mut descriptions = Vec::new();
    <Operations as Catalog<Invocations>>::ENTRIES.try_for_each(|entry| {
        if methods
            .iter()
            .any(|method| method == entry.declaration.path)
        {
            descriptions.push(Description {
                path: entry.declaration.path,
                access: entry.declaration.access,
                contract: entry.declaration.wire.contract()?,
            });
        }
        Ok::<_, Error>(())
    })?;
    Ok(descriptions)
}

fn prepare<O>(input: &[u8]) -> Result<Request>
where
    O: Endpoint,
    O::Request: Protobuf + DeserializeOwned + Sync,
    O::Response: Protobuf + MemorySize + Serialize,
{
    let message: O::Request = serde_json::from_slice(input).map_err(Error::Input)?;
    Ok(Request {
        path: O::PATH,
        bytes: message.encoded_len(),
        call: Box::new(move |client| {
            Box::pin(async move {
                let response = client.unary::<O>(O::PATH, message).await?;
                serde_json::to_value(&*response).map_err(Error::Output)
            })
        }),
    })
}

/* === Tests === */
