/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Human and Structured Protocol Output

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost_reflect::ReflectMessage;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn value<T: ReflectMessage>(value: &T) -> crate::Result<serde_json::Value> {
    let mut message = prost_reflect::DynamicMessage::new(value.descriptor());
    message.transcode_from(value)?;
    let mut bytes = Vec::new();
    let mut serializer = serde_json::Serializer::new(&mut bytes);
    let options = prost_reflect::SerializeOptions::new().use_proto_field_name(true);
    message.serialize_with_options(&mut serializer, &options)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn print<T: ReflectMessage>(value: &T, json: bool) -> crate::Result<()> {
    let mut message = prost_reflect::DynamicMessage::new(value.descriptor());
    message.transcode_from(value)?;
    if json {
        let mut bytes = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut bytes);
        let options = prost_reflect::SerializeOptions::new().use_proto_field_name(true);
        message.serialize_with_options(&mut serializer, &options)?;
        println!("{}", String::from_utf8(bytes)?);
    } else {
        println!("{message:#}");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::value;
    use dokime::value::Kind;
    use dokime::{ExactNumber, Value};

    #[test]
    fn output_preserves_wide_integers_and_exact_rationals() -> crate::Result<()> {
        let integer = Value {
            kind: Some(Kind::UnsignedInteger(u64::MAX)),
        };
        let encoded = value(&integer)?;
        assert_eq!(encoded["unsigned_integer"], u64::MAX.to_string());

        let exact = ExactNumber {
            negative: true,
            numerator: "18446744073709551617".into(),
            denominator: "3".into(),
        };
        let rational = Value { kind: Some(Kind::Exact(exact.clone())) };
        let encoded = value(&rational)?;
        assert_eq!(encoded["exact"]["negative"], exact.negative);
        assert_eq!(encoded["exact"]["numerator"], exact.numerator);
        assert_eq!(encoded["exact"]["denominator"], exact.denominator);
        Ok(())
    }
}
