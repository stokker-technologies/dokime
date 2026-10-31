/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Captured Record Queries

/* === Modules === */

#[cfg(feature = "testing")]
#[path = "record_fixture.rs"]
mod testing;

/* === Re-Exports === */

#[cfg(feature = "testing")]
pub use testing::Fixture as RecordQueryFixture;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::{MemorySize, Message, operations::OperationDeclaration};

// Local Imports
use super::super::{
    history::storage_error,
    selection::{Selection, SelectionRequest},
    snapshots::{QuerySnapshot, Scope, Seek},
    tokens::{self, Token, Tokens},
};
use crate::{
    RuntimeHandle,
    config::QueryConfig,
    storage::sqlite::{
        State,
        data::{RecordQueryRead, RecordQueryScan},
    },
};
use dokime_domain::model::{
    identity::{TestCycleId, TestExecutionId},
    query::{
        error::{Failure, Reason},
        page::{PageLimits, RecordPage},
        selection::RecordScope,
    },
    records::RecordContract,
};
use dokime_operations::query::ListRecords;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(serde::Deserialize)]
struct Contracts {
    records: Vec<RecordContract>,
    record_epoch: Option<String>,
}

struct EncodedBytes {
    used: u64,
    maximum: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::io::Write for EncodedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let used = self.used.saturating_add(bytes.len() as u64);
        if used > self.maximum {
            return Err(std::io::Error::other("record query encoding exceeds byte budget"));
        }
        self.used = used;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl RuntimeHandle {
    pub async fn query_list_records(&self, request: ListRecords, message_bytes: usize) -> Result<RecordPage, Box<Failure>> {
        let operation = ListRecords::ID;
        tokio::time::timeout(self.query_config().limits().active_time, async {
            request.validate_shape()?;
            let (scope, _) = record_scope(&request.scope)?;
            let limits = self.query_config().page_limits(&request.page, message_bytes, operation)?;
            let binding = tokens::binding(&request, self.query_config().limits().working_bytes as usize, operation)?;
            let snapshot = self
                .query_snapshot(scope, request.at.as_ref(), request.page.after.as_deref(), binding, operation)
                .await
                .map_err(|mut error| {
                    error.operation = operation.into();
                    error
                })?;
            page(&self.local_storage, self.query_config(), &request, snapshot, limits, binding).await
        })
        .await
        .map_err(|_| Failure::new(Reason::Deadline, operation, "Record query exceeded its configured active-work deadline"))?
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn page(
    local: &State, config: &QueryConfig, request: &ListRecords, snapshot: QuerySnapshot, limits: PageLimits, binding: [u8; 32],
) -> Result<RecordPage, Box<Failure>> {
    let operation = ListRecords::ID;
    let (_, scope) = record_scope(&request.scope)?;
    if let RecordScope::Cycle(cycle) = &request.scope
        && let Some(execution) = &cycle.execution_id
        && !local.query_record_member(snapshot.scoped_cut(), execution.clone()).await.map_err(error)?
    {
        return Err(Failure::new(Reason::ScopeNotFound, operation, "Execution is not a member of the captured cycle"));
    }
    let after = match &snapshot.after {
        None => 0,
        Some(Seek::Evidence { position, occurrence }) if *position <= snapshot.cut.at.sequence && occurrence.len() == 36 => *position,
        _ => return Err(Failure::new(Reason::InvalidContinuation, operation, "Continuation does not identify a record position")),
    };
    let bytes = local.snapshot_contracts(snapshot.scoped_cut(), config.limits().schema_bytes as usize).await.map_err(error)?;
    let contracts: Contracts =
        serde_json::from_slice(&bytes).map_err(|_| Failure::new(Reason::Corruption, operation, "Captured record contracts are invalid"))?;
    let selection = Selection::records(
        &contracts.records,
        &request.record_type,
        SelectionRequest {
            versions: &request.versions,
            fields: &[],
            predicates: &request.r#where,
            between: request.between.as_ref(),
            operation,
        },
        config,
    )?;
    let retained = contracts
        .records
        .memory_size()
        .saturating_add(bytes.len())
        .saturating_add(selection.schema().memory_size())
        .saturating_add(request.memory_size());
    let tokens = Tokens {
        key: local.snapshot_key().await.map_err(error)?,
        maximum: config.limits().token_bytes,
    };
    let continuation = |last| {
        tokens.encode(
            &Token::Continuation {
                cut: snapshot.cut.clone(),
                selection: binding,
                last,
            },
            operation,
        )
    };
    let mut response = RecordPage {
        rows: Vec::new(),
        schema: Some(selection.schema().clone()),
        snapshot: Some(snapshot.snapshot.clone()),
        limits: Some(limits.clone()),
        next: Some(continuation(Seek::Evidence {
            position: u64::MAX,
            occurrence: "f".repeat(36),
        })?),
        complete: false,
    };
    let mut encoded = json_size(&response, limits.bytes)?;
    let overhead = encoded;
    let mut wire = response.encoded_size() as u64;
    let wire_overhead = wire;
    let working = (config.limits().working_bytes as usize)
        .checked_sub(retained.saturating_add(response.memory_size()))
        .filter(|bytes| *bytes > 0)
        .ok_or_else(|| resource("working_bytes", config.limits().working_bytes))?;
    let scan_rows = limits.rows.min(u32::try_from(config.limits().scanned_rows).unwrap_or(u32::MAX));
    let RecordQueryScan { rows, bytes, mut complete } = local
        .query_records(RecordQueryRead {
            cut: snapshot.scoped_cut(),
            scope,
            record_epoch: contracts.record_epoch,
            after,
            rows: scan_rows,
            bytes: working / 2,
            scanned_bytes: config.limits().scanned_bytes,
        })
        .await
        .map_err(error)?;
    if bytes > config.limits().scanned_bytes {
        return Err(resource("scanned_bytes", config.limits().scanned_bytes));
    }
    let scanned = rows.len() as u64;
    let mut last = None;
    let mut response_memory = 0usize;
    for row in rows {
        if response.rows.len() == limits.rows as usize {
            complete = false;
            break;
        }
        let seek = Seek::Evidence {
            position: row.position,
            occurrence: row.record.record_id.to_string(),
        };
        let mut record = row.record;
        if record.record_type.as_str() != request.record_type || !selection.schema().versions.contains(&record.schema_version) {
            last = Some(seek);
            continue;
        }
        // Undeclared Extras Are Neither Query Fields Nor Captured-Schema Output.
        let contract = contracts
            .records
            .iter()
            .find(|contract| contract.record_type.as_str() == request.record_type && contract.schema_version == record.schema_version)
            .expect("selected captured record contract");
        record.details.retain(|id, _| contract.fields.iter().any(|field| field.metadata.identifier.as_str() == id));
        for field in &contract.fields {
            if let Some(value) = record.details.get(field.metadata.identifier.as_str()) {
                let valid = match value {
                    dokime_domain::model::values::Value::Null => !field.required || field.nullable,
                    _ => dokime_domain::model::values::value_matches_type(value, &field.value_type),
                };
                if !valid {
                    let mut failure = Failure::new(Reason::Corruption, operation, "Stored record field differs from its captured type");
                    failure.field = Some(field.metadata.identifier.to_string());
                    return Err(failure);
                }
            }
        }
        if !super::projection::matches(&selection, &record)? {
            last = Some(seek);
            continue;
        }
        let record = super::projection::record(&record)?;
        let row_bytes = match json_size(&record, limits.bytes) {
            Ok(bytes) => bytes + 1,
            Err(_) if !response.rows.is_empty() => {
                complete = false;
                break;
            }
            Err(error) => return Err(error),
        };
        let row_wire = record.encoded_size() as u64 + 10;
        if !response.rows.is_empty() && (encoded.saturating_add(row_bytes) > limits.bytes || wire.saturating_add(row_wire) > limits.bytes) {
            complete = false;
            break;
        }
        if overhead.saturating_add(row_bytes) > limits.bytes || wire_overhead.saturating_add(row_wire) > limits.bytes {
            return Err(resource("page_bytes", limits.bytes));
        }
        response_memory = response_memory.saturating_add(record.memory_size());
        if response_memory > working / 2 {
            return Err(resource("working_bytes", config.limits().working_bytes));
        }
        encoded += row_bytes;
        wire += row_wire;
        response.rows.push(record);
        last = Some(seek);
    }
    if !complete && scanned >= config.limits().scanned_rows {
        return Err(resource("scanned_rows", config.limits().scanned_rows));
    }
    response.next =
        if complete { None } else { Some(continuation(last.ok_or_else(|| resource("working_bytes", config.limits().working_bytes))?)?) };
    response.complete = complete;
    json_size(&response, limits.bytes)?;
    if response.encoded_size() as u64 > limits.bytes {
        return Err(resource("page_bytes", limits.bytes));
    }
    Ok(response)
}

fn record_scope(scope: &RecordScope) -> Result<(Scope, crate::evidence_runtime::ScopeSelection), Box<Failure>> {
    let invalid = || Failure::new(Reason::InvalidSelection, ListRecords::ID, "Select valid execution and cycle identifiers");
    let mut selected = crate::evidence_runtime::ScopeSelection::default();
    let scope = match scope {
        RecordScope::Execution(id) => {
            selected.test_execution_id = Some(TestExecutionId::from_uuid(id).map_err(|_| invalid())?);
            Scope::Execution(id.clone())
        }
        RecordScope::Cycle(scope) => {
            selected.test_cycle_id = Some(TestCycleId::from_uuid(&scope.cycle_id).map_err(|_| invalid())?);
            selected.test_execution_id = scope.execution_id.as_ref().map(TestExecutionId::from_uuid).transpose().map_err(|_| invalid())?;
            Scope::Cycle(scope.cycle_id.clone())
        }
    };
    Ok((scope, selected))
}

fn json_size(value: &impl serde::Serialize, maximum: u64) -> Result<u64, Box<Failure>> {
    let mut encoded = EncodedBytes { used: 0, maximum };
    serde_json::to_writer(&mut encoded, value).map_err(|_| resource("page_bytes", maximum))?;
    Ok(encoded.used)
}

fn error(value: crate::storage::Error) -> Box<Failure> {
    if matches!(value, crate::storage::Error::Invalid(_) | crate::storage::Error::Timestamp(_) | crate::storage::Error::HarmosDomain(_)) {
        return Failure::new(Reason::Corruption, ListRecords::ID, "Stored record evidence is invalid");
    }
    let mut error = storage_error(value);
    error.operation = ListRecords::ID.into();
    error
}

fn resource(name: &str, value: u64) -> Box<Failure> {
    let mut error = Failure::new(
        Reason::ResourceLimit,
        ListRecords::ID,
        "Record query exceeds its configured budget; narrow the selection or increase the page budget",
    );
    error.limit = Some(dokime_domain::model::query::error::EffectiveLimit { name: name.into(), value });
    error
}

/* === Tests === */
