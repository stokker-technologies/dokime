/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Finite Arrow Query Planning

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use harmos::Message;

// Local Imports
use super::{
    selection::{Envelope, FieldRef, Selection, SelectionRequest},
    snapshots::{QuerySnapshot, Scope, Seek},
    tokens,
};
use crate::{
    RuntimeHandle,
    config::QueryConfig,
    storage::sqlite::{QueryRead, QueryWork, State},
};
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::{
        error::{Failure, Reason},
        page::{PageLimits, SamplePage, SampleRow},
    },
    telemetry::{TelemetryContract, TelemetrySample, wire},
    values::Value,
};
use dokime_operations::query::ListSamples;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Plan {
    pub read: QueryRead,
    pub selection: Arc<Selection>,
}

pub(super) struct RawPage {
    rows: BTreeMap<u64, (SampleRow, usize)>,
    bytes: usize,
    limits: PageLimits,
    memory: usize,
    truncated: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn query_list_samples(&self, request: ListSamples, message_bytes: usize) -> Result<SamplePage, Box<Failure>> {
        let operation = "dokime.query.list_samples";
        tokio::time::timeout(self.query_config().limits().active_time, async {
            request.validate_shape()?;
            if request.bucket.is_some() {
                return Err(Failure::new(Reason::InvalidSelection, operation, "Use the bucket executor for reduced pages"));
            }
            let limits = self.query_config().page_limits(&request.page, message_bytes, operation)?;
            let binding = tokens::binding(&request, self.query_config().limits().working_bytes as usize, operation)?;
            let snapshot = self
                .query_snapshot(
                    Scope::Execution(request.execution_id.clone()),
                    request.at.as_ref(),
                    request.page.after.as_deref(),
                    binding,
                    operation,
                )
                .await?;
            let contracts = self.query_snapshot_contracts(&snapshot, operation).await?;
            let plan = Plan::new(&contracts, &request, &snapshot, self.query_config(), operation)?;
            let reserve = self
                .query_continuation(
                    &snapshot,
                    binding,
                    Seek::Evidence {
                        position: u64::MAX,
                        occurrence: "f".repeat(36),
                    },
                    operation,
                )
                .await?;
            let mut response = SamplePage {
                rows: Vec::new(),
                schema: Some(plan.selection.schema().clone()),
                snapshot: Some(snapshot.snapshot.clone()),
                limits: Some(limits.clone()),
                next: Some(reserve),
                complete: false,
            };
            let budget = page_budget(&response, &limits, operation)?;
            let page = raw_page(&self.local_storage, plan, self.query_config().clone(), budget).await?;
            response.next = match page.next {
                Some((position, occurrence)) => {
                    Some(self.query_continuation(&snapshot, binding, Seek::Evidence { position, occurrence }, operation).await?)
                }
                None => None,
            };
            response.rows = page.rows;
            response.complete = response.next.is_none();
            check_page(&response, &limits, operation)?;
            Ok(response)
        })
        .await
        .map_err(|_| Failure::new(Reason::Deadline, operation, "Sample query exceeded its active-work deadline"))?
    }
}

impl Plan {
    pub fn new(
        bytes: &[u8], request: &ListSamples, snapshot: &QuerySnapshot, config: &QueryConfig, operation: &'static str,
    ) -> Result<Self, Box<Failure>> {
        #[derive(serde::Deserialize)]
        struct Contracts {
            telemetry: Vec<TelemetryContract>,
        }
        let captured: Contracts = serde_json::from_slice(bytes)
            .map_err(|_| Failure::new(Reason::Corruption, operation, "Captured telemetry contracts are invalid"))?;
        let selection = Selection::samples(
            &captured.telemetry,
            &request.stream,
            SelectionRequest {
                versions: &request.versions,
                fields: &request.fields,
                predicates: &request.r#where,
                between: request.between.as_ref(),
                operation,
            },
            config,
        )?;
        let after = match &snapshot.after {
            None => 0,
            Some(Seek::Evidence { position, .. }) if *position <= snapshot.cut.at.sequence => *position,
            _ => return Err(Failure::new(Reason::InvalidContinuation, operation, "Expected a sample position continuation")),
        };
        let fields = selection
            .required_fields()
            .iter()
            .filter_map(|field| match field {
                FieldRef::Declared(id) => Some(id.clone()),
                _ => None,
            })
            .collect();
        let read = QueryRead {
            cut: snapshot.scoped_cut(),
            execution: request.execution_id.clone(),
            stream: request.stream.clone(),
            versions: selection.schema().versions.clone(),
            fields,
            between: request.between.clone(),
            after,
        };
        Ok(Self { read, selection: Arc::new(selection) })
    }
}

