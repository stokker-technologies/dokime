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

// Third Party Imports
use harmos::storage::QueryValue;
use prost::{Message, bytes::BufMut};
use tonic::{
    Status,
    codec::{BufferSettings, Codec as TonicCodec, EncodeBuf, Encoder as TonicEncoder},
};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Bytes = QueryValue<Vec<u8>>;

/* === Structs === */
pub struct Codec<U>(PhantomData<fn() -> U>);
pub struct Encoder;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<U> Default for Codec<U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<U: Message + Default + Send + 'static> TonicCodec for Codec<U> {
    type Encode = Bytes;
    type Decode = U;
    type Encoder = Encoder;
    type Decoder = tonic_prost::ProstDecoder<U>;

    fn encoder(&mut self) -> Encoder {
        Encoder
    }
    fn decoder(&mut self) -> Self::Decoder {
        tonic_prost::ProstDecoder::new(BufferSettings::default())
    }
}

impl TonicEncoder for Encoder {
    type Item = Bytes;
    type Error = Status;

    fn encode(&mut self, value: Bytes, buffer: &mut EncodeBuf<'_>) -> Result<(), Status> {
        // Retain The Original Loan Until Transport Copies The Canonical Bytes.
        buffer.put_slice(&value);
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
