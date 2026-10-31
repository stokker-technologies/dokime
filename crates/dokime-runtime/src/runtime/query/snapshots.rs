/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Capture, Token Reuse And Seek Continuation Lifecycle

/* === Modules === */

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;

/* === Re-Exports === */

pub use super::tokens::Seek;
pub use crate::storage::sqlite::snapshots::{Cut, Scope};

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::{ObjectVersion, ObjectVersionToken, StorageCursor, StoreHistory};

// Local Imports
use super::{
    history::storage_error,
    tokens::{self, Token, Tokens},
};
use crate::RuntimeHandle;
use dokime_domain::model::{
    identity::{Identifier, TestCycleId, TestExecutionId, wire::Timestamp},
    query::{
        error::{Failure, Reason},
        snapshot::{Snapshot, SnapshotRef},
    },
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct QuerySnapshot {
    pub snapshot: Snapshot,
    pub cut: Cut,
    pub after: Option<Seek>,
    pub scope: Scope,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl QuerySnapshot {
    pub(crate) fn scoped_cut(&self) -> Cut {
        let mut cut = self.cut.clone();
        cut.scope = self.scope.clone();
        cut
    }
}

impl RuntimeHandle {
    // Registration/Caller Authorization Remains With The Operation Owner. Tokens
    // Are Read Coordinates And Never Grant Scope Access.
    pub async fn query_snapshot(
        &self, scope: Scope, at: Option<&SnapshotRef>, after: Option<&str>, selection: [u8; 32], operation: &str,
    ) -> Result<QuerySnapshot, Box<Failure>> {
        let config = self.query_config().limits();
        tokio::time::timeout(config.active_time, self.query_snapshot_inner(scope, at, after, selection, operation))
            .await
            .map_err(|_| Failure::new(Reason::Deadline, operation, "Query capture exceeded its configured active-work deadline"))?
    }

    async fn query_snapshot_inner(
        &self, scope: Scope, at: Option<&SnapshotRef>, after: Option<&str>, selection: [u8; 32], operation: &str,
    ) -> Result<QuerySnapshot, Box<Failure>> {
        validate_scope(&scope, operation)?;
        let tokens = self.query_tokens().await?;
        let mut cut = match at {
            None => None,
            Some(SnapshotRef::Token(token)) => match tokens.decode(token, operation)? {
                Token::Snapshot(cut) => Some(cut),
                _ => return Err(Failure::new(Reason::InvalidContinuation, operation, "Expected a snapshot token")),
            },
            Some(SnapshotRef::Checkpoint(checkpoint)) if after.is_none() => {
                Some(self.query_checkpoint(&scope, checkpoint, operation).await?)
            }
            Some(SnapshotRef::Checkpoint(_)) => None,
        };
        let last = if let Some(after) = after {
            let Token::Continuation { cut: previous, selection: bound, last } = tokens.decode(after, operation)? else {
                return Err(Failure::new(Reason::InvalidContinuation, operation, "Expected a seek continuation"));
            };
            if selection != bound || cut.as_ref().is_some_and(|cut| cut != &previous) {
                return Err(Failure::new(Reason::InvalidContinuation, operation, "Continuation belongs to another selection or snapshot"));
            }
            cut = Some(previous);
            Some(last)
        } else {
            None
        };
        if let (Some(SnapshotRef::Checkpoint(checkpoint)), Some(previous)) = (at, cut.as_ref()) {
            let native = checkpoint_coordinates(checkpoint, self.query_config().limits().token_bytes, operation)?;
            if previous.scope != Scope::Execution(native.execution_id.to_string())
                || previous.at != native.evidence_through
                || previous.metadata != native.position.sequence
            {
                return Err(Failure::new(Reason::InvalidContinuation, operation, "Continuation belongs to another checkpoint"));
            }
        }
        let mut cut = match cut {
            Some(cut) => cut,
            None => {
                let expires = expiry(self.query_config().limits().snapshot_lifetime, operation)?;
                if scope == Scope::History {
                    let _evidence = self.evidence_admission.lock().await;
                    let (contracts, journal, metadata) = self.query_capture_contracts(&scope, operation).await?;
                    let maximum = self.query_config().limits().schema_bytes as usize;
                    let retained = self.local_storage.query_telemetry_contracts(scope.clone(), metadata, maximum, operation).await?;
                    let contracts = merge_telemetry_contracts(&scope, &contracts, retained, maximum, operation)?;
                    self.local_storage
                        .snapshot_capture(scope.clone(), None, Some(metadata), expires, contracts, Some(journal))
                        .await
                        .map_err(storage_error)?
                } else {
                    // Drain The Existing Fixed Publication Frontier Before Taking
                    // Admission Gates; Ingest Uses The Telemetry Gate Itself.
                    self.flush_plugin_publications().await.map_err(|_| unavailable(operation))?;
                    let _evidence = self.evidence_admission.lock().await;
                    let _telemetry = self.telemetry_admission.lock().await;
                    let (contracts, journal, metadata) = self.query_capture_contracts(&scope, operation).await?;
                    self.record_source.wait_through(&self.state_manager, journal).await.map_err(|_| unavailable(operation))?;
                    let admitted = self.storage().cursor_accepted();
                    self.storage().flush_until(&admitted).await.map_err(|_| unavailable(operation))?;
                    let maximum = self.query_config().limits().schema_bytes as usize;
                    let retained =
                        self.local_storage.query_telemetry_contracts(scope.clone(), admitted.sequence, maximum, operation).await?;
                    let contracts = merge_telemetry_contracts(&scope, &contracts, retained, maximum, operation)?;
                    self.local_storage
                        .snapshot_capture(scope.clone(), Some(admitted), Some(metadata), expires, contracts, Some(journal))
                        .await
                        .map_err(storage_error)?
                }
                .ok_or_else(|| Failure::new(Reason::ScopeNotFound, operation, "Scope has no retained local revision"))?
            }
        };
        let backend = self.state_read().identity.node.node_id.to_string();
        let resumed = after.is_some() || matches!(at, Some(SnapshotRef::Token(_)));
        if resumed && cut.backend != backend {
            return Err(unavailable(operation));
        }
        if !resumed {
            cut.backend = backend;
        }
        validate_cut(&cut, &scope, now(), operation)?;
        let mut scoped = cut.clone();
        scoped.scope = scope.clone();
        if !self.local_storage.snapshot_available(scoped).await.map_err(storage_error)? {
            return Err(unavailable(operation));
        }
        let token = tokens.encode(&Token::Snapshot(cut.clone()), operation)?;
        Ok(QuerySnapshot {
            snapshot: Snapshot {
                token,
                identity: tokens::identity(&cut),
                expires_at: Some(cut.expires),
            },
            cut,
            after: last,
            scope,
        })
    }

    pub async fn query_continuation(
        &self, snapshot: &QuerySnapshot, selection: [u8; 32], last: Seek, operation: &str,
    ) -> Result<String, Box<Failure>> {
        self.query_tokens().await?.encode(
            &Token::Continuation {
                cut: snapshot.cut.clone(),
                selection,
                last,
            },
            operation,
        )
    }

    pub async fn query_cycle_members(
        &self, snapshot: &QuerySnapshot, after: &str, limit: u32, operation: &str,
    ) -> Result<Vec<String>, Box<Failure>> {
        if limit == 0 || limit > self.query_config().limits().max_page_rows {
            return Err(Failure::new(Reason::InvalidSelection, operation, "Invalid member page size"));
        }
        if (limit as usize).saturating_mul(36 + std::mem::size_of::<String>()) > self.query_config().limits().working_bytes as usize {
            return Err(Failure::new(Reason::ResourceLimit, operation, "Member page exceeds the working budget"));
        }
        validate_cut(&snapshot.cut, &snapshot.scope, now(), operation)?;
        let cut = snapshot.scoped_cut();
        self.local_storage
            .snapshot_members(cut, after.to_owned(), limit)
            .await
            .map_err(storage_error)?
            .ok_or_else(|| unavailable(operation))
    }

    pub async fn query_snapshot_contracts(&self, snapshot: &QuerySnapshot, operation: &str) -> Result<Vec<u8>, Box<Failure>> {
        validate_cut(&snapshot.cut, &snapshot.scope, now(), operation)?;
        self.local_storage
            .snapshot_contracts(snapshot.scoped_cut(), self.query_config().limits().schema_bytes as usize)
            .await
            .map_err(storage_error)
    }

    async fn query_capture_contracts(&self, scope: &Scope, operation: &str) -> Result<(Vec<u8>, u64, u64), Box<Failure>> {
        let mut retained = None;
        if let Scope::Execution(id) = scope {
            let id = TestExecutionId::from_uuid(id).map_err(|_| unavailable(operation))?;
            if let Some(view) = crate::runtime::storage::execution::ExecutionView::open(self.storage(), &id, ObjectVersion::Latest)
                .await
                .map_err(|_| unavailable(operation))?
                && view.execution().saved.is_some()
            {
                let view = view.snapshot().await.map_err(|_| unavailable(operation))?;
                let source = view.source_snapshot().map_err(|_| unavailable(operation))?;
                retained = Some(contract_bytes(
                    source.record_contracts.values().flat_map(|versions| versions.values()),
                    source.telemetry_contracts.values().flat_map(|versions| versions.values()),
                    self.query_config().limits().schema_bytes as usize,
                    operation,
                )?);
            }
        }
        let captured = self
            .state_manager
            .state()
            .read(|state| {
                let contracts = match retained {
                    Some(contracts) => contracts,
                    None => contract_bytes(
                        state.historical_record_contracts().values().flat_map(|versions| versions.values()),
                        state.telemetry_definitions.values(),
                        self.query_config().limits().schema_bytes as usize,
                        operation,
                    )?,
                };
                Ok::<_, Box<Failure>>((contracts, self.storage().cursor_accepted().sequence))
            })
            .await
            .map_err(|_| unavailable(operation))?;
        let (contracts, metadata) = captured.value?;
        Ok((contracts, captured.position.0, metadata))
    }

    async fn query_tokens(&self) -> Result<Tokens, Box<Failure>> {
        Ok(Tokens {
            key: self.local_storage.snapshot_key().await.map_err(storage_error)?,
            maximum: self.query_config().limits().token_bytes,
        })
    }

    async fn query_checkpoint(
        &self, scope: &Scope, checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint, operation: &str,
    ) -> Result<Cut, Box<Failure>> {
        if scope != &Scope::Execution(checkpoint.execution_identifier.clone()) {
            return Err(unavailable(operation));
        }
        let native = checkpoint_coordinates(checkpoint, self.query_config().limits().token_bytes, operation)?;
        let retained = crate::runtime::storage::execution::ExecutionView::open_checkpoint(self.storage(), &native)
            .await
            .map_err(|_| unavailable(operation))?;
        if retained.is_none() {
            return Err(unavailable(operation));
        }
        let view = retained.expect("retained checkpoint").snapshot().await.map_err(|_| unavailable(operation))?;
        let source = view.source_snapshot().map_err(|_| unavailable(operation))?;
        let contracts = contract_bytes(
            source.record_contracts.values().flat_map(|versions| versions.values()),
            source.telemetry_contracts.values().flat_map(|versions| versions.values()),
            self.query_config().limits().schema_bytes as usize,
            operation,
        )?;
        drop(view);
        self.local_storage
            .snapshot_capture(
                scope.clone(),
                Some(native.evidence_through),
                Some(native.position.sequence),
                expiry(self.query_config().limits().snapshot_lifetime, operation)?,
                contracts,
                None,
            )
            .await
            .map_err(storage_error)?
            .ok_or_else(|| unavailable(operation))
    }
}

/* === Public Functions === */

pub(super) fn validate_cut(cut: &Cut, scope: &Scope, now: Timestamp, operation: &str) -> Result<(), Box<Failure>> {
    if cut.source != "local" {
        return Err(Failure::new(Reason::UnsupportedSource, operation, "Query snapshots cover only the local store"));
    }
    let expires = cut.expires.to_datetime().map_err(|_| unavailable(operation))?;
    if now.to_datetime().map_err(|_| unavailable(operation))? >= expires {
        return Err(Failure::new(Reason::SnapshotExpired, operation, "Query snapshot has expired"));
    }
    if !cut.compatible(scope) {
        return Err(unavailable(operation));
    }
    Ok(())
}

/* === Private Functions === */

fn merge_telemetry_contracts(
    scope: &Scope, bytes: &[u8], retained: Vec<crate::telemetry::TelemetryContract>, maximum: usize, operation: &str,
) -> Result<Vec<u8>, Box<Failure>> {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct Contracts {
        records: Vec<crate::records::RecordContract>,
        telemetry: Vec<crate::telemetry::TelemetryContract>,
    }
    let mut captured: Contracts = serde_json::from_slice(bytes).map_err(|_| unavailable(operation))?;
    // Persisted Scoped Contracts Own Streams Already Present In This Evidence Cut.
    if *scope != Scope::History {
        captured.telemetry.retain(|contract| !retained.iter().any(|stored| stored.stream == contract.stream));
    }
    for contract in retained {
        if let Some(prior) =
            captured.telemetry.iter().find(|prior| prior.stream == contract.stream && prior.schema_version == contract.schema_version)
        {
            if prior.stream_schema() != contract.stream_schema() {
                return Err(Failure::new(
                    Reason::IncompatibleSchema,
                    operation,
                    "A retained stream reuses a version with different contracts",
                ));
            }
            continue;
        }
        captured.telemetry.push(contract);
    }
    tokens::bounded_json(&captured, maximum, operation)
}

fn contract_bytes<'a>(
    records: impl Iterator<Item = &'a crate::records::RecordContract>,
    telemetry: impl Iterator<Item = &'a crate::telemetry::TelemetryContract>, maximum: usize, operation: &str,
) -> Result<Vec<u8>, Box<Failure>> {
    #[derive(serde::Serialize)]
    struct Contracts<'a> {
        records: Vec<&'a crate::records::RecordContract>,
        telemetry: Vec<&'a crate::telemetry::TelemetryContract>,
    }
    let mut captured = Contracts {
        records: Vec::new(),
        telemetry: Vec::new(),
    };
    let mut count = 0usize;
    for contract in records {
        count += 1;
        if count > maximum / std::mem::size_of::<usize>() {
            return Err(Failure::new(Reason::ResourceLimit, operation, "Captured contracts exceed the schema budget"));
        }
        captured.records.push(contract);
    }
    for contract in telemetry {
        count += 1;
        if count > maximum / std::mem::size_of::<usize>() {
            return Err(Failure::new(Reason::ResourceLimit, operation, "Captured contracts exceed the schema budget"));
        }
        captured.telemetry.push(contract);
    }
    tokens::bounded_json(&captured, maximum, operation)
}

