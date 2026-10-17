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
use std::{collections::BTreeMap, fmt, num::NonZeroUsize};

// Third Party Imports
use serde::{
    Serialize, Serializer,
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};

// Local Imports
use super::Value;
use crate::{runtime::Error, runtime::Result};

/* === Constants === */
const MAX_DEPTH: usize = 128;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug)]
struct EncodingError(String);
struct Budget {
    limit: usize,
    used: usize,
    build: bool,
}
struct Encoder<'a> {
    budget: &'a mut Budget,
    depth: usize,
}
struct Node {
    value: Option<Value>,
    string: bool,
}
struct Sequence<'a> {
    encoder: Encoder<'a>,
    expected: usize,
    seen: usize,
    values: Option<Vec<Value>>,
    tag: Option<String>,
}
struct Object<'a> {
    encoder: Encoder<'a>,
    expected: usize,
    seen: usize,
    values: Option<BTreeMap<String, Value>>,
    key: Option<String>,
    pending: bool,
    tag: Option<String>,
}
struct Count(usize);
struct Text {
    value: String,
    limit: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl fmt::Display for EncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for EncodingError {}
impl serde::ser::Error for EncodingError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}
impl Budget {
    fn charge(&mut self, bytes: usize) -> std::result::Result<(), EncodingError> {
        let total = self
            .used
            .checked_add(bytes)
            .ok_or_else(|| error("report value memory overflows"))?;
        if total > self.limit {
            return Err(error("report value exceeds original allowance"));
        }
        self.used = total;
        Ok(())
    }
}
impl Encoder<'_> {
    fn scalar(self, make: impl FnOnce() -> Value) -> std::result::Result<Node, EncodingError> {
        Ok(Node {
            value: self.budget.build.then(make),
            string: false,
        })
    }
    fn text(self, value: &str) -> std::result::Result<Node, EncodingError> {
        self.budget.charge(value.len())?;
        let value = if self.budget.build {
            Some(Value::String(copy_text(value)?))
        } else {
            None
        };
        Ok(Node {
            value,
            string: true,
        })
    }
    fn enter(&self) -> std::result::Result<(), EncodingError> {
        if self.depth > MAX_DEPTH {
            return Err(error("report value nesting exceeds limit"));
        }
        Ok(())
    }
}
impl<'a> Serializer for Encoder<'a> {
    type Ok = Node;
    type Error = EncodingError;
    type SerializeSeq = Sequence<'a>;
    type SerializeTuple = Sequence<'a>;
    type SerializeTupleStruct = Sequence<'a>;
    type SerializeTupleVariant = Sequence<'a>;
    type SerializeMap = Object<'a>;
    type SerializeStruct = Object<'a>;
    type SerializeStructVariant = Object<'a>;
    fn serialize_bool(self, value: bool) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Bool(value))
    }
    fn serialize_i8(self, value: i8) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Integer(i64::from(value)))
    }
    fn serialize_i16(self, value: i16) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Integer(i64::from(value)))
    }
    fn serialize_i32(self, value: i32) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Integer(i64::from(value)))
    }
    fn serialize_i64(self, value: i64) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Integer(value))
    }
    fn serialize_u8(self, value: u8) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Unsigned(u64::from(value)))
    }
    fn serialize_u16(self, value: u16) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Unsigned(u64::from(value)))
    }
    fn serialize_u32(self, value: u32) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Unsigned(u64::from(value)))
    }
    fn serialize_u64(self, value: u64) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Unsigned(value))
    }

    fn serialize_i128(self, value: i128) -> std::result::Result<Node, EncodingError> {
        self.serialize_i64(
            value
                .try_into()
                .map_err(|_| error("report signed scalar exceeds native range"))?,
        )
    }
    fn serialize_u128(self, value: u128) -> std::result::Result<Node, EncodingError> {
        self.serialize_u64(
            value
                .try_into()
                .map_err(|_| error("report unsigned scalar exceeds native range"))?,
        )
    }
    fn serialize_f32(self, value: f32) -> std::result::Result<Node, EncodingError> {
        self.serialize_f64(f64::from(value))
    }
    fn serialize_f64(self, value: f64) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        if !value.is_finite() {
            return Err(error("report number must be finite"));
        }
        self.scalar(|| Value::Number(value))
    }
    fn serialize_char(self, value: char) -> std::result::Result<Node, EncodingError> {
        self.serialize_str(value.encode_utf8(&mut [0; 4]))
    }
    fn serialize_str(self, value: &str) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.text(value)
    }
    fn serialize_bytes(self, values: &[u8]) -> std::result::Result<Node, EncodingError> {
        let mut sequence = self.serialize_seq(Some(values.len()))?;
        for value in values {
            SerializeSeq::serialize_element(&mut sequence, value)?;
        }
        SerializeSeq::end(sequence)
    }
    fn serialize_none(self) -> std::result::Result<Node, EncodingError> {
        self.serialize_unit()
    }
    fn serialize_some<T: ?Sized + Serialize>(
        self,
        value: &T,
    ) -> std::result::Result<Node, EncodingError> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.scalar(|| Value::Null)
    }
    fn serialize_unit_struct(self, _: &'static str) -> std::result::Result<Node, EncodingError> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> std::result::Result<Node, EncodingError> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        value: &T,
    ) -> std::result::Result<Node, EncodingError> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        self.budget.charge(object_heap(1))?;
        self.budget.charge(variant.len())?;
        let build = self.budget.build;
        let node = value.serialize(Encoder {
            budget: self.budget,
            depth: self.depth + 1,
        })?;
        Ok(Node {
            value: if build {
                Some(Value::Object(BTreeMap::from([(
                    copy_text(variant)?,
                    node.value.unwrap(),
                )])))
            } else {
                None
            },
            string: false,
        })
    }
    fn serialize_seq(self, len: Option<usize>) -> std::result::Result<Sequence<'a>, EncodingError> {
        self.enter()?;
        let expected =
            len.ok_or_else(|| error("report sequence requires its exact field count"))?;
        self.budget.charge(
            expected
                .checked_mul(std::mem::size_of::<Value>())
                .ok_or_else(|| error("report sequence slots overflow"))?,
        )?;
        let values = if self.budget.build {
            Some(slots(expected)?)
        } else {
            None
        };
        Ok(Sequence {
            encoder: self,
            expected,
            seen: 0,
            values,
            tag: None,
        })
    }
    fn serialize_tuple(self, len: usize) -> std::result::Result<Sequence<'a>, EncodingError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> std::result::Result<Sequence<'a>, EncodingError> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        mut self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Sequence<'a>, EncodingError> {
        self.budget.charge(object_heap(1))?;
        self.budget.charge(variant.len())?;
        let tag = if self.budget.build {
            Some(copy_text(variant)?)
        } else {
            None
        };
        self.depth += 1;
        let mut sequence = self.serialize_seq(Some(len))?;
        sequence.tag = tag;
        Ok(sequence)
    }
    fn serialize_map(self, len: Option<usize>) -> std::result::Result<Object<'a>, EncodingError> {
        self.enter()?;
        let expected = len.ok_or_else(|| error("report object requires its exact field count"))?;
        self.budget.charge(object_heap(expected))?;
        let values = self.budget.build.then(BTreeMap::new);
        Ok(Object {
            encoder: self,
            expected,
            seen: 0,
            values,
            key: None,
            pending: false,
            tag: None,
        })
    }
    fn serialize_struct(
        self,
        _: &'static str,
        len: usize,
    ) -> std::result::Result<Object<'a>, EncodingError> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(
        mut self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> std::result::Result<Object<'a>, EncodingError> {
        self.budget.charge(object_heap(1))?;
        self.budget.charge(variant.len())?;
        let tag = if self.budget.build {
            Some(copy_text(variant)?)
        } else {
            None
        };
        self.depth += 1;
        let mut object = self.serialize_map(Some(len))?;
        object.tag = tag;
        Ok(object)
    }
    fn collect_str<T: ?Sized + fmt::Display>(
        self,
        value: &T,
    ) -> std::result::Result<Node, EncodingError> {
        self.enter()?;
        let mut count = Count(0);
        fmt::write(&mut count, format_args!("{value}"))
            .map_err(|_| error("report display length failed"))?;
        self.budget.charge(count.0)?;
        let text = if self.budget.build {
            let mut text = String::new();
            text.try_reserve_exact(count.0)
                .map_err(|_| error("report display allocation failed"))?;
            if text.capacity() > count.0 {
                return Err(error("report display capacity exceeds admission"));
            }
            let mut text = Text {
                value: text,
                limit: count.0,
            };
            fmt::write(&mut text, format_args!("{value}"))
                .map_err(|_| error("report display changed beyond allowance"))?;
            Some(Value::String(text.value))
        } else {
            None
        };
        Ok(Node {
            value: text,
            string: true,
        })
    }
}
impl Sequence<'_> {
    fn element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        if self.seen == self.expected {
            return Err(error("report sequence exceeds declared count"));
        }
        let node = value.serialize(Encoder {
            budget: self.encoder.budget,
            depth: self.encoder.depth + 1,
        })?;
        if let Some(values) = &mut self.values {
            values.push(node.value.unwrap());
        }
        self.seen += 1;
        Ok(())
    }
    fn finish(self) -> std::result::Result<Node, EncodingError> {
        if self.seen != self.expected {
            return Err(error("report sequence differs from declared count"));
        }
        let value = self
            .values
            .map(|values| tagged(Value::Array(values), self.tag));
        Ok(Node {
            value,
            string: false,
        })
    }
}
impl Object<'_> {
    fn key<T: ?Sized + Serialize>(&mut self, key: &T) -> std::result::Result<(), EncodingError> {
        if self.pending || self.seen == self.expected {
            return Err(error("report object key count differs"));
        }
        let node = key.serialize(Encoder {
            budget: self.encoder.budget,
            depth: self.encoder.depth + 1,
        })?;
        if !node.string {
            return Err(error("report object keys must be strings"));
        }
        if let Some(value) = node.value {
            let Value::String(key) = value else {
                unreachable!()
            };
            self.key = Some(key);
        }
        self.pending = true;
        Ok(())
    }
    fn value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        if !self.pending {
            return Err(error("report object value has no key"));
        }
        let node = value.serialize(Encoder {
            budget: self.encoder.budget,
            depth: self.encoder.depth + 1,
        })?;
        if let Some(values) = &mut self.values
            && values
                .insert(self.key.take().unwrap(), node.value.unwrap())
                .is_some()
        {
            return Err(error("report object key is duplicated"));
        }
        self.pending = false;
        self.seen += 1;
        Ok(())
    }
    fn field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.key(key)?;
        self.value(value)
    }
    fn finish(self) -> std::result::Result<Node, EncodingError> {
        if self.pending || self.seen != self.expected {
            return Err(error("report object differs from declared field count"));
        }
        let value = self
            .values
            .map(|values| tagged(Value::Object(values), self.tag));
        Ok(Node {
            value,
            string: false,
        })
    }
}
impl SerializeSeq for Sequence<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.element(value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}
impl SerializeTuple for Sequence<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_element<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.element(value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}
impl SerializeTupleStruct for Sequence<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.element(value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}
impl SerializeTupleVariant for Sequence<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.element(value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}

