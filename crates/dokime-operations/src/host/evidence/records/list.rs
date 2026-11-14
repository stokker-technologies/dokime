/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Records List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse;
use dokime_protocol::host::evidence::record_scope::Kind as RecordScopeKind;
use prost::Message;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;

// Local Imports

use dokime_protocol::host::evidence::CapturedRecord;
use dokime_protocol::host::{Field, FieldType, Schema};
use dokime_protocol::model::RecordContracts;
use dokime_storage::ExecutionCut;
use harmos_protocol::model::Declarations;
use prost_reflect::{DescriptorPool, Kind};

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::ReadCut;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.evidence.Records/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<RecordsListRequest, RecordsListResponse>(ID, settings.read_timeout)
}

pub async fn handle(context: &Context, request: RecordsListRequest) -> Result<RecordsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async { read_page(context, request, None) }).await
}

pub(crate) fn read_page(context: &Context, mut request: RecordsListRequest, after: Option<&ReadCut>) -> Result<RecordsListResponse> {
    let selection = request.selection.clone().unwrap_or_default();
    crate::host::evidence::selection::validate(&selection)?;
    let mut scope = selection.scope.clone().unwrap_or_default();
    if let Some(selected) = request.scope.as_ref().and_then(|scope| scope.kind.as_ref()) {
        match selected {
            RecordScopeKind::Execution(id) => {
                if scope.execution_id.as_ref().is_some_and(|value| value != id) {
                    return Err(Error::Invalid("List Records: Keep Both Scope Selections Consistent".into()));
                }
                scope.execution_id = Some(id.clone());
            }
            RecordScopeKind::Cycle(cycle) => {
                if let Some(execution) = &cycle.execution_id {
                    if scope.execution_id.as_ref().is_some_and(|value| value != execution) {
                        return Err(Error::Invalid("List Records: Keep Both Execution Selections Consistent".into()));
                    }
                    scope.execution_id = Some(execution.clone());
                }
                if scope.cycle_id.as_ref().is_some_and(|value| value != &cycle.cycle_id) {
                    return Err(Error::Invalid("List Records: Keep Both Cycle Selections Consistent".into()));
                }
                scope.cycle_id = Some(cycle.cycle_id.clone());
            }
        }
    }
    if context.caller.is_some() {
        if matches!(request.scope.as_ref().and_then(|scope| scope.kind.as_ref()), Some(RecordScopeKind::Cycle(_))) {
            return Err(Error::Permission("List Records: Select the Calling Execution Instead of a Cycle".into()));
        }
        let execution = context.resolve_execution(scope.execution_id.as_deref(), ID)?.to_owned();
        scope.execution_id = Some(execution);
    } else if scope.execution_id.is_none() && scope.cycle_id.is_none() {
        return Err(Error::Invalid("List Records: Select an Execution or Cycle".into()));
    }
    if !request.step.is_empty() {
        if scope.step_id.as_ref().is_some_and(|step| step != &request.step) {
            return Err(Error::Invalid("List Records: Keep Both Step Selections Consistent".into()));
        }
        scope.step_id = Some(request.step.clone());
    }
    if let Some(id) = &scope.execution_id {
        let cut = context.host.store().metadata(id)?;
        if context.caller.is_some() && scope.cycle_id.as_ref().is_some_and(|cycle| cycle != &cut.execution.cycle) {
            return Err(Error::Permission("List Records: Select the Calling Execution's Cycle".into()));
        }
    }
    let page = request.page.take().unwrap_or_default();
    let limit = crate::host::projection::limit(page.page_size, context.settings.page_items);
    let maximum = if page.max_bytes == 0 {
        context.settings.message_bytes
    } else {
        usize::try_from(page.max_bytes).unwrap_or(usize::MAX).min(context.settings.message_bytes)
    };
    if !page.page_token.is_empty() && request.at.is_none() {
        return Err(Error::Invalid("List Records: Continue with the Returned Read Cut".into()));
    }
    let at = request.at.take();
    let position = if let Some(at) = &at {
        if at.positions.len() != 1 || !at.revisions.is_empty() {
            return Err(Error::Invalid("List Records: Use Its Returned Evidence Cut".into()));
        }
        context.host.store().evidence_position(at.positions.first())?
    } else {
        context.host.store().evidence_position(None)?
    };
    let cut = ReadCut {
        positions: vec![position.clone()],
        revisions: Vec::new(),
    };
    let floor = if let Some(after) = after {
        if after.positions.len() != 1
            || !after.revisions.is_empty()
            || after.positions[0].history != position.history
            || after.positions[0].sequence > position.sequence
        {
            return Err(Error::Precondition("Watch Records: A Retained History Was Removed; Resynchronize".into()));
        }
        after.positions[0].sequence
    } else {
        0
    };
    let mut digest = Sha256::new();
    digest.update(scope.encode_to_vec());
    digest.update(request.encode_to_vec());
    digest.update(cut.encode_to_vec());
    digest.update(floor.to_be_bytes());
    let binding = digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let mut scanned = floor;
    if !page.page_token.is_empty() {
        let (key, offset) =
            page.page_token.split_once(':').ok_or_else(|| Error::Invalid("List Records: Use the Returned Page Token".into()))?;
        scanned = offset.parse().map_err(|_| Error::Invalid("List Records: Use the Returned Page Position".into()))?;
        if key != binding || scanned < floor || scanned > position.sequence {
            return Err(Error::Invalid("List Records: Keep the Selection and Cut for Every Page".into()));
        }
    }
    let store = context.host.store();
    let (reader, _cancellation) = store.read_session()?;
    reader.evidence_position(Some(&position))?;
    let entries =
        reader.evidence_entries(scanned, position.sequence, NonZeroUsize::new(context.settings.page_items + 1).unwrap(), false)?;
    let mut response = RecordsListResponse {
        resolved_at: Some(cut),
        ..Default::default()
    };
    let mut more = false;
    for (index, (sequence, execution, evidence)) in entries.into_iter().enumerate() {
        if index == context.settings.page_items {
            more = true;
            break;
        }
        let frozen = reader.metadata(&execution)?;
        if !crate::host::evidence::selection::scope_matches(Some(&scope), &frozen, &evidence) {
            scanned = sequence;
            continue;
        }
        let Some(record) = crate::host::evidence::selection::project(&frozen, &evidence)? else {
            scanned = sequence;
            continue;
        };
        if record_matches(&request, &record) {
            if response.records.len() == limit {
                more = true;
                break;
            }
            response.records.push(record);
            response.next_page_token = format!("{binding}:{sequence}");
            if response.encoded_len() > maximum {
                response.records.pop();
                if response.records.is_empty() {
                    return Err(Error::Bound("List Records: Narrow the Record to Fit the Reply Byte Bound".into()));
                }
                more = true;
                break;
            }
        }
        scanned = sequence;
    }
    let mut cursor = 0;
    let mut schema = None;
    let mut heterogeneous = false;
    loop {
        let entries = reader.evidence_entries(cursor, position.sequence, NonZeroUsize::new(context.settings.page_items).unwrap(), false)?;
        if entries.is_empty() {
            break;
        }
        for (sequence, execution, evidence) in entries {
            cursor = sequence;
            let frozen = reader.metadata(&execution)?;
            if !crate::host::evidence::selection::scope_matches(Some(&scope), &frozen, &evidence) {
                continue;
            }
            let Some(record) = crate::host::evidence::selection::project(&frozen, &evidence)? else {
                continue;
            };
            if record_matches(&request, &record) {
                response.count += 1;
                if !heterogeneous {
                    let candidate = record_schema(&frozen, &record.record_type)?;
                    if schema.as_ref().is_some_and(|saved| Some(saved) != candidate.as_ref()) {
                        if !request.record_type.is_empty() {
                            return Err(Error::Invalid("List Records: Select One Frozen Record Schema Across Executions".into()));
                        }
                        heterogeneous = true;
                        schema = None;
                    } else {
                        schema = candidate;
                    }
                }
                response.last = Some(record);
            }
        }
    }
    response.schema = schema;
    response.next_page_token = if more { format!("{binding}:{scanned}") } else { String::new() };
    if response.encoded_len() > maximum {
        return Err(Error::Bound("List Records: Narrow the Selection to Fit Its Population Metadata".into()));
    }
    crate::host::bounded_reply(context, response)
}

