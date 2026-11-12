/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Value};

// Local Imports
use crate::{Error, Resources, Result};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.statistics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::StatisticsRequest, v1::Statistics>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::StatisticsRequest) -> Result<v1::Statistics> {
    resources.check_scope(&request.execution)?;
    let snapshot = resources.store.read(&resources.execution)?;
    let catalog = h::Catalog::decode(snapshot.execution.catalog.as_slice())?;
    let stream = catalog
        .streams
        .iter()
        .find(|stream| stream.id == request.stream)
        .ok_or_else(|| Error::Invalid(format!("Select a Declared Stream: {}", request.stream)))?;
    let pool = DescriptorPool::from_file_descriptor_set(
        catalog.types.ok_or_else(|| Error::Invalid("Reconnect the Plugin with Its Message Descriptors".into()))?,
    )
    .map_err(|error| Error::Invalid(error.to_string()))?;
    let descriptor = pool
        .get_message_by_name(&stream.row)
        .ok_or_else(|| Error::Invalid("Reconnect the Plugin with Its Stream Row Descriptor".into()))?;
    let field = if request.field.is_empty() && descriptor.fields().len() == 1 {
        descriptor.fields().next()
    } else {
        descriptor.get_field_by_name(&request.field)
    }
    .ok_or_else(|| Error::Invalid("Select a Numeric Stream Column".into()))?;
    let mut values = Vec::new();
    for sample in &snapshot.samples {
        if sample.stream != request.stream || !snapshot.step_contains(&request.step, sample.position) {
            continue;
        }
        let row = DynamicMessage::decode(descriptor.clone(), sample.row.as_slice())?;
        let value = match row.get_field(&field).as_ref() {
            Value::F64(value) => *value,
            Value::F32(value) => *value as f64,
            Value::I64(value) => *value as f64,
            Value::U64(value) => *value as f64,
            Value::I32(value) => *value as f64,
            Value::U32(value) => *value as f64,
            _ => return Err(Error::Invalid("Select a Numeric Stream Column".into())),
        };
        if !value.is_finite() {
            return Err(Error::Invalid("Publish Finite Values Before Computing Statistics".into()));
        }
        values.push(value);
    }
    resources.evidence("read:statistics", request.encode_to_vec(), None)?;
    if values.is_empty() {
        return Ok(v1::Statistics::default());
    }
    let first = values[0];
    let last = values[values.len() - 1];
    let count = values.len() as u64;
    let mean = values.iter().sum::<f64>() / count as f64;
    let stddev = (values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / count as f64).sqrt();
    values.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        let index = p * (values.len() - 1) as f64;
        let low = index.floor() as usize;
        let high = index.ceil() as usize;
        values[low] + (values[high] - values[low]) * index.fract()
    };
    Ok(v1::Statistics {
        count,
        min: values[0],
        max: values[values.len() - 1],
        mean,
        stddev,
        median: percentile(0.5),
        p05: percentile(0.05),
        p95: percentile(0.95),
        p99: percentile(0.99),
        first,
        last,
    })
}

/* === Private Functions === */

/* === Tests === */
