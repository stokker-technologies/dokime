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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};
// Third Party Imports
use arrow_array::{
    ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array, StringArray, StructArray, UInt32Array, UInt64Array,
};
use arrow_schema::Field;
// Local Imports
use dokime_domain::model::values::Value;
use harmos::schema::{
    Cardinality,
    graph::{Kind, Schema},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn capture_payload(schema: &Schema, values: &BTreeMap<String, Value>) -> Result<Vec<u8>, String> {
    schema.preparation(&harmos::config::SchemaSettings::default()).map_err(|error| error.to_string())?;
    let fields = schema.definitions.get(schema.root).ok_or("matcher schema root is absent")?;
    if values.keys().any(|name| !fields.iter().any(|field| field.name == name.as_str())) {
        return Err("matcher values contain an undeclared field".into());
    }
    let mut columns = Vec::with_capacity(fields.len());
    for field in fields {
        let value = values.get(field.name.as_ref());
        let invalid = || format!("matcher field {} does not match its declared scalar type", field.name);
        if matches!(field.cardinality, Cardinality::Repeated | Cardinality::Variant)
            || (field.cardinality == Cardinality::Required && value.is_none())
        {
            return Err(invalid());
        }
        let array: ArrayRef = match field.kind {
            Kind::Bool => Arc::new(BooleanArray::from(vec![match value {
                Some(Value::Bool(value)) => Some(*value),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::I32 => Arc::new(Int32Array::from(vec![match value {
                Some(Value::Integer(value)) => Some(i32::try_from(*value).map_err(|_| invalid())?),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::I64 => Arc::new(Int64Array::from(vec![match value {
                Some(Value::Integer(value)) => Some(*value),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::U32 => Arc::new(UInt32Array::from(vec![match value {
                Some(Value::Integer(value)) => Some(u32::try_from(*value).map_err(|_| invalid())?),
                Some(Value::Unsigned(value)) => Some(u32::try_from(*value).map_err(|_| invalid())?),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::U64 => Arc::new(UInt64Array::from(vec![match value {
                Some(Value::Integer(value)) => Some(u64::try_from(*value).map_err(|_| invalid())?),
                Some(Value::Unsigned(value)) => Some(*value),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::F32 => Arc::new(Float32Array::from(vec![match value {
                Some(Value::Number(value)) if value.is_finite() && (*value as f32).is_finite() => Some(*value as f32),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::F64 => Arc::new(Float64Array::from(vec![match value {
                Some(Value::Number(value)) if value.is_finite() => Some(*value),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::String => Arc::new(StringArray::from(vec![match value {
                Some(Value::Text(value)) => Some(value.as_str()),
                None => None,
                _ => return Err(invalid()),
            }])),
            Kind::Bytes | Kind::Message { .. } => return Err(invalid()),
        };
        columns.push((
            Arc::new(Field::new(field.name.as_ref(), array.data_type().clone(), field.cardinality == Cardinality::Optional)),
            array,
        ));
    }
    let payload = StructArray::from(columns);
    let allowance = NonZeroUsize::new((64usize * 1024).saturating_mul(2)).expect("line captures have a nonzero bound");
    let mut rows = harmos::storage::arrow::encode_payloads(schema, &payload, harmos::storage::PayloadEncoding::PostcardV1, allowance)
        .map_err(|error| error.to_string())?;
    rows.pop().ok_or_else(|| "matcher encoded no row".into())
}

/* === Private Functions === */

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;
    use harmos::{Message, schema::Described};

    #[harmos::message]
    struct Reading {
        counter: u64,
        voltage: f64,
        state: Option<String>,
    }

    #[test]
    fn captures_preserve_the_declared_codec_and_unsigned_range() {
        let graph = Schema::from_declaration(Reading::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap();
        let row = Reading { counter: u64::MAX, voltage: 1.5, state: None };
        let values = BTreeMap::from([("counter".to_owned(), Value::Unsigned(u64::MAX)), ("voltage".to_owned(), Value::Number(1.5))]);
        assert_eq!(capture_payload(&graph, &values).unwrap(), row.encode_to_vec());
        let mut unknown = values.clone();
        unknown.insert("undeclared".to_owned(), Value::Bool(true));
        assert!(capture_payload(&graph, &unknown).is_err());
        let mut invalid = values;
        invalid.insert("counter".to_owned(), Value::Integer(-1));
        assert!(capture_payload(&graph, &invalid).is_err());
    }
}
