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
use crate::host::evidence::selection;
use crate::{Error, Result, host, host::Context, resources};
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_storage::{BatchSelection, Evidence, NumericWindow, StatisticsRead, Store};

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
    host::complete(ID, context.settings.read_timeout, async { read(context, request) }).await
}

/* === Private Functions === */

fn read(context: &Context, request: EvidenceStatisticsRequest) -> Result<EvidenceStatisticsResponse> {
    let selected = request.scope.as_ref().and_then(|scope| scope.execution_id.as_deref());
    let execution = context.resolve_execution(selected, ID)?.to_owned();
    if let Some(caller) = &context.caller {
        caller.check_action_scope(&execution, ID)?;
    }
    let store = context.host.store();
    let read = select(&store, &execution, request, context.settings.page_items)?;
    if let Some(caller) = &context.caller {
        caller.evidence("read:statistics", read.encode_to_vec(), None)?;
    }
    Ok(read.response.unwrap())
}

fn select(store: &Store, execution: &str, request: EvidenceStatisticsRequest, page_items: usize) -> Result<StatisticsRead> {
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
    let start_ns =
        request.between.as_ref().and_then(|window| window.start.as_ref()).map(resources::timestamp_ns).transpose()?.unwrap_or(i64::MIN);
    let end_ns =
        request.between.as_ref().and_then(|window| window.end.as_ref()).map(resources::timestamp_ns).transpose()?.unwrap_or(i64::MAX);
    if start_ns > end_ns {
        return Err(Error::Invalid("Compute Statistics: Choose Increasing Source-Time Bounds".into()));
    }
    let selected = Evidence { step: step.into(), ..Default::default() };
    let scope_matches = selection::scope_matches(scope, &snapshot, &selected);
    let page_size = NonZeroUsize::new(page_items).unwrap();
    let (after, through) = store.step_range(execution, step, snapshot.frontier, page_size)?;
    let start = after;
    let frontier = store.sample_frontier(execution, &request.stream, after, through)?;
    let mut key = request.clone();
    key.field = field.name().into();
    if let Some(mut read) = store.cached_statistics(execution, &key, start, frontier)? {
        let response = read.response.get_or_insert_default();
        response.resolved_at = Some(ReadCut {
            positions: vec![resolved_at],
            revisions: Vec::new(),
        });
        return Ok(read);
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
        request: Some(request),
        positions,
        response: Some(response),
    };
    store.cache_statistics(execution, &key, start, frontier, read.clone())?;
    Ok(read)
}

/* === Tests === */

crate::native::unary!(EvidenceStatisticsRequest, EvidenceStatisticsResponse, context, "dokime.host.v1", "Evidence", "Statistics");

#[cfg(test)]
mod tests {
    use prost::Message;
    use prost_types::{
        DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, Timestamp,
        field_descriptor_proto::{Label, Type},
    };
    use tempfile::TempDir;

    use super::{EvidenceStatisticsRequest, Result, Store, select};
    use dokime_protocol::model::{EvidenceScope, Import, Origin, Window, origin, value};
    use dokime_storage::{Evidence, Execution, Sample};
    use harmos_protocol::model::{Declarations, StreamDeclaration};

    #[derive(Clone, PartialEq, Message)]
    struct Row {
        #[prost(uint64, tag = "1")]
        value: u64,
    }

    fn fixture() -> Result<(TempDir, Store, EvidenceStatisticsRequest)> {
        let root = tempfile::tempdir().map_err(dokime_storage::Error::from)?;
        let store = Store::open(root.path())?;
        let types = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("statistics.proto".into()),
                package: Some("statistics".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Row".into()),
                    field: vec![FieldDescriptorProto {
                        name: Some("value".into()),
                        number: Some(1),
                        label: Some(Label::Optional as i32),
                        r#type: Some(Type::Uint64 as i32),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let catalog = Declarations {
            types: Some(types),
            streams: vec![StreamDeclaration {
                id: "readings".into(),
                row: "statistics.Row".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store.create(&Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::Import(Import::default())),
            }),
            catalog: catalog.encode_to_vec(),
            ..Default::default()
        })?;
        store.open_stream("execution", "readings")?;
        store.append(
            "execution",
            Evidence {
                step: "measure".into(),
                kind: "step:start".into(),
                ..Default::default()
            },
        )?;
        for (source_ns, value) in [(10, 1), (20, 100)] {
            store.sample(
                "execution",
                Sample {
                    stream: "readings".into(),
                    source_ns,
                    row: Row { value }.encode_to_vec(),
                    ..Default::default()
                },
            )?;
        }
        store.append(
            "execution",
            Evidence {
                step: "measure".into(),
                kind: "step:end".into(),
                ..Default::default()
            },
        )?;
        let request = EvidenceStatisticsRequest {
            stream: "readings".into(),
            step: "measure".into(),
            ..Default::default()
        };
        Ok((root, store, request))
    }

    #[test]
    fn cached_statistics_keep_source_time_windows_at_the_same_frontier() -> Result<()> {
        let (_root, store, mut request) = fixture()?;
        let frontier = store.cut("execution")?.frontier;
        for (nanos, expected) in [(10, 1), (20, 100)] {
            let timestamp = Timestamp { seconds: 0, nanos };
            request.between = Some(Window {
                start: Some(timestamp),
                end: Some(timestamp),
            });
            let read = select(&store, "execution", request.clone(), 1)?;
            assert_eq!(read.request.as_ref(), Some(&request));
            assert_eq!(read.positions.len(), 1);
            let statistics = read.response.unwrap().statistics.unwrap();
            assert_eq!(statistics.count, 1);
            assert_eq!(statistics.mean.unwrap().kind, Some(value::Kind::UnsignedInteger(expected)));
        }
        assert_eq!(store.cut("execution")?.frontier, frontier);
        Ok(())
    }

    #[test]
    fn cached_statistics_keep_scope_and_version_selections() -> Result<()> {
        let (_root, store, request) = fixture()?;
        let scoped = EvidenceStatisticsRequest {
            scope: Some(EvidenceScope {
                template_id: Some("foreign".into()),
                ..Default::default()
            }),
            ..request.clone()
        };
        let versioned = EvidenceStatisticsRequest { versions: vec![2], ..request.clone() };
        for excluded in [scoped, versioned] {
            let original = select(&store, "execution", request.clone(), 1)?;
            assert_eq!(original.response.as_ref().unwrap().statistics.as_ref().unwrap().count, 2);
            let filtered = select(&store, "execution", excluded.clone(), 1)?;
            assert_eq!(filtered.request, Some(excluded));
            assert!(filtered.positions.is_empty());
            assert_eq!(filtered.response.unwrap().statistics.unwrap().count, 0);
        }
        Ok(())
    }

    #[test]
    fn cache_hits_preserve_the_original_request_and_positions() -> Result<()> {
        let (_root, store, request) = fixture()?;
        let original = select(&store, "execution", request.clone(), 1)?;
        let mut explicit = request.clone();
        explicit.field = "value".into();
        let cached = select(&store, "execution", explicit, 1)?;
        assert_eq!(cached.request, Some(request));
        assert_eq!(cached.positions, original.positions);
        assert_eq!(cached.response, original.response);
        Ok(())
    }
}
