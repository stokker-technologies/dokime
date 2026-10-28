/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use super::{Core, Value, ValueError, ValueField, ValueFields, ValueItems, ValueKind, ValueNode};

pub(super) fn value(value: &Core) -> Result<Value, ValueError> {
    let count = node_count(value)?;
    let mut wire = Value { nodes: slots(count)? };
    push(&mut wire, value)?;
    Ok(wire)
}

pub(super) fn object(values: &BTreeMap<String, Core>) -> Result<Value, ValueError> {
    let mut count = 1_usize;
    for value in values.values() {
        count = count.checked_add(node_count(value)?).ok_or(ValueError::Capacity)?;
    }
    u32::try_from(count).map_err(|_| ValueError::Capacity)?;
    let mut wire = Value { nodes: slots(count)? };
    wire.nodes.push(ValueNode { kind: None });
    let fields = fields(&mut wire, values)?;
    wire.nodes[0].kind = Some(ValueKind::Object(ValueFields { fields }));
    Ok(wire)
}

pub(crate) fn slots<T>(count: usize) -> Result<Vec<T>, ValueError> {
    let mut target = Vec::new();
    target.try_reserve_exact(count).map_err(|_| ValueError::Allocation)?;
    Ok(target)
}

pub(crate) fn text(value: &str) -> Result<String, ValueError> {
    let mut target = String::new();
    target.try_reserve_exact(value.len()).map_err(|_| ValueError::Allocation)?;
    target.push_str(value);
    Ok(target)
}

fn node_count(value: &Core) -> Result<usize, ValueError> {
    match value {
        Core::Array(values) => count_children(values.iter()),
        Core::Object(values) => count_children(values.values()),
        _ => Ok(1),
    }
}

fn count_children<'a>(values: impl Iterator<Item = &'a Core>) -> Result<usize, ValueError> {
    let mut count = 1_usize;
    for value in values {
        let children = node_count(value)?;
        count = count.checked_add(children).ok_or(ValueError::Capacity)?;
    }
    u32::try_from(count).map_err(|_| ValueError::Capacity)?;
    Ok(count)
}

fn push(wire: &mut Value, value: &Core) -> Result<u32, ValueError> {
    let index = wire.nodes.len();
    let converted = u32::try_from(index).map_err(|_| ValueError::Capacity)?;
    wire.nodes.push(ValueNode { kind: None });
    let kind = match value {
        Core::Null => ValueKind::Null,
        Core::Bool(value) => ValueKind::Bool(*value),
        Core::Integer(value) => ValueKind::Integer(*value),
        Core::Unsigned(value) => ValueKind::Unsigned(*value),
        Core::Number(value) => ValueKind::Number(*value),
        Core::Text(value) => ValueKind::Text(text(value)?),
        Core::Datetime(value) => ValueKind::Datetime(super::Timestamp::from(*value)),
        Core::Array(values) => {
            let mut items = slots(values.len())?;
            for value in values {
                items.push(push(wire, value)?);
            }
            ValueKind::Array(ValueItems { items })
        }
        Core::Object(values) => ValueKind::Object(ValueFields { fields: fields(wire, values)? }),
    };
    wire.nodes[index].kind = Some(kind);
    Ok(converted)
}

fn fields(wire: &mut Value, values: &BTreeMap<String, Core>) -> Result<Vec<ValueField>, ValueError> {
    let mut fields = slots(values.len())?;
    for (key, value) in values {
        fields.push(ValueField {
            key: text(key)?,
            value: push(wire, value)?,
        });
    }
    Ok(fields)
}
