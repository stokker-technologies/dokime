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
use harmos_protocol::v1::OperationContract;
use prost::Message;
use prost_reflect::{DescriptorPool, Kind};

// Local Imports
use crate::{Error, Resources, Result};
use dokime_protocol::context::EvidenceStatisticsRequest;
use dokime_protocol::context::EvidenceStatisticsResponse;
use dokime_storage::StatisticsRead;

use dokime_storage::BatchSelection;
use dokime_storage::NumericWindow;

/* === Constants === */

pub const ID: &str = "dokime.plugin.host.v1.Evidence/Statistics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<OperationContract> {
    crate::declaration::<dokime_protocol::context::EvidenceStatisticsRequest, dokime_protocol::context::EvidenceStatisticsResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: EvidenceStatisticsRequest) -> Result<EvidenceStatisticsResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let mut snapshot = resources.store.cut(&resources.execution)?;
    let resolved_at = resources.store.resource_value(&resources.execution, "progress", request.at.as_ref())?.resolved_at;
    snapshot.frontier = resolved_at.sequence;
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
    if field.is_list()
        || field.is_map()
        || !matches!(
            field.kind(),
            Kind::Float
                | Kind::Double
                | Kind::Int32
                | Kind::Sint32
                | Kind::Sfixed32
                | Kind::Uint32
                | Kind::Fixed32
                | Kind::Int64
                | Kind::Sint64
                | Kind::Sfixed64
                | Kind::Uint64
                | Kind::Fixed64
        )
    {
        return Err(Error::Invalid("Select a Numeric Stream Column".into()));
    }
    let page_size = NonZeroUsize::new(256).unwrap();
    let (after, through) = resources.store.step_range(&resources.execution, &request.step, snapshot.frontier, page_size)?;
    let start = after;
    let frontier = resources.store.sample_frontier(&resources.execution, &request.stream, after, through)?;
    let mut key = request.clone();
    key.field = field.name().into();
    if let Some(mut read) = resources.store.cached_statistics(&resources.execution, &key, start, frontier)? {
        read.request = Some(request);
        let mut response = read.response.clone().unwrap_or_default();
        response.resolved_at = Some(resolved_at);
        resources.evidence("read:statistics", read.encode_to_vec(), None)?;
        return Ok(response);
    }
    let mut positions = Vec::new();
    resources.store.read_batches(
        BatchSelection {
            execution: &resources.execution,
            stream: &request.stream,
            after,
            through,
            start_ns: i64::MIN,
            end_ns: i64::MAX,
        },
        &[field.name().into()],
        |batch| {
            use arrow_array::{Float64Array, StructArray, UInt64Array};
            let values = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
            if let Some(column) = values.column_by_name(field.name()).and_then(|a| a.as_any().downcast_ref::<Float64Array>())
                && column.values().iter().any(|v| !v.is_finite())
            {
                return Err(dokime_storage::Error::Data("publish finite values before computing statistics"));
            }
            let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
            let values = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
            positions.extend(values.values().iter().copied());
            Ok(true)
        },
    )?;
    let mut response = resources.store.numeric_statistics(&NumericWindow {
        execution: &resources.execution,
        stream: &request.stream,
        field: field.name(),
        after,
        through,
        start_ns: i64::MIN,
        end_ns: i64::MAX,
        bins: NonZeroUsize::MIN,
    })?;
    response.resolved_at = Some(resolved_at);
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
