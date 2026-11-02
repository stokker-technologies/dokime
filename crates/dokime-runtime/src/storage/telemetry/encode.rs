/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reconstitute Harmos Envelopes From Typed Arrow Values

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::{
    message::postcard::Value as PostcardValue,
    schema::{
        Cardinality,
        graph::{Kind, Schema},
    },
};

// Local Imports
use crate::storage::{Error, Result};
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn payload(graph: &Schema, root: usize, values: &BTreeMap<String, Value>) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    message(graph, root, values, &mut bytes)?;
    Ok(bytes)
}

pub(super) fn restore_strings(values: &mut BTreeMap<String, Value>, originals: &[(Vec<String>, String)]) -> Result<()> {
    fn restore(value: &mut Value, path: &[String], text: &str) -> Result<()> {
        let Some((name, rest)) = path.split_first() else {
            *value = Value::Text(text.to_owned());
            return Ok(());
        };
        let child = match value {
            Value::Object(fields) => fields.get_mut(name),
            Value::Array(items) => name.parse::<usize>().ok().and_then(|index| items.get_mut(index)),
            _ => None,
        }
        .ok_or(Error::Corrupt("original telemetry string path is absent"))?;
        restore(child, rest, text)
    }
    for (path, text) in originals {
        let (name, rest) = path.split_first().ok_or(Error::Corrupt("original telemetry string path is empty"))?;
        let value = values.get_mut(name).ok_or(Error::Corrupt("original telemetry field is absent"))?;
        restore(value, rest, text)?;
    }
    Ok(())
}

/* === Private Functions === */

fn message(graph: &Schema, root: usize, values: &BTreeMap<String, Value>, bytes: &mut Vec<u8>) -> Result<()> {
    let fields = graph.definitions.get(root).ok_or(Error::Invalid("telemetry graph definition is absent"))?;
    let variant = fields.iter().any(|field| field.cardinality == Cardinality::Variant);
    for (index, field) in fields.iter().enumerate() {
        let value = values.get(field.name.as_ref()).unwrap_or(&Value::Null);
        if variant {
            if matches!(value, Value::Null) {
                continue;
            }
            (index as u32).write(bytes);
        }
        match field.cardinality {
            Cardinality::Optional => {
                (!matches!(value, Value::Null)).write(bytes);
                if matches!(value, Value::Null) {
                    continue;
                }
            }
            Cardinality::Repeated => {
                let Value::Array(items) = value else {
                    return Err(Error::Corrupt("Arrow repeated value is not an array"));
                };
                items.len().write(bytes);
                for item in items {
                    scalar(graph, field.kind, item, bytes)?;
                }
                continue;
            }
            _ => {}
        }
        scalar(graph, field.kind, value, bytes)?;
        if variant {
            return Ok(());
        }
    }
    Ok(())
}

fn scalar(graph: &Schema, kind: Kind, value: &Value, bytes: &mut Vec<u8>) -> Result<()> {
    let mismatch = || Error::Corrupt("Arrow value differs from its Harmos schema");
    match (kind, value) {
        (Kind::Bool, Value::Bool(value)) => value.write(bytes),
        (Kind::U32, Value::Unsigned(value)) => u32::try_from(*value).map_err(|_| mismatch())?.write(bytes),
        (Kind::U64, Value::Unsigned(value)) => value.write(bytes),
        (Kind::I32, Value::Integer(value)) => i32::try_from(*value).map_err(|_| mismatch())?.write(bytes),
        (Kind::I64, Value::Integer(value)) => value.write(bytes),
        (Kind::F32, Value::Number(value)) => (*value as f32).write(bytes),
        (Kind::F64, Value::Number(value)) => value.write(bytes),
        (Kind::String, Value::Text(value)) => value.write(bytes),
        (Kind::String, Value::Datetime(value)) => value.format(&time::format_description::well_known::Rfc3339)?.write(bytes),
        (Kind::Bytes, Value::Array(items)) => {
            items.len().write(bytes);
            for item in items {
                let byte = match item {
                    Value::Unsigned(value) => u8::try_from(*value).map_err(|_| mismatch())?,
                    Value::Integer(value) => u8::try_from(*value).map_err(|_| mismatch())?,
                    _ => return Err(mismatch()),
                };
                byte.write(bytes);
            }
        }
        (Kind::Message { definition }, Value::Object(fields)) => message(graph, definition, fields, bytes)?,
        _ => return Err(mismatch()),
    }
    Ok(())
}

/* === Tests === */