/* === Private Functions === */

fn record_schema(cut: &ExecutionCut, kind: &str) -> Result<Option<Schema>> {
    let (types, message) = if kind == "dokime.log" {
        (dokime_protocol::descriptor_pool().clone(), "dokime.host.v1.evidence.LogsAppendRequest".to_owned())
    } else {
        let catalog = Declarations::decode(cut.execution.catalog.as_slice())?;
        let mut message = None;
        for section in &catalog.sections {
            if section.r#type != "dokime.model.v1.RecordContracts" {
                continue;
            }
            for record in RecordContracts::decode(section.payload.as_slice())?.records {
                if record.metadata.as_ref().is_some_and(|metadata| metadata.identifier == kind) {
                    message = Some(record.message);
                }
            }
        }
        let Some(message) = message else {
            return Ok(None);
        };
        let types = catalog.types.ok_or_else(|| Error::Invalid("List Records: Restore Its Frozen Record Descriptors".into()))?;
        let pool = DescriptorPool::from_file_descriptor_set(types)
            .map_err(|source| Error::Invalid(format!("List Records: Read the Frozen Schema: {source}")))?;
        (pool, message)
    };
    let descriptor =
        types.get_message_by_name(&message).ok_or_else(|| Error::Invalid("List Records: Restore the Frozen Record Message".into()))?;
    let fields = descriptor
        .fields()
        .map(|field| {
            let value_type = if field.is_list() || field.is_map() {
                FieldType::Array
            } else {
                match field.kind() {
                    Kind::Bool => FieldType::Boolean,
                    Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Enum(_) => FieldType::Integer,
                    Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => FieldType::Int64,
                    Kind::Uint32 | Kind::Fixed32 | Kind::Uint64 | Kind::Fixed64 => FieldType::Uint64,
                    Kind::Float | Kind::Double => FieldType::Float64,
                    Kind::String => FieldType::Text,
                    Kind::Message(message) if message.full_name() == "google.protobuf.Timestamp" => FieldType::Datetime,
                    _ => FieldType::Object,
                }
            };
            Field {
                id: field.name().into(),
                value_type: value_type as i32,
                nullable: field.supports_presence(),
                unit: None,
            }
        })
        .collect();
    Ok(Some(Schema {
        identity: message,
        versions: vec![1],
        fields,
    }))
}

