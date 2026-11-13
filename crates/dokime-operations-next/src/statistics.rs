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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, Value};

// Local Imports
use crate::{Error, Resources, Result};
use dokime_storage::StatisticsRead;

/* === Constants === */

pub const ID: &str = "dokime.context.v1.Evidence/Statistics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::EvidenceStatisticsRequest, dokime_protocol::context::EvidenceStatisticsResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::EvidenceStatisticsRequest,
) -> Result<dokime_protocol::context::EvidenceStatisticsResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let mut snapshot = resources.store.cut(&resources.execution)?;
    let resolved_at=resources.store.resource_value(&resources.execution,"progress",request.at.as_ref())?.resolved_at;
    snapshot.frontier=resolved_at.sequence;
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
    let page_size = NonZeroUsize::new(256).unwrap();
    let (mut after, through) = resources.store.step_range(&resources.execution, &request.step, snapshot.frontier, page_size)?;
    let start = after;
    let frontier = resources.store.sample_frontier(&resources.execution, &request.stream, after, through)?;
    let mut key = request.clone();
    key.field = field.name().into();
    if let Some(mut read) = resources.store.cached_statistics(&resources.execution, &key, start, frontier)? {
        read.request = Some(request);
        let mut response = read.response.clone().unwrap_or_default();
        response.resolved_at=Some(resolved_at);
        resources.evidence("read:statistics", read.encode_to_vec(), None)?;
        return Ok(response);
    }
    let mut values = Vec::new();
    let mut positions = Vec::new();
    loop {
        let page = resources.store.sample_page(&resources.execution, &request.stream, after, through, page_size)?;
        if page.is_empty() {
            break;
        }
        for sample in page {
            after = sample.position;
            positions.push(sample.position);
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
    }
    if values.is_empty() {
        let response = dokime_protocol::context::EvidenceStatisticsResponse {resolved_at:Some(resolved_at),..Default::default()};
        let read = StatisticsRead {
            request: Some(request),
            positions,
            response: Some(response.clone()),
        };
        resources.store.cache_statistics(&resources.execution, &key, start, frontier, read.clone())?;
        resources.evidence("read:statistics", read.encode_to_vec(), None)?;
        return Ok(response);
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
    let response = dokime_protocol::context::EvidenceStatisticsResponse {
        resolved_at:Some(resolved_at),
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
    };
    let read = StatisticsRead {
        request: Some(request),
        positions,
        response: Some(response.clone()),
    };
    resources.store.cache_statistics(&resources.execution, &key, start, frontier, read.clone())?;
    resources.evidence("read:statistics", read.encode_to_vec(), None)?;
    Ok(response)
}

/* === Private Functions === */

/* === Tests === */
