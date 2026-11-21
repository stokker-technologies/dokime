/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plain and Structured Payloads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use crate::{Message, Result, message::Wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/// A Plain Value or Structured Protobuf Row at the SDK Boundary.
pub trait Payload: Sized + 'static {
    type Message: Message;
    fn into_message(self) -> Self::Message;
    fn from_message(message: Self::Message) -> Self;
}

/* === Implementations === */

impl<T: Message> Payload for T {
    type Message = T;
    fn into_message(self) -> T {
        self
    }
    fn from_message(message: T) -> T {
        message
    }
}

impl Payload for bool {
    type Message = Wire<bool>;
    fn into_message(self) -> Self::Message {
        Wire(self)
    }
    fn from_message(message: Self::Message) -> Self {
        message.0
    }
}

impl Payload for String {
    type Message = Wire<String>;
    fn into_message(self) -> Self::Message {
        Wire(self)
    }
    fn from_message(message: Self::Message) -> Self {
        message.0
    }
}

impl Payload for i64 {
    type Message = Wire<i64>;
    fn into_message(self) -> Self::Message {
        Wire(self)
    }
    fn from_message(message: Self::Message) -> Self {
        message.0
    }
}

impl Payload for f64 {
    type Message = Wire<f64>;
    fn into_message(self) -> Self::Message {
        Wire(self)
    }
    fn from_message(message: Self::Message) -> Self {
        message.0
    }
}

/* === Public Functions === */

pub(crate) fn decode<T: Payload>(bytes: &[u8]) -> Result<T> {
    let message = <T::Message as prost::Message>::decode(bytes)
        .map_err(|error| crate::Error::Invalid(format!("Decode Payload: Supply a Matching Record: {error}")))?;
    Ok(T::from_message(message))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Payload, decode};
    use prost::Message;
    use prost_reflect::{DescriptorPool, DynamicMessage, Value};

    fn roundtrip<T: Payload + Clone + PartialEq + core::fmt::Debug>(value: T, field: Value) {
        let expected = value.clone();
        let message = value.into_message();
        let bytes = message.encode_to_vec();
        assert_eq!(decode::<T>(&bytes).unwrap(), expected);
        let pool = DescriptorPool::from_file_descriptor_set(<T::Message as ::harmos::Message>::types()).unwrap();
        let descriptor = pool.get_message_by_name(&<T::Message as ::harmos::Message>::name()).unwrap();
        let dynamic = DynamicMessage::decode(descriptor, bytes.as_slice()).unwrap();
        assert_eq!(dynamic.get_field_by_name("value").unwrap().as_ref(), &field);
    }

    #[test]
    fn plain_values_roundtrip_well_known_wrappers() {
        roundtrip(false, Value::Bool(false));
        roundtrip(true, Value::Bool(true));
        roundtrip(String::from("a value"), Value::String("a value".into()));
        roundtrip(i64::MIN, Value::I64(i64::MIN));
        roundtrip(i64::MAX, Value::I64(i64::MAX));
        roundtrip(2.125_f64, Value::F64(2.125));
    }
}
