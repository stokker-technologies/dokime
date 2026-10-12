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
use std::marker::PhantomData;
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{MemorySize, storage::QueryValue};
use prost::Message;
use tonic::{
    Status,
    codec::{BufferSettings, Codec as TonicCodec, EncodeBuf, Encoder as TonicEncoder},
};

// Local Imports
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type RunReceipt = QueryValue<RunWire>;
pub type CommandReceipt = QueryValue<CommandWire>;
pub type RunCodec<U> = Codec<RunWire, U>;
pub type CommandCodec<U> = Codec<CommandWire, U>;

/* === Structs === */
pub struct RunWire(pub(super) v1::RunRequestReceipt);
pub struct CommandWire(pub(super) v1::ExecutionCommandReceipt);
pub struct Codec<W, U>(PhantomData<fn() -> (W, U)>);
pub struct Encoder<W>(PhantomData<fn() -> W>);

/* === Enums === */

/* === Traits === */
pub trait Wire: MemorySize + Send + 'static {
    type Message: Message;
    fn message(&self) -> &Self::Message;
}

/* === Implementations === */
impl Wire for RunWire {
    type Message = v1::RunRequestReceipt;
    fn message(&self) -> &Self::Message {
        &self.0
    }
}
impl Wire for CommandWire {
    type Message = v1::ExecutionCommandReceipt;
    fn message(&self) -> &Self::Message {
        &self.0
    }
}
impl MemorySize for RunWire {
    fn memory_size(&self) -> usize {
        let row = &self.0;
        let mut bytes = std::mem::size_of::<Self>();
        for value in [&row.request_id, &row.node_id, &row.cycle_id] {
            bytes = bytes.saturating_add(value.capacity());
        }
        for value in [row.submitted_by.as_ref(), row.interruption.as_ref()]
            .into_iter()
            .flatten()
        {
            bytes = bytes.saturating_add(value.capacity());
        }
        if let Some(claim) = &row.last_claim {
            bytes = bytes
                .saturating_add(claim.case_id.capacity())
                .saturating_add(claim.execution_id.capacity());
        }
        bytes
    }
}
impl MemorySize for CommandWire {
    fn memory_size(&self) -> usize {
        let row = &self.0;
        let mut bytes = std::mem::size_of::<Self>();
        for value in [&row.request_id, &row.command_id, &row.execution_id] {
            bytes = bytes.saturating_add(value.capacity());
        }
        for value in [
            row.step_id.as_ref(),
            row.attempt_id.as_ref(),
            row.detail.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(value.capacity());
        }
        bytes
    }
}
impl<W, U> Default for Codec<W, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<W: Wire, U: Message + Default + Send + 'static> TonicCodec for Codec<W, U> {
    type Encode = QueryValue<W>;
    type Decode = U;
    type Encoder = Encoder<W>;
    type Decoder = tonic_prost::ProstDecoder<U>;
    fn encoder(&mut self) -> Self::Encoder {
        Encoder(PhantomData)
    }
    fn decoder(&mut self) -> Self::Decoder {
        tonic_prost::ProstDecoder::new(BufferSettings::default())
    }
}
impl<W: Wire> TonicEncoder for Encoder<W> {
    type Item = QueryValue<W>;
    type Error = Status;
    fn encode(&mut self, owner: Self::Item, buffer: &mut EncodeBuf<'_>) -> Result<(), Status> {
        owner
            .message()
            .encode(buffer)
            .map_err(|error| Status::internal(error.to_string()))
    }
}

/* === Public Functions === */
pub(super) fn reserve(
    runtime: &dokime_app::RuntimeHandle,
    bytes: usize,
) -> Result<harmos::storage::StorageProductionPreparation, Status> {
    let bytes = NonZeroUsize::new(bytes)
        .ok_or_else(|| Status::resource_exhausted("invalid receipt memory allowance"))?;
    runtime
        .storage()
        .prepare_production(bytes)
        .map_err(memory_error)
}

pub(super) fn memory_error(error: harmos::Error) -> Status {
    let error = dokime_app::Error::from(error);
    super::super::error::detailed(Status::resource_exhausted(error.to_string()), &error)
}

/* === Private Functions === */

/* === Tests === */
