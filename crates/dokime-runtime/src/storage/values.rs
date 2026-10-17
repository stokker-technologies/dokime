/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod map {
    use super::*;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        cells: &[Cell],
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let value = decode(cells).map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Vec<Cell>, D::Error> {
        let value = BTreeMap::<String, Value>::deserialize(deserializer)?;
        encode(&value).map_err(serde::de::Error::custom)
    }
}

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports

// Local Imports
use super::{Error, Result};
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */
const MAX_DEPTH: usize = 64;
const MAX_CELLS: usize = 131072;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Cell {
    pub name: Option<String>,
    pub kind: String,
    pub children: u32,
    pub boolean: Option<bool>,
    pub integer: Option<i64>,
    pub unsigned: Option<u64>,
    pub number: Option<f64>,
    pub text: Option<String>,
}

struct Encoding {
    cells: usize,
    strings: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn encoding_memory(details: &BTreeMap<String, Value>) -> Result<usize> {
    let plan = encoding(details)?;
    Ok(std::mem::size_of::<Vec<Cell>>()
        .saturating_add(plan.cells.saturating_mul(std::mem::size_of::<Cell>()))
        .saturating_add(plan.strings))
}

pub(crate) fn encode(details: &BTreeMap<String, Value>) -> Result<Vec<Cell>> {
    let plan = encoding(details)?;
    let mut cells = Vec::new();
    cells
        .try_reserve_exact(plan.cells)
        .map_err(|_| error("stored cell allocation failed"))?;
    cells.push(Cell {
        kind: "object".into(),
        children: details
            .len()
            .try_into()
            .map_err(|_| error("stored value child count exceeds limit"))?,
        ..Default::default()
    });
    for (name, value) in details {
        append(value, Some(name.clone()), 1, &mut cells)?;
    }
    Ok(cells)
}

pub(crate) fn value_encoding_memory(value: &Value) -> Result<usize> {
    let plan = value_encoding(value)?;
    Ok(std::mem::size_of::<Vec<Cell>>()
        .saturating_add(plan.cells.saturating_mul(std::mem::size_of::<Cell>()))
        .saturating_add(plan.strings))
}

pub(crate) fn encode_value(value: &Value) -> Result<Vec<Cell>> {
    let plan = value_encoding(value)?;
    let mut cells = Vec::new();
    cells
        .try_reserve_exact(plan.cells)
        .map_err(|_| error("stored scalar allocation failed"))?;
    cells.push(Cell {
        kind: "object".into(),
        children: 1,
        ..Default::default()
    });
    append(value, Some("value".into()), 1, &mut cells)?;
    Ok(cells)
}

pub(crate) fn datetime_memory(value: time::OffsetDateTime) -> Result<usize> {
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("datetime output size overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0);
    value
        .to_offset(time::UtcOffset::UTC)
        .format_into(&mut count, &time::format_description::well_known::Rfc3339)?;
    Ok(count.0)
}

pub(crate) fn encode_datetime(value: time::OffsetDateTime) -> Result<String> {
    let length = datetime_memory(value)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| error("stored datetime allocation failed"))?;
    value
        .to_offset(time::UtcOffset::UTC)
        .format_into(&mut bytes, &time::format_description::well_known::Rfc3339)?;
    String::from_utf8(bytes).map_err(|_| error("stored datetime formatter produced non-UTF8 text"))
}

pub(crate) fn decode(cells: &[Cell]) -> Result<BTreeMap<String, Value>> {
    if cells.len() > MAX_CELLS {
        return Err(error("stored values exceed cell limit"));
    }
    let mut at = 0;
    let Value::Object(details) = read(cells, &mut at, 0)? else {
        return Err(error("stored values root must be an object"));
    };
    if at != cells.len() || encode(&details)? != cells {
        return Err(error("stored values contain noncanonical cells"));
    }
    Ok(details)
}

pub(crate) fn owned_memory(cells: &[Cell]) -> Result<usize> {
    if cells.len() > MAX_CELLS || cells.first().is_none_or(|cell| cell.kind != "object") {
        return Err(error("stored values require a bounded object root"));
    }
    let mut at = 0;
    let bytes = measure(cells, &mut at, 0, false)?;
    if at != cells.len() {
        return Err(error("stored values contain trailing cells"));
    }
    if bytes == usize::MAX {
        return Ok(usize::MAX);
    }
    Ok(bytes
        .saturating_sub(std::mem::size_of::<Value>())
        .saturating_add(std::mem::size_of::<BTreeMap<String, Value>>()))
}

pub(crate) fn decode_owned(
    cells: Vec<Cell>,
    allowance: NonZeroUsize,
) -> Result<BTreeMap<String, Value>> {
    let bound = owned_memory(&cells)?;
    if bound == usize::MAX || bound > allowance.get() {
        return Err(error("stored value projection exceeds memory allowance"));
    }
    let mut cells = cells.into_iter();
    let Value::Object(value) = consume(&mut cells)? else {
        return Err(error("stored values root must be an object"));
    };
    Ok(value)
}

/* === Private Functions === */
fn encoding(details: &BTreeMap<String, Value>) -> Result<Encoding> {
    let mut plan = Encoding {
        cells: 1,
        strings: "object".len(),
    };
    u32::try_from(details.len()).map_err(|_| error("stored value child count exceeds limit"))?;
    for (name, value) in details {
        measure_encoding(value, Some(name), 1, &mut plan)?;
    }
    Ok(plan)
}

fn value_encoding(value: &Value) -> Result<Encoding> {
    let mut plan = Encoding {
        cells: 1,
        strings: "object".len(),
    };
    measure_encoding(value, Some("value"), 1, &mut plan)?;
    Ok(plan)
}

fn measure_encoding(
    value: &Value,
    name: Option<&str>,
    depth: usize,
    plan: &mut Encoding,
) -> Result<()> {
    if depth > MAX_DEPTH || plan.cells == MAX_CELLS {
        return Err(error("stored values exceed nesting or cell limit"));
    }
    plan.cells += 1;
    let (kind, text) = match value {
        Value::Null => ("null", 0),
        Value::Bool(_) => ("bool", 0),
        Value::Integer(_) => ("integer", 0),
        Value::Unsigned(_) => ("unsigned", 0),
        Value::Number(value) if value.is_finite() => ("number", 0),
        Value::Number(_) => return Err(error("stored value number must be finite")),
        Value::Text(text) => ("text", text.len()),
        Value::Datetime(value) => ("datetime", datetime_memory(*value)?),
        Value::Array(values) => {
            u32::try_from(values.len())
                .map_err(|_| error("stored value child count exceeds limit"))?;
            ("array", 0)
        }
        Value::Object(values) => {
            u32::try_from(values.len())
                .map_err(|_| error("stored value child count exceeds limit"))?;
            ("object", 0)
        }
    };
    plan.strings = plan
        .strings
        .saturating_add(kind.len())
        .saturating_add(name.map_or(0, str::len))
        .saturating_add(text);
    match value {
        Value::Array(values) => {
            for value in values {
                measure_encoding(value, None, depth + 1, plan)?;
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                measure_encoding(value, Some(name), depth + 1, plan)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn measure(cells: &[Cell], at: &mut usize, depth: usize, named: bool) -> Result<usize> {
    if depth > MAX_DEPTH {
        return Err(error("stored values exceed nesting limit"));
    }
    let cell = cells
        .get(*at)
        .ok_or_else(|| error("stored value child is missing"))?;
    *at += 1;
    if cell.name.is_some() != named {
        return Err(error("stored value name is noncanonical"));
    }
    let scalar_count = usize::from(cell.boolean.is_some())
        + usize::from(cell.integer.is_some())
        + usize::from(cell.unsigned.is_some())
        + usize::from(cell.number.is_some())
        + usize::from(cell.text.is_some());
    let valid = match cell.kind.as_str() {
        "null" => scalar_count == 0 && cell.children == 0,
        "bool" => scalar_count == 1 && cell.boolean.is_some() && cell.children == 0,
        "integer" => scalar_count == 1 && cell.integer.is_some() && cell.children == 0,
        "unsigned" => scalar_count == 1 && cell.unsigned.is_some() && cell.children == 0,
        "number" => {
            scalar_count == 1 && cell.number.is_some_and(f64::is_finite) && cell.children == 0
        }
        "text" | "datetime" => scalar_count == 1 && cell.text.is_some() && cell.children == 0,
        "array" | "object" => {
            scalar_count == 0 && cell.children as usize <= cells.len().saturating_sub(*at)
        }
        _ => false,
    };
    if !valid {
        return Err(error("stored value scalar shape is noncanonical"));
    }
    let inline = std::mem::size_of::<Value>();
    let allocation = match cell.kind.as_str() {
        "text" => cell.text.as_ref().expect("validated text").capacity(),
        "datetime" => {
            parse_datetime(cell.text.as_deref().expect("validated datetime"))?;
            0
        }
        "array" | "object" => {
            let object = cell.kind == "object";
            let mut bytes = if object {
                harmos::btree_map_memory::<String, Value>(cell.children as usize)
                    .saturating_sub(std::mem::size_of::<BTreeMap<String, Value>>())
            } else {
                (cell.children as usize).saturating_mul(inline)
            };
            let mut previous: Option<&str> = None;
            for _ in 0..cell.children {
                if object {
                    let name = cells
                        .get(*at)
                        .and_then(|child| child.name.as_ref())
                        .ok_or_else(|| error("object child name is missing"))?;
                    if previous.is_some_and(|previous| previous >= name.as_str()) {
                        return Err(error("stored object names must be strictly ordered"));
                    }
                    previous = Some(name);
                    bytes = bytes.saturating_add(name.capacity());
                }
                let child = measure(cells, at, depth + 1, object)?;
                bytes = if child == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(child.saturating_sub(inline))
                };
            }
            bytes
        }
        _ => 0,
    };
    Ok(inline.saturating_add(allocation))
}

fn consume(cells: &mut std::vec::IntoIter<Cell>) -> Result<Value> {
    let cell = cells
        .next()
        .ok_or_else(|| error("stored value child is missing"))?;
    Ok(match cell.kind.as_str() {
        "null" => Value::Null,
        "bool" => Value::Bool(cell.boolean.expect("validated bool")),
        "integer" => Value::Integer(cell.integer.expect("validated integer")),
        "unsigned" => Value::Unsigned(cell.unsigned.expect("validated unsigned")),
        "number" => Value::Number(cell.number.expect("validated number")),
        "text" => Value::Text(cell.text.expect("validated text")),
        "datetime" => Value::Datetime(parse_datetime(
            cell.text.as_deref().expect("validated datetime"),
        )?),
        "array" => {
            let mut values = Vec::new();
            values
                .try_reserve_exact(cell.children as usize)
                .map_err(|_| error("stored array allocation failed"))?;
            for _ in 0..cell.children {
                values.push(consume(cells)?);
            }
            Value::Array(values)
        }
        "object" => {
            let mut values = BTreeMap::new();
            for _ in 0..cell.children {
                let name = cells
                    .as_mut_slice()
                    .first_mut()
                    .and_then(|child| child.name.take())
                    .ok_or_else(|| error("object child name is missing"))?;
                values.insert(name, consume(cells)?);
            }
            Value::Object(values)
        }
        _ => return Err(error("unknown stored value kind")),
    })
}

fn parse_datetime(text: &str) -> Result<time::OffsetDateTime> {
    struct Compare<'a> {
        remaining: &'a [u8],
    }
    impl std::io::Write for Compare<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if !self.remaining.starts_with(bytes) {
                return Err(std::io::Error::other("noncanonical datetime"));
            }
            self.remaining = &self.remaining[bytes.len()..];
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let format = time::format_description::well_known::Rfc3339;
    let datetime = time::OffsetDateTime::parse(text, &format)?.to_offset(time::UtcOffset::UTC);
    let mut compare = Compare {
        remaining: text.as_bytes(),
    };
    datetime
        .format_into(&mut compare, &format)
        .map_err(|_| error("stored datetime is noncanonical"))?;
    if !compare.remaining.is_empty() {
        return Err(error("stored datetime is noncanonical"));
    }
    Ok(datetime)
}

fn error(reason: &'static str) -> Error {
    Error::Invalid(reason)
}

fn append(value: &Value, name: Option<String>, depth: usize, cells: &mut Vec<Cell>) -> Result<()> {
    if depth > MAX_DEPTH || cells.len() == MAX_CELLS {
        return Err(error("stored values exceed nesting or cell limit"));
    }
    let mut cell = Cell {
        name,
        ..Default::default()
    };
    cell.kind = match value {
        Value::Null => "null",
        Value::Bool(value) => {
            cell.boolean = Some(*value);
            "bool"
        }
        Value::Integer(value) => {
            cell.integer = Some(*value);
            "integer"
        }
        Value::Unsigned(value) => {
            cell.unsigned = Some(*value);
            "unsigned"
        }
        Value::Number(value) if value.is_finite() => {
            cell.number = Some(*value);
            "number"
        }
        Value::Number(_) => return Err(error("stored value number must be finite")),
        Value::Text(value) => {
            cell.text = Some(value.clone());
            "text"
        }
        Value::Datetime(value) => {
            cell.text = Some(encode_datetime(*value)?);
            "datetime"
        }
        Value::Array(values) => {
            cell.children = values
                .len()
                .try_into()
                .map_err(|_| error("stored value child count exceeds limit"))?;
            "array"
        }
        Value::Object(values) => {
            cell.children = values
                .len()
                .try_into()
                .map_err(|_| error("stored value child count exceeds limit"))?;
            "object"
        }
    }
    .into();
    cells.push(cell);

    match value {
        Value::Array(values) => {
            for value in values {
                append(value, None, depth + 1, cells)?;
            }
        }
        Value::Object(values) => {
            for (name, value) in values {
                append(value, Some(name.clone()), depth + 1, cells)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn read(cells: &[Cell], at: &mut usize, depth: usize) -> Result<Value> {
    if depth > MAX_DEPTH {
        return Err(error("stored values exceed nesting limit"));
    }
    let cell = cells
        .get(*at)
        .ok_or_else(|| error("stored value child is missing"))?;
    *at += 1;
    let missing = || error("stored value scalar is missing");
    Ok(match cell.kind.as_str() {
        "null" => Value::Null,
        "bool" => Value::Bool(cell.boolean.ok_or_else(missing)?),
        "integer" => Value::Integer(cell.integer.ok_or_else(missing)?),
        "unsigned" => Value::Unsigned(cell.unsigned.ok_or_else(missing)?),
        "number" => Value::Number(cell.number.ok_or_else(missing)?),
        "text" => Value::Text(cell.text.clone().ok_or_else(missing)?),
        "datetime" => {
            let timestamp = Timestamp::new(cell.text.clone().ok_or_else(missing)?);
            Value::Datetime(timestamp.to_datetime()?)
        }
        "array" | "object" => {
            if cell.children as usize > cells.len().saturating_sub(*at) {
                return Err(error("stored value children exceed remaining cells"));
            }
            if cell.kind == "array" {
                let mut values = Vec::new();
                for _ in 0..cell.children {
                    values.push(read(cells, at, depth + 1)?);
                }
                Value::Array(values)
            } else {
                let mut values = BTreeMap::new();
                for _ in 0..cell.children {
                    let name = cells
                        .get(*at)
                        .and_then(|cell| cell.name.clone())
                        .ok_or_else(|| error("object child name is missing"))?;
                    if values.insert(name, read(cells, at, depth + 1)?).is_some() {
                        return Err(error("duplicate stored value name"));
                    }
                }
                Value::Object(values)
            }
        }
        _ => return Err(error("unknown stored value kind")),
    })
}

/* === Tests === */
