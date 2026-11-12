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