impl RawPage {
    fn push(&mut self, position: u64, row: SampleRow, operation: &str) -> Result<Option<u64>, Box<Failure>> {
        let bytes = tokens::bounded_json(&row, self.limits.bytes as usize, operation)?.len().max(row.encoded_size()) + 16;
        let memory = harmos::MemorySize::memory_size(&row);
        if memory > self.memory || bytes > self.limits.bytes as usize {
            return Err(Failure::new(Reason::ResourceLimit, operation, "A selected sample exceeds the page or working-byte ceiling"));
        }
        self.bytes += bytes;
        self.rows.insert(position, (row, bytes));
        while self.rows.len() > self.limits.rows as usize
            || self.bytes > self.limits.bytes as usize
            || self.rows.values().map(|(row, _)| harmos::MemorySize::memory_size(row)).sum::<usize>() > self.memory
        {
            let (_, (_, bytes)) = self.rows.pop_last().expect("nonempty page");
            self.bytes -= bytes;
            self.truncated = true;
        }
        Ok(self.truncated.then(|| *self.rows.last_key_value().expect("one admitted row").0))
    }
}

/* === Public Functions === */

pub(super) fn matches(selection: &Selection, sample: &TelemetrySample) -> bool {
    let timestamp = sample.timestamp.to_datetime().ok().map(Timestamp::from);
    let received = sample.received_at.to_datetime().ok().map(Value::Datetime);
    let time = sample.timestamp.to_datetime().ok().map(Value::Datetime);
    let id = Value::Text(sample.sample_id.to_string());
    let source = Value::Text(sample.source.clone());
    let version = Value::Unsigned(sample.schema_version.into());
    selection.matches(timestamp, |field| match field {
        FieldRef::Declared(id) => sample.values.get(id),
        FieldRef::Envelope(envelope) => match envelope {
            Envelope::Id => Some(&id),
            Envelope::Timestamp => time.as_ref(),
            Envelope::ReceivedAt => received.as_ref(),
            Envelope::Source => Some(&source),
            Envelope::SchemaVersion => Some(&version),
            Envelope::Severity => None,
        },
    })
}

pub(super) fn page_budget(value: &SamplePage, limits: &PageLimits, operation: &str) -> Result<PageLimits, Box<Failure>> {
    let overhead = tokens::bounded_json(value, limits.bytes as usize, operation)?.len().max(value.encoded_size());
    let bytes = limits
        .bytes
        .checked_sub(overhead as u64 + u64::from(limits.rows))
        .filter(|bytes| *bytes > 0)
        .ok_or_else(|| Failure::new(Reason::ResourceLimit, operation, "Sample page metadata exceeds the effective byte ceiling"))?;
    Ok(PageLimits { rows: limits.rows, bytes })
}

pub(super) fn check_page(value: &(impl serde::Serialize + Message), limits: &PageLimits, operation: &str) -> Result<(), Box<Failure>> {
    tokens::bounded_json(value, limits.bytes as usize, operation)?;
    if value.encoded_size() as u64 > limits.bytes {
        return Err(Failure::new(Reason::ResourceLimit, operation, "Encoded query reply exceeds its byte ceiling"));
    }
    Ok(())
}

pub(super) struct RawResult {
    pub rows: Vec<SampleRow>,
    pub next: Option<(u64, String)>,
}

pub(super) async fn raw_page(local: &State, plan: Plan, config: QueryConfig, limits: PageLimits) -> Result<RawResult, Box<Failure>> {
    let operation = "dokime.query.list_samples";
    let output = RawPage {
        rows: BTreeMap::new(),
        bytes: 0,
        limits,
        memory: config.limits().working_bytes as usize / 4,
        truncated: false,
    };
    let selection = plan.selection;
    let (page, _, complete) = local
        .query_telemetry(plan.read, QueryWork::new(config, operation), output, move |page, position, mut sample| {
            if !matches(&selection, &sample) {
                return Ok(None);
            }
            sample.values.retain(|id, _| selection.schema().fields.iter().any(|field| field.id == *id));
            let row = SampleRow::Raw(Box::new(project(&sample, operation)?));
            page.push(position, row, operation)
        })
        .await?;
    let next = if !complete || page.truncated {
        page.rows.last_key_value().map(|(position, (row, _))| {
            let SampleRow::Raw(sample) = row else { unreachable!() };
            (*position, sample.sample_id.clone())
        })
    } else {
        None
    };
    Ok(RawResult {
        rows: page.rows.into_values().map(|(row, _)| row).collect(),
        next,
    })
}