fn record_matches(request: &RecordsListRequest, record: &CapturedRecord) -> bool {
    let selection = request.selection.as_ref();
    let record_types = selection.map_or(&[][..], |selection| selection.record_types.as_slice());
    let kind_matches = if request.record_type.is_empty() && record_types.is_empty() {
        record.record_type != "dokime.log"
    } else {
        (request.record_type.is_empty() || request.record_type == record.record_type)
            && (record_types.is_empty()
                || record_types.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, &record.record_type)))
    };
    let source_matches = selection.is_none_or(|selection| {
        selection.sources.is_empty()
            || selection.sources.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, &record.source))
    });
    let severity_matches =
        selection.is_none_or(|selection| selection.severities.is_empty() || selection.severities.contains(&record.severity));
    let version_matches = request.versions.is_empty() || request.versions.contains(&record.schema_version);
    let time = record.timestamp.as_ref().map(|time| (time.seconds, time.nanos));
    let in_window = request.between.as_ref().is_none_or(|window| {
        window.start.as_ref().is_none_or(|start| time.is_some_and(|time| time >= (start.seconds, start.nanos)))
            && window.end.as_ref().is_none_or(|end| time.is_some_and(|time| time <= (end.seconds, end.nanos)))
    });
    kind_matches && source_matches && severity_matches && version_matches && in_window
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{CapturedRecord, RecordsListRequest, record_matches};
    use dokime_protocol::host::evidence::CapturedRecordSelection;
    use dokime_protocol::model::{Severity, Window};
    use prost_types::Timestamp;

    #[test]
    fn record_population_and_page_use_the_same_type_version_and_source_window() {
        let mut request = RecordsListRequest {
            record_type: "reading".into(),
            versions: vec![2],
            between: Some(Window {
                start: Some(Timestamp { seconds: 10, nanos: 5 }),
                end: Some(Timestamp { seconds: 11, nanos: 0 }),
            }),
            selection: Some(CapturedRecordSelection {
                sources: vec!["sensor.*".into()],
                severities: vec![Severity::Info as i32],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut record = CapturedRecord {
            record_type: "reading".into(),
            schema_version: 2,
            source: "sensor.a".into(),
            severity: Severity::Info as i32,
            timestamp: Some(Timestamp { seconds: 10, nanos: 5 }),
            ..Default::default()
        };
        assert!(record_matches(&request, &record));
        record.timestamp.as_mut().unwrap().nanos = 4;
        assert!(!record_matches(&request, &record));
        record.timestamp = Some(Timestamp { seconds: 11, nanos: 0 });
        assert!(record_matches(&request, &record));
        record.timestamp.as_mut().unwrap().nanos = 1;
        assert!(!record_matches(&request, &record));
        request.between = None;
        record.schema_version = 1;
        assert!(!record_matches(&request, &record));
        record.schema_version = 2;
        record.source = "other".into();
        assert!(!record_matches(&request, &record));
    }

    #[test]
    fn logs_require_an_explicit_record_selection() {
        let record = CapturedRecord {
            record_type: "dokime.log".into(),
            ..Default::default()
        };
        let mut request = RecordsListRequest::default();
        assert!(!record_matches(&request, &record));
        request.selection = Some(CapturedRecordSelection {
            record_types: vec!["dokime.*".into()],
            ..Default::default()
        });
        assert!(record_matches(&request, &record));
        request.record_type = "reading".into();
        assert!(!record_matches(&request, &record));
    }
}

crate::native::unary!(RecordsListRequest, RecordsListResponse, context, "dokime.host.v1.evidence", "Records", "List");
