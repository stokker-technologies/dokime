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
use arrow_array::{Float64Array, StructArray, UInt64Array};
use harmos_protocol::model::Declarations;
use harmos_protocol::model::ReadCut;
use prost::Message;
use prost_reflect::{DescriptorPool, Kind};

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_storage::{BatchSelection, Evidence, NumericWindow, StatisticsRead};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Evidence/Statistics";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: EvidenceStatisticsRequest) -> Result<EvidenceStatisticsResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async { read(context, request) }).await
}

fn read(context: &Context, request: EvidenceStatisticsRequest) -> Result<EvidenceStatisticsResponse> {
    let selected = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref());
    let execution = context.resolve_execution(selected, ID)?;
    if let Some(caller) = &context.caller {
        caller.check_action_scope(execution, ID)?;
    }
    let store = context.host.store();
    if request.at.as_ref().is_some_and(|at| at.positions.len() != 1 || !at.revisions.is_empty()) {
        return Err(Error::Invalid("Compute Statistics: Supply Its Returned Execution Cut".into()));
    }
    let mut snapshot = store.cut(execution)?;
    let resolved_at = store.resource_value(execution, "progress", request.at.as_ref().and_then(|at| at.positions.first()))?.resolved_at;
    snapshot.frontier = resolved_at.sequence;
    let catalog = Declarations::decode(snapshot.execution.catalog.as_slice())?;
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
    let scope = request.scope.as_ref();
    if scope.is_some_and(|scope| scope.cycle_id.as_ref().is_some_and(|cycle| cycle != &snapshot.execution.cycle)) {
        return Err(Error::Invalid("Compute Statistics: Select the Execution's Cycle".into()));
    }
    if scope.is_some_and(|scope| {
        scope.session_id.is_some() || scope.case_id.is_some() || scope.criterion_id.is_some() || scope.origin.is_some()
    }) {
        return Err(Error::Invalid("Compute Statistics: Select Execution, Cycle and Step Scopes".into()));
    }
    let scope_step = scope.and_then(|scope| scope.step_id.as_deref()).unwrap_or_default();
    if !request.step.is_empty() && !scope_step.is_empty() && request.step != scope_step {
        return Err(Error::Invalid("Compute Statistics: Keep Both Step Selections Consistent".into()));
    }
    let step = if request.step.is_empty() { scope_step } else { &request.step };
    let start_ns = request
        .between
        .as_ref()
        .and_then(|window| window.start.as_ref())
        .map(crate::resources::timestamp_ns)
        .transpose()?
        .unwrap_or(i64::MIN);
    let end_ns = request
        .between
        .as_ref()
        .and_then(|window| window.end.as_ref())
        .map(crate::resources::timestamp_ns)
        .transpose()?
        .unwrap_or(i64::MAX);
    if start_ns > end_ns {
        return Err(Error::Invalid("Compute Statistics: Choose Increasing Source-Time Bounds".into()));
    }
    let selected = Evidence { step: step.into(), ..Default::default() };
    let scope_matches = crate::host::evidence::selection::scope_matches(scope, &snapshot, &selected);
    let page_size = NonZeroUsize::new(context.settings.page_items).unwrap();
    let (after, through) = store.step_range(execution, step, snapshot.frontier, page_size)?;
    let start = after;
    let frontier = store.sample_frontier(execution, &request.stream, after, through)?;
    let mut key = request.clone();
    key.field = field.name().into();
    if let Some(mut read) = store.cached_statistics(execution, &key, start, frontier)? {
        read.request = Some(request);
        let response = read.response.get_or_insert_default();
        response.resolved_at = Some(ReadCut {
            positions: vec![resolved_at],
            revisions: Vec::new(),
        });
        let response = response.clone();
        if let Some(caller) = &context.caller {
            caller.evidence("read:statistics", read.encode_to_vec(), None)?;
        }
        return Ok(response);
    }
    let mut positions = Vec::new();
    store.read_batches(
        BatchSelection {
            execution,
            stream: &request.stream,
            after,
            through,
            start_ns,
            end_ns,
        },
        &[field.name().into()],
        |batch| {
            let values = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
            if let Some(column) = values.column_by_name(field.name()).and_then(|a| a.as_any().downcast_ref::<Float64Array>())
                && column.values().iter().any(|v| !v.is_finite())
            {
                return Err(dokime_storage::Error::Data("Compute Statistics: Publish Finite Values"));
            }
            let metadata = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
            let values = metadata.column_by_name("position").unwrap().as_any().downcast_ref::<UInt64Array>().unwrap();
            positions.extend(values.values().iter().copied());
            Ok(true)
        },
    )?;
    let mut statistics = store.numeric_statistics(&NumericWindow {
        execution,
        stream: &request.stream,
        field: field.name(),
        after,
        through,
        start_ns,
        end_ns,
        bins: NonZeroUsize::MIN,
    })?;
    if !scope_matches || !request.versions.is_empty() && !request.versions.contains(&1) {
        statistics = Default::default();
        positions.clear();
    }
    let response = EvidenceStatisticsResponse {
        statistics: Some(statistics),
        resolved_at: Some(ReadCut {
            positions: vec![resolved_at],
            revisions: Vec::new(),
        }),
    };
    let read = StatisticsRead {
        request: Some(request.clone()),
        positions,
        response: Some(response.clone()),
    };
    store.cache_statistics(execution, &key, start, frontier, read.clone())?;
    if let Some(caller) = &context.caller {
        caller.evidence("read:statistics", read.encode_to_vec(), None)?;
    }
    Ok(response)
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(EvidenceStatisticsRequest, EvidenceStatisticsResponse, context, "dokime.host.v1", "Evidence", "Statistics");
