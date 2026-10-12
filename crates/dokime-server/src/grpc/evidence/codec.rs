/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod inputs {
    include!(concat!(env!("OUT_DIR"), "/evidence-inputs.rs"));
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::marker::PhantomData;

// Third Party Imports
use prost::Message;
use prost::bytes::Buf;
use prost::encoding::{DecodeContext, WireType, decode_key, decode_varint, skip_field};
use tonic::Status;
use tonic::codec::{
    BufferSettings, Codec as TonicCodec, DecodeBuf, Decoder as TonicDecoder, EncodeBuf,
    Encoder as TonicEncoder,
};

// Local Imports
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Page(pub(super) harmos::storage::QueryValue<WirePage>);

pub(super) struct WirePage(pub(super) v1::RecordPage);

pub struct Codec<U>(PhantomData<fn() -> U>);

pub struct Encoder;

pub struct Decoder<U> {
    inner: tonic_prost::ProstDecoder<U>,
}

/* === Enums === */

/* === Traits === */
pub trait Input: Message + Default + Send + 'static {
    const SELECTION: u32;
}

/* === Implementations === */
impl harmos::MemorySize for WirePage {
    fn memory_size(&self) -> usize {
        let page = &self.0;
        let mut bytes = std::mem::size_of::<Self>().saturating_add(
            page.records
                .capacity()
                .saturating_mul(std::mem::size_of::<v1::CapturedRecord>()),
        );
        for record in &page.records {
            bytes = bytes.saturating_add(super::wire::owned_memory(record));
        }
        for cursor in [page.scanned.as_ref(), page.through.as_ref()]
            .into_iter()
            .flatten()
        {
            bytes = bytes
                .saturating_add(cursor.epoch.capacity())
                .saturating_add(cursor.stream.capacity());
        }
        bytes
    }
}

impl<U> Default for Codec<U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<U: Input> TonicCodec for Codec<U> {
    type Encode = Page;
    type Decode = U;
    type Encoder = Encoder;
    type Decoder = Decoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        Encoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        Decoder {
            inner: tonic_prost::ProstDecoder::new(BufferSettings::default()),
        }
    }
}

impl<U: Input> TonicDecoder for Decoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, buffer: &mut DecodeBuf<'_>) -> Result<Option<U>, Status> {
        let input = buffer.chunk();
        if input.len() != buffer.remaining() {
            return Err(Status::internal(
                "Evidence decoder requires a complete message buffer",
            ));
        }
        check_input::<U>(input)?;
        self.inner.decode(buffer)
    }
}

impl TonicEncoder for Encoder {
    type Item = Page;
    type Error = Status;

    fn encode(&mut self, page: Self::Item, buffer: &mut EncodeBuf<'_>) -> Result<(), Status> {
        // Borrow The Canonical Message While Its Same Output Loan Remains Live.
        let message: &WirePage = &page.0;
        message
            .0
            .encode(buffer)
            .map_err(|error| Status::internal(error.to_string()))
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn check_input<U: Input>(mut input: &[u8]) -> Result<(), Status> {
    let mut counts = [0usize; 3];
    while !input.is_empty() {
        let (tag, wire) = decode_key(&mut input).map_err(malformed)?;
        if tag == U::SELECTION && wire == WireType::LengthDelimited {
            check_selection(delimited(&mut input)?, &mut counts)?;
        } else {
            skip_field(wire, tag, &mut input, DecodeContext::default()).map_err(malformed)?;
        }
    }
    Ok(())
}

fn malformed(error: prost::DecodeError) -> Status {
    Status::invalid_argument(error.to_string())
}

fn delimited<'a>(input: &mut &'a [u8]) -> Result<&'a [u8], Status> {
    let len = usize::try_from(decode_varint(input).map_err(malformed)?)
        .map_err(|_| Status::invalid_argument("selection length overflow"))?;
    let bytes = input
        .get(..len)
        .ok_or_else(|| Status::invalid_argument("truncated selection"))?;
    *input = &input[len..];
    Ok(bytes)
}

fn alternative(count: &mut usize, limit: usize) -> Result<(), Status> {
    if *count >= limit {
        return Err(Status::invalid_argument(
            "selection exceeds its alternatives limit",
        ));
    }
    *count += 1;
    Ok(())
}

fn check_selection(mut input: &[u8], counts: &mut [usize; 3]) -> Result<(), Status> {
    while !input.is_empty() {
        let (tag, wire) = decode_key(&mut input).map_err(malformed)?;
        if (tag == inputs::RECORD_TYPES || tag == inputs::SOURCES)
            && wire == WireType::LengthDelimited
        {
            let index = usize::from(tag == inputs::SOURCES);
            alternative(&mut counts[index], super::selection::MAX_ALTERNATIVES)?;
            delimited(&mut input)?;
        } else if tag == inputs::SEVERITIES && wire == WireType::LengthDelimited {
            let mut packed = delimited(&mut input)?;
            while !packed.is_empty() {
                alternative(&mut counts[2], super::selection::MAX_SEVERITIES)?;
                decode_varint(&mut packed).map_err(malformed)?;
            }
        } else if tag == inputs::SEVERITIES && wire == WireType::Varint {
            alternative(&mut counts[2], super::selection::MAX_SEVERITIES)?;
            decode_varint(&mut input).map_err(malformed)?;
        } else {
            skip_field(wire, tag, &mut input, DecodeContext::default()).map_err(malformed)?;
        }
    }
    Ok(())
}

/* === Tests === */
