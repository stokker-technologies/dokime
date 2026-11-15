/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Message

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fmt;

// Third Party Imports
use prost::{
    Message,
    bytes::{Buf, BufMut},
    encoding::{DecodeContext, WireType},
};
use prost_reflect::ReflectMessage;

// Local Imports
use harmos::Message as HarmosMessage;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The generated message itself on the wire; no envelope, projection or duplicate schema.
#[derive(Clone, Default, PartialEq)]
pub struct Wire<T>(pub T);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: fmt::Debug> fmt::Debug for Wire<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl<T: Message> Message for Wire<T> {
    fn encode_raw(&self, buffer: &mut impl BufMut) {
        self.0.encode_raw(buffer);
    }
    fn merge_field(
        &mut self, tag: u32, wire: WireType, buffer: &mut impl Buf, context: DecodeContext,
    ) -> std::result::Result<(), prost::DecodeError> {
        self.0.merge_field(tag, wire, buffer, context)
    }
    fn encoded_len(&self) -> usize {
        self.0.encoded_len()
    }
    fn clear(&mut self) {
        self.0.clear();
    }
}

impl<T: ReflectMessage + Default + 'static> HarmosMessage for Wire<T> {
    fn name() -> String {
        T::default().descriptor().full_name().into()
    }
    fn types() -> prost_types::FileDescriptorSet {
        let descriptor = T::default().descriptor();
        let types = prost_types::FileDescriptorSet {
            file: descriptor.parent_pool().files().map(|file| file.file_descriptor_proto().clone()).collect(),
        };
        harmos_protocol::schema::operation_types(&types, &[descriptor.full_name()]).expect("Generated Protocol Message Descriptor")
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