pub(super) fn project(sample: &TelemetrySample, operation: &str) -> Result<wire::TelemetrySample, Box<Failure>> {
    use dokime_domain::model::{identity::wire::EvidenceScope, values::wire::Value as WireValue};
    let conversion = |_| Failure::new(Reason::ResourceLimit, operation, "Sample projection exceeds encoding limits");
    let scope = sample.scope.as_ref().map(|scope| EvidenceScope {
        session_id: scope.session_id.as_ref().map(ToString::to_string),
        cycle_id: scope.test_cycle_id.as_ref().map(ToString::to_string),
        case_id: scope.test_case_id.as_ref().map(ToString::to_string),
        execution_id: scope.test_execution_id.as_ref().map(ToString::to_string),
        step_id: scope.step_id.as_ref().map(ToString::to_string),
        criterion_id: scope.criterion_id.as_ref().map(ToString::to_string),
        template_id: scope.template_id.as_ref().map(ToString::to_string),
        suite_id: scope.suite_id.as_ref().map(ToString::to_string),
        origin: Some(dokime_domain::model::identity::wire::EvidenceOrigin {
            kind: Some(match &scope.origin {
                dokime_domain::model::execution::EvidenceOrigin::Default => {
                    dokime_domain::model::identity::wire::EvidenceOriginKind::Default(true)
                }
                dokime_domain::model::execution::EvidenceOrigin::Initialization => {
                    dokime_domain::model::identity::wire::EvidenceOriginKind::Initialization(true)
                }
                dokime_domain::model::execution::EvidenceOrigin::Step { attempt_id } => {
                    dokime_domain::model::identity::wire::EvidenceOriginKind::StepAttemptId(attempt_id.to_string())
                }
                dokime_domain::model::execution::EvidenceOrigin::Cleanup => {
                    dokime_domain::model::identity::wire::EvidenceOriginKind::Cleanup(true)
                }
            }),
        }),
    });
    Ok(wire::TelemetrySample {
        sample_id: sample.sample_id.to_string(),
        stream: sample.stream.to_string(),
        schema_version: sample.schema_version,
        sequence: sample.sequence,
        timestamp: Some(Timestamp::from(
            sample.timestamp.to_datetime().map_err(|_| Failure::new(Reason::Corruption, operation, "Invalid sample timestamp"))?,
        )),
        received_at: Some(Timestamp::from(
            sample.received_at.to_datetime().map_err(|_| Failure::new(Reason::Corruption, operation, "Invalid receive timestamp"))?,
        )),
        source: sample.source.clone(),
        scope,
        values: Some(WireValue::try_from(&sample.values).map_err(conversion)?),
        summary: Some(WireValue::try_from(&sample.summary).map_err(conversion)?),
        metadata: Some(WireValue::try_from(&sample.metadata).map_err(conversion)?),
    })
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::{
        identity::{Identifier, Metadata},
        query::error::Reason,
        telemetry::{TelemetryColumn, TelemetryRole},
        values::ValueType,
    };

    #[test]
    fn query_versions() {
        let column =
            |id: &str, kind| TelemetryColumn::new(Metadata::new(Identifier::new(id).unwrap(), id, None), kind, TelemetryRole::Value);
        let mut first = TelemetryContract::declared("query.readings", "Readings");
        first.columns = vec![column("counter", ValueType::Integer)];
        let mut second = first.clone();
        second.schema_version = 2;
        second.columns.push(column("extra", ValueType::Text));
        let config =
            crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        let cursor = harmos::storage::StorageCursor {
            history: harmos::storage::StoreHistory {
                store: Identifier::new("query.store").unwrap(),
                epoch: Identifier::new("query.epoch").unwrap(),
            },
            sequence: 100,
        };
        let snapshot = QuerySnapshot {
            snapshot: dokime_domain::model::query::snapshot::Snapshot {
                token: String::new(),
                identity: String::new(),
                expires_at: None,
            },
            cut: super::super::snapshots::Cut {
                source: "local".into(),
                backend: "query.backend".into(),
                scope: Scope::Execution("execution".into()),
                at: cursor,
                metadata: 100,
                journal: None,
                catalog: 1,
                retention: 0,
                expires: Timestamp { seconds: 100, nanos: 0 },
                contracts: String::new(),
            },
            after: None,
            scope: Scope::Execution("execution".into()),
        };
        let mut request = ListSamples {
            execution_id: "execution".into(),
            stream: "query.readings".into(),
            versions: Vec::new(),
            fields: vec!["extra".into()],
            r#where: vec![dokime_domain::model::query::predicate::Predicate {
                field: "counter".into(),
                operator: dokime_domain::model::query::predicate::Operator::Eq,
                values: vec![dokime_domain::model::query::predicate::Scalar::Unsigned(1)],
            }],
            between: None,
            bucket: None,
            page: dokime_domain::model::query::page::PageRequest { rows: None, bytes: None, after: None },
            at: None,
        };
        let prepare = |contracts: &[TelemetryContract], request: &ListSamples| {
            let bytes = serde_json::to_vec(&serde_json::json!({"telemetry": contracts})).unwrap();
            Plan::new(&bytes, request, &snapshot, &config, "test.query")
        };
        let plan = prepare(&[first.clone(), second.clone()], &request).unwrap();
        assert_eq!(plan.read.versions, vec![1, 2]);
        assert_eq!(plan.read.fields, vec!["counter", "extra"]);
        assert!(plan.selection.schema().fields[0].nullable);
        second.columns[0].value_type = ValueType::Number;
        assert_eq!(prepare(&[first.clone(), second.clone()], &request).err().unwrap().reason, Reason::IncompatibleSchema);
        request.versions = vec![1];
        request.fields = vec!["counter".into()];
        assert_eq!(prepare(&[first, second], &request).unwrap().read.versions, vec![1]);
    }
}