fn validate_scope(scope: &Scope, operation: &str) -> Result<(), Box<Failure>> {
    let valid = match scope {
        Scope::History => true,
        Scope::Execution(id) => TestExecutionId::from_uuid(id).is_ok(),
        Scope::Cycle(id) => TestCycleId::from_uuid(id).is_ok(),
    };
    if valid { Ok(()) } else { Err(Failure::new(Reason::InvalidSelection, operation, "Invalid query scope identifier")) }
}

fn now() -> Timestamp {
    Timestamp::from(time::OffsetDateTime::now_utc())
}

fn expiry(lifetime: std::time::Duration, operation: &str) -> Result<Timestamp, Box<Failure>> {
    let duration = time::Duration::try_from(lifetime).map_err(|_| unavailable(operation))?;
    let expires = time::OffsetDateTime::now_utc().checked_add(duration).ok_or_else(|| unavailable(operation))?;
    Ok(Timestamp::from(expires))
}

fn checkpoint_coordinates(
    checkpoint: &dokime_domain::model::execution::wire_checkpoint::ExecutionCheckpoint, maximum: usize, operation: &str,
) -> Result<dokime_domain::model::execution::ExecutionCheckpoint, Box<Failure>> {
    if checkpoint.version.len() > maximum {
        return Err(Failure::new(Reason::ResourceLimit, operation, "Checkpoint version exceeds token budget"));
    }
    let native = dokime_domain::model::execution::ExecutionCheckpoint {
        execution_id: TestExecutionId::from_uuid(&checkpoint.execution_identifier).map_err(|_| unavailable(operation))?,
        position: cursor(checkpoint.position.as_ref(), operation)?,
        evidence_through: cursor(checkpoint.evidence_through.as_ref(), operation)?,
        version: ObjectVersionToken::decode_bounded(&checkpoint.version, maximum.try_into().expect("positive token budget"))
            .map_err(|_| unavailable(operation))?,
    };
    native.validate().map_err(|_| unavailable(operation))?;
    Ok(native)
}

fn cursor(
    value: Option<&dokime_domain::model::execution::wire_checkpoint::StorageCursor>, operation: &str,
) -> Result<StorageCursor, Box<Failure>> {
    let value = value.ok_or_else(|| unavailable(operation))?;
    let history = value.history.as_ref().ok_or_else(|| unavailable(operation))?;
    Ok(StorageCursor {
        history: StoreHistory {
            store: Identifier::new(&history.store).map_err(|_| unavailable(operation))?,
            epoch: Identifier::new(&history.epoch).map_err(|_| unavailable(operation))?,
        },
        sequence: value.sequence,
    })
}

fn unavailable(operation: &str) -> Box<Failure> {
    Failure::new(Reason::SnapshotUnavailable, operation, "The exact query snapshot is unavailable in this local store")
}

/* === Tests === */