impl SerializeMap for Object<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_key<T: ?Sized + Serialize>(
        &mut self,
        key: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.key(key)
    }
    fn serialize_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.value(value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}
impl SerializeStruct for Object<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.field(key, value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}
impl SerializeStructVariant for Object<'_> {
    type Ok = Node;
    type Error = EncodingError;
    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), EncodingError> {
        self.field(key, value)
    }
    fn end(self) -> std::result::Result<Node, EncodingError> {
        self.finish()
    }
}

impl fmt::Write for Count {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.checked_add(text.len()).ok_or(fmt::Error)?;
        Ok(())
    }
}
impl fmt::Write for Text {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self
            .value
            .len()
            .checked_add(text.len())
            .is_none_or(|total| total > self.limit)
        {
            return Err(fmt::Error);
        }
        self.value.push_str(text);
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn memory<T: ?Sized + Serialize>(value: &T) -> Result<usize> {
    let mut budget = Budget {
        limit: usize::MAX,
        used: std::mem::size_of::<Value>(),
        build: false,
    };
    value
        .serialize(Encoder {
            budget: &mut budget,
            depth: 0,
        })
        .map_err(|error| Error::InvalidReportInputs(error.0))?;
    Ok(budget.used)
}
pub(super) fn build<T: ?Sized + Serialize>(value: &T, allowance: NonZeroUsize) -> Result<Value> {
    let bytes = memory(value)?;
    if bytes == usize::MAX || bytes > allowance.get() {
        return Err(Error::InvalidReportInputs(
            "report value exceeds original allowance".into(),
        ));
    }
    let mut budget = Budget {
        limit: allowance.get(),
        used: 0,
        build: true,
    };
    budget
        .charge(std::mem::size_of::<Value>())
        .map_err(|error| Error::InvalidReportInputs(error.0))?;
    let node = value
        .serialize(Encoder {
            budget: &mut budget,
            depth: 0,
        })
        .map_err(|error| Error::InvalidReportInputs(error.0))?;
    Ok(node.value.expect("building owned values"))
}

/* === Private Functions === */
fn error(message: &str) -> EncodingError {
    EncodingError(message.into())
}
fn object_heap(count: usize) -> usize {
    let bytes = harmos::btree_map_memory::<String, Value>(count.max(1));
    if bytes == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_sub(std::mem::size_of::<BTreeMap<String, Value>>())
    }
}
fn slots(count: usize) -> std::result::Result<Vec<Value>, EncodingError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| error("report value slots allocation failed"))?;
    if values.capacity() > count {
        return Err(error("report value slots exceed admission"));
    }
    Ok(values)
}
fn copy_text(source: &str) -> std::result::Result<String, EncodingError> {
    let mut text = String::new();
    text.try_reserve_exact(source.len())
        .map_err(|_| error("report text allocation failed"))?;
    if text.capacity() > source.len() {
        return Err(error("report text capacity exceeds admission"));
    }
    text.push_str(source);
    Ok(text)
}
fn tagged(value: Value, tag: Option<String>) -> Value {
    match tag {
        Some(tag) => Value::Object(BTreeMap::from([(tag, value)])),
        None => value,
    }
}

/* === Tests === */
