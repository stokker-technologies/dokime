/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::{
    DefinitionId, Identified, MemorySize,
    storage::{FileReference, ObjectRetention, ObjectRevision, ObjectVersion, ObjectVersionToken, StorageCursor, StoredObject},
};

// Local Imports
use super::{annotations, artifacts, case, cycle, metadata, templates};
use crate::storage::{Error, Manager, Result, records::ScopeRow};
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */
macro_rules! native_map {
    ($name:ident, $entry:ident, $value:ty) => {
        #[harmos::message]
        #[derive(Clone, PartialEq)]
        struct $name {
            entries: Vec<$entry>,
        }
        #[harmos::message]
        #[derive(Clone, PartialEq)]
        struct $entry {
            key: String,
            value: Option<$value>,
        }
        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
                let mut values = BTreeMap::new();
                for entry in &self.entries {
                    let value = entry.value.as_ref().ok_or_else(|| serde::ser::Error::custom("native map member is missing"))?;
                    if values.insert(&entry.key, value).is_some() {
                        return Err(serde::ser::Error::custom("native map key is duplicated"));
                    }
                }
                values.serialize(serializer)
            }
        }
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
                let values = <BTreeMap<String, $value> as serde::Deserialize>::deserialize(deserializer)?;
                let entries = values.into_iter().map(|(key, value)| $entry { key, value: Some(value) }).collect();
                Ok(Self { entries })
            }
        }
    };
}
native_map!(Parameters, ParameterEntry, Parameter);
native_map!(Documents, DocumentEntry, templates::documents::Document);

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct PublicationCheck<'a> {
    pub source: &'a Revision,
    pub input: &'a FileReference,
    pub report: &'a FileReference,
    pub evidence: &'a StorageCursor,
    pub saved: &'a StorageCursor,
    pub at: &'a str,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    saved: Option<Seal>,
    test_cycle_id: String,
    test_execution_id: String,
    test_case_id: String,
    suite: Option<metadata::Row>,
    status: String,
    mode: String,
    snapshot: Option<Snapshot>,
    invalidation: Option<Invalidation>,
    scope: Option<ScopeRow>,
    output_parameters: Parameters,
    documents: Documents,
    records: Option<annotations::Set>,
    started_at: Option<String>,
    finished_at: Option<String>,
    steps: Vec<Step>,
    step_attempts: Vec<Attempt>,
    active_step_attempt: Option<AttemptStart>,
    capture_phase: String,
    traceability: Vec<Traceability>,
    result: Option<cycle::Summary>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    execution_id: String,
    position: Option<StorageCursor>,
    version: Option<ObjectVersionToken>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    owner_node_id: String,
    evidence_through: Option<StorageCursor>,
    execution: Option<Revision>,
    context: Option<FileReference>,
    outputs: Vec<Output>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    relative_path: String,
    file: Option<FileReference>,
    stream_state: Option<String>,
    content: Option<artifacts::Content>,
    created_at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Seal {
    source: Option<Source>,
    report: Option<FileReference>,
    dependencies_saved: Option<StorageCursor>,
    sealed_at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    case: Option<case::Case>,
    template: Option<templates::Template>,
    parameter_definitions: Vec<templates::Parameter>,
    plugin: Option<Plugin>,
    host_version: Option<String>,
    parameters: Parameters,
    documents: Documents,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Plugin {
    source: String,
    version: Option<String>,
    implementation: Option<Implementation>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Implementation {
    kind: String,
    content_digest: Option<String>,
    generation: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Invalidation {
    previous: Option<Plugin>,
    current: Option<Plugin>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    metadata: Option<metadata::Row>,
    group_id: Option<String>,
    source: String,
    value: Option<templates::Value>,
    unit: Option<templates::Unit>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    step_id: String,
    metadata: Option<metadata::Row>,
    status: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    measurements: Vec<Measurement>,
    criteria: Vec<Criterion>,
    detail: Option<Diagnostic>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptStart {
    attempt_id: String,
    step_id: String,
    started_at: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Attempt {
    attempt_id: String,
    step_id: String,
    started_at: String,
    finished_at: Option<String>,
    termination: String,
    detail: Option<Diagnostic>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Measurement {
    metadata: Option<metadata::Row>,
    value: Option<templates::Value>,
    unit: Option<templates::Unit>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Criterion {
    criterion_id: String,
    metadata: Option<metadata::Row>,
    status: String,
    observed: Option<templates::Value>,
    message: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Diagnostic {
    message: String,
    chain: Vec<String>,
    location: String,
    span_trace: Vec<String>,
    backtrace: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Traceability {
    identifier: String,
    label: String,
    rows: Vec<TraceabilityRow>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceabilityRow {
    identifier: String,
    label: String,
    value: Option<templates::Value>,
    unit: Option<String>,
    source: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Execution {
    pub fn is_sealed(&self) -> bool {
        self.saved.is_some()
    }

    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        Ok(<Self as harmos::Message>::decoding_memory(bytes)?)
    }
}

impl Execution {
    pub fn encoding_memory(value: &TestExecution) -> Result<usize> {
        for time in value.started_at.iter().chain(&value.finished_at) {
            time.to_datetime()?;
        }
        let fields = [
            value.saved.as_ref().map(Seal::encoding_memory).transpose()?.unwrap_or(0),
            value.suite.as_ref().map(metadata::Row::encoding_memory).transpose()?.unwrap_or(0),
            Snapshot::encoding_memory(&value.snapshot)?,
            value.invalidation.as_ref().map_or(0, Invalidation::encoding_memory),
            ScopeRow::encoding_memory(&value.scope),
            Parameters::encoding_memory(&value.output_parameters)?,
            Documents::encoding_memory(&value.documents)?,
            annotations::Set::encoding_memory(&value.records)?,
            value.active_step_attempt.as_ref().map(AttemptStart::encoding_memory).transpose()?.unwrap_or(0),
            value.result.as_ref().map_or(0, cycle::Summary::encoding_memory),
            value.test_cycle_id.as_str().len(),
            value.test_execution_id.as_str().len(),
            value.test_case_id.as_str().len(),
            execution_status_text(&value.status).len(),
            execution_mode_text(&value.mode).len(),
            capture_phase_text(&value.capture_phase).len(),
            value.started_at.as_ref().map_or(0, |time| time.as_str().len()),
            value.finished_at.as_ref().map_or(0, |time| time.as_str().len()),
            harmos::btree_map_memory::<&dokime_domain::model::identity::StepAttemptId, ()>(value.step_attempts.len()),
        ];
        if fields.contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for step in &value.steps {
            bytes = bytes.saturating_add(Step::encoding_memory(step)?);
        }
        for attempt in &value.step_attempts {
            bytes = bytes.saturating_add(Attempt::encoding_memory(attempt)?);
        }
        for section in &value.traceability {
            bytes = bytes.saturating_add(Traceability::encoding_memory(section)?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&TestExecution> for Execution {
    type Error = Error;
    fn try_from(value: &TestExecution) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("execution encoding memory overflows"));
        }
        validate_owner(value)?;
        validate_attempts(value)?;

        let mut steps = Vec::new();
        steps.try_reserve_exact(value.steps.len()).map_err(|_| Error::Invalid("execution steps encoding allocation failed"))?;
        for step in &value.steps {
            steps.push(Step::try_from(step)?);
        }
        let mut step_attempts = Vec::new();
        step_attempts
            .try_reserve_exact(value.step_attempts.len())
            .map_err(|_| Error::Invalid("execution attempts encoding allocation failed"))?;
        for attempt in &value.step_attempts {
            step_attempts.push(Attempt::try_from(attempt)?);
        }
        let mut traceability = Vec::new();
        traceability
            .try_reserve_exact(value.traceability.len())
            .map_err(|_| Error::Invalid("execution traceability encoding allocation failed"))?;
        for section in &value.traceability {
            traceability.push(Traceability::try_from(section)?);
        }
        Ok(Self {
            saved: value.saved.as_ref().map(Seal::try_from).transpose()?,
            test_cycle_id: value.test_cycle_id.as_str().to_owned(),
            test_execution_id: value.test_execution_id.as_str().to_owned(),
            test_case_id: value.test_case_id.as_str().to_owned(),
            suite: value.suite.as_ref().map(metadata::Row::try_from).transpose()?,
            status: execution_status_text(&value.status).to_owned(),
            mode: execution_mode_text(&value.mode).to_owned(),
            snapshot: Some(Snapshot::try_from(&value.snapshot)?),
            invalidation: value.invalidation.as_ref().map(Invalidation::try_from).transpose()?,
            scope: Some(ScopeRow::from(&value.scope)),
            output_parameters: Parameters::try_from(&value.output_parameters)?,
            documents: Documents::try_from(&value.documents)?,
            records: Some(annotations::Set::try_from(&value.records)?),
            started_at: value.started_at.as_ref().map(|time| time.as_str().to_owned()),
            finished_at: value.finished_at.as_ref().map(|time| time.as_str().to_owned()),
            steps,
            step_attempts,
            active_step_attempt: value.active_step_attempt.as_ref().map(AttemptStart::try_from).transpose()?,
            capture_phase: capture_phase_text(&value.capture_phase).to_owned(),
            traceability,
            result: value.result.as_ref().map(cycle::Summary::from),
        })
    }
}

impl Execution {
    pub fn projection_memory(&self) -> Result<usize> {
        execution_status(&self.status)?;
        execution_mode(&self.mode)?;
        capture_phase(&self.capture_phase)?;
        let snapshot = self.snapshot.as_ref().ok_or(Error::Invalid("execution snapshot is missing"))?.projection_memory()?;
        let scope = self.scope.as_ref().ok_or(Error::Invalid("execution scope is missing"))?.projection_memory();
        let records = self.records.as_ref().ok_or(Error::Invalid("execution records are missing"))?.projection_memory();
        let fields = [
            self.saved.as_ref().map(Seal::projection_memory).transpose()?.unwrap_or(0),
            self.suite.as_ref().map(metadata::Row::projection_memory).transpose()?.unwrap_or(0),
            snapshot,
            scope,
            records,
            self.invalidation.as_ref().map(Invalidation::projection_memory).transpose()?.unwrap_or(0),
            self.output_parameters.projection_memory()?,
            self.documents.projection_memory()?,
            self.active_step_attempt.as_ref().map(AttemptStart::projection_memory).unwrap_or(0),
            self.test_cycle_id.capacity(),
            self.test_execution_id.capacity(),
            self.test_case_id.capacity(),
            self.started_at.as_ref().map_or(0, String::capacity),
            self.finished_at.as_ref().map_or(0, String::capacity),
        ];
        let mut bytes = fields
            .into_iter()
            .fold(std::mem::size_of::<TestExecution>(), usize::saturating_add)
            .saturating_add(harmos::btree_map_memory::<&dokime_domain::model::identity::StepAttemptId, ()>(self.step_attempts.len()));
        for step in &self.steps {
            bytes = bytes.saturating_add(step.projection_memory()?);
        }
        for attempt in &self.step_attempts {
            bytes = bytes.saturating_add(attempt.projection_memory());
        }
        for traceability in &self.traceability {
            bytes = bytes.saturating_add(traceability.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<Execution> for TestExecution {
    type Error = Error;

    fn try_from(row: Execution) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution projection memory overflows"));
        }
        if [&row.test_cycle_id, &row.test_execution_id].into_iter().any(|id| id.bytes().any(|byte| byte.is_ascii_uppercase())) {
            return Err(Error::Invalid("noncanonical execution or cycle identity"));
        }
        let mut steps = Vec::new();
        steps.try_reserve_exact(row.steps.len()).map_err(|_| Error::Invalid("execution steps allocation failed"))?;
        for step in row.steps {
            steps.push(step.try_into()?);
        }
        let mut step_attempts = Vec::new();
        step_attempts.try_reserve_exact(row.step_attempts.len()).map_err(|_| Error::Invalid("execution attempts allocation failed"))?;
        for attempt in row.step_attempts {
            step_attempts.push(attempt.try_into()?);
        }
        let mut traceability = Vec::new();
        traceability.try_reserve_exact(row.traceability.len()).map_err(|_| Error::Invalid("execution traceability allocation failed"))?;
        for section in row.traceability {
            traceability.push(section.try_into()?);
        }
        let started_at = row.started_at.map(dokime_domain::model::time::Timestamp::new);
        let finished_at = row.finished_at.map(dokime_domain::model::time::Timestamp::new);
        for at in [&started_at, &finished_at].into_iter().flatten() {
            at.to_datetime()?;
        }
        let execution = Self {
            saved: row.saved.map(TryInto::try_into).transpose()?,
            mode: execution_mode(&row.mode)?,
            test_cycle_id: dokime_domain::model::identity::TestCycleId::from_uuid(row.test_cycle_id)?,
            test_execution_id: TestExecutionId::from_uuid(row.test_execution_id)?,
            test_case_id: dokime_domain::model::identity::CaseId::from_wire(row.test_case_id)?,
            suite: row.suite.map(TryInto::try_into).transpose()?,
            status: execution_status(&row.status)?,
            snapshot: row.snapshot.ok_or(Error::Invalid("execution snapshot is missing"))?.try_into()?,
            invalidation: row.invalidation.map(TryInto::try_into).transpose()?,
            scope: row.scope.ok_or(Error::Invalid("execution scope is missing"))?.try_into()?,
            output_parameters: row.output_parameters.try_into()?,
            documents: row.documents.try_into()?,
            records: row.records.ok_or(Error::Invalid("execution records are missing"))?.restore_owned()?,
            started_at,
            finished_at,
            steps,
            step_attempts,
            active_step_attempt: row.active_step_attempt.map(TryInto::try_into).transpose()?,
            capture_phase: capture_phase(&row.capture_phase)?,
            traceability,
            result: row.result.map(TryInto::try_into).transpose()?,
        };
        // The Original Case Codec Has Already Consumed And Validated The Case.
        validate_owner(&execution)?;
        validate_attempts(&execution)?;
        Ok(execution)
    }
}

impl Documents {
    fn encoding_memory(value: &BTreeMap<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition>) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>().saturating_add(value.len().saturating_mul(std::mem::size_of::<DocumentEntry>()));
        for (key, document) in value {
            let memory = templates::documents::Document::encoding_memory(document)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(key.as_str().len())
                .saturating_add(memory.saturating_sub(std::mem::size_of::<templates::documents::Document>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<&BTreeMap<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition>> for Documents {
    type Error = Error;
    fn try_from(value: &BTreeMap<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition>) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("document map encoding memory overflows"));
        }
        let mut entries = Vec::new();
        entries.try_reserve_exact(value.len()).map_err(|_| Error::Invalid("document entries allocation failed"))?;
        for (key, document) in value {
            entries.push(DocumentEntry { key: key.as_str().to_owned(), value: Some(templates::documents::Document::try_from(document)?) });
        }
        Ok(Self { entries })
    }
}

impl Documents {
    fn projection_memory(&self) -> Result<usize> {
        if self.entries.windows(2).any(|pair| pair[0].key >= pair[1].key) {
            return Err(Error::Invalid("noncanonical document map order"));
        }
        let mut bytes =
            harmos::btree_map_memory::<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition>(self.entries.len());
        for entry in &self.entries {
            let memory = entry.value.as_ref().ok_or(Error::Invalid("document map member is missing"))?.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(entry.key.capacity())
                .saturating_add(memory.saturating_sub(std::mem::size_of::<crate::document::DocumentDefinition>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<Documents> for BTreeMap<dokime_domain::model::identity::DocumentId, crate::document::DocumentDefinition> {
    type Error = Error;

    fn try_from(row: Documents) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("document map projection memory overflows"));
        }
        let mut documents = BTreeMap::new();
        for entry in row.entries {
            let key = dokime_domain::model::identity::DocumentId::from_wire(entry.key)?;
            let value = entry.value.ok_or(Error::Invalid("document map member is missing"))?.try_into()?;
            documents.insert(key, value);
        }
        Ok(documents)
    }
}

impl Snapshot {
    fn encoding_memory(value: &dokime_domain::model::execution::snapshot::ExecutionSnapshot) -> Result<usize> {
        let fields = [
            case::Case::encoding_memory(value.case())?,
            value.template().map(templates::Template::encoding_memory).transpose()?.unwrap_or(0),
            value.plugin().map_or(0, Plugin::encoding_memory),
            value.host_version().map_or(0, str::len),
            Parameters::encoding_memory(value.parameters())?,
            Documents::encoding_memory(value.documents())?,
        ];
        if fields.contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for definition in value.parameter_definitions() {
            bytes = bytes.saturating_add(templates::Parameter::encoding_memory(definition)?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&dokime_domain::model::execution::snapshot::ExecutionSnapshot> for Snapshot {
    type Error = Error;
    fn try_from(value: &dokime_domain::model::execution::snapshot::ExecutionSnapshot) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("snapshot encoding memory overflows"));
        }
        let mut parameter_definitions = Vec::new();
        parameter_definitions
            .try_reserve_exact(value.parameter_definitions().len())
            .map_err(|_| Error::Invalid("snapshot definitions encoding allocation failed"))?;
        for definition in value.parameter_definitions() {
            parameter_definitions.push(templates::Parameter::try_from(definition)?);
        }
        Ok(Self {
            case: Some(case::Case::try_from(value.case())?),
            template: value.template().map(templates::Template::try_from).transpose()?,
            parameter_definitions,
            plugin: value.plugin().map(Plugin::try_from).transpose()?,
            host_version: value.host_version().map(str::to_owned),
            parameters: Parameters::try_from(value.parameters())?,
            documents: Documents::try_from(value.documents())?,
        })
    }
}

impl Snapshot {
    fn projection_memory(&self) -> Result<usize> {
        let case = self.case.as_ref().ok_or(Error::Invalid("execution snapshot case is missing"))?.projection_memory()?;
        let template = self.template.as_ref().map(templates::Template::projection_memory).transpose()?.unwrap_or(0);
        let plugin = self.plugin.as_ref().map(Plugin::projection_memory).transpose()?.unwrap_or(0);
        let mut bytes = std::mem::size_of::<dokime_domain::model::execution::snapshot::ExecutionSnapshot>()
            .saturating_add(case)
            .saturating_add(template)
            .saturating_add(plugin)
            .saturating_add(self.host_version.as_ref().map_or(0, String::capacity))
            .saturating_add(self.parameters.projection_memory()?)
            .saturating_add(self.documents.projection_memory()?);
        for definition in &self.parameter_definitions {
            bytes = bytes.saturating_add(definition.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<Snapshot> for dokime_domain::model::execution::snapshot::ExecutionSnapshot {
    type Error = Error;

    fn try_from(row: Snapshot) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution snapshot projection memory overflows"));
        }
        let mut definitions = Vec::new();
        definitions
            .try_reserve_exact(row.parameter_definitions.len())
            .map_err(|_| Error::Invalid("snapshot parameter definitions allocation failed"))?;
        for definition in row.parameter_definitions {
            definitions.push(definition.try_into()?);
        }
        let case = row.case.ok_or(Error::Invalid("execution snapshot case is missing"))?.try_into()?;
        let template = row.template.map(TryInto::try_into).transpose()?;
        let plugin = row.plugin.map(TryInto::try_into).transpose()?;
        let parameters = row.parameters.try_into()?;
        let documents = row.documents.try_into()?;
        Ok(Self::from_captured(case, template, definitions, plugin, row.host_version, parameters, documents))
    }
}

impl AttemptStart {
    fn encoding_memory(value: &dokime_domain::model::execution::result::StepAttemptStart) -> Result<usize> {
        value.started_at.to_datetime()?;
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.attempt_id.as_str().len())
            .saturating_add(value.step_id.as_str().len())
            .saturating_add(value.started_at.as_str().len()))
    }

    fn projection_memory(&self) -> usize {
        std::mem::size_of::<dokime_domain::model::execution::result::StepAttemptStart>()
            .saturating_add(self.attempt_id.capacity())
            .saturating_add(self.step_id.capacity())
            .saturating_add(self.started_at.capacity())
    }
}

impl TryFrom<&dokime_domain::model::execution::result::StepAttemptStart> for AttemptStart {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::StepAttemptStart) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("step admission encoding memory overflows"));
        }
        Ok(Self {
            attempt_id: value.attempt_id.as_str().to_owned(),
            step_id: value.step_id.as_str().to_owned(),
            started_at: value.started_at.as_str().to_owned(),
        })
    }
}

impl TryFrom<AttemptStart> for dokime_domain::model::execution::result::StepAttemptStart {
    type Error = Error;
    fn try_from(row: AttemptStart) -> Result<Self> {
        if row.projection_memory() == usize::MAX {
            return Err(Error::Invalid("step admission projection memory overflows"));
        }
        let started_at = dokime_domain::model::time::Timestamp::new(row.started_at);
        started_at.to_datetime()?;
        Ok(Self {
            attempt_id: dokime_domain::model::identity::StepAttemptId::from_uuid(row.attempt_id)?,
            step_id: dokime_domain::model::identity::StepId::from_uuid(row.step_id)?,
            started_at,
        })
    }
}

impl Attempt {
    fn encoding_memory(value: &dokime_domain::model::execution::result::TestStepAttempt) -> Result<usize> {
        value.started_at.to_datetime()?;
        if let Some(finished) = &value.finished_at {
            finished.to_datetime()?;
        }
        let detail = value.detail.as_ref().map_or(0, Diagnostic::encoding_memory);
        if detail == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            value.attempt_id.as_str().len(),
            value.step_id.as_str().len(),
            value.started_at.as_str().len(),
            value.finished_at.as_ref().map_or(0, |at| at.as_str().len()),
            termination_reason(&value.termination).len(),
            detail.saturating_sub(std::mem::size_of::<Diagnostic>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    fn projection_memory(&self) -> usize {
        use harmos::MemorySize;
        let detail = self.detail.as_ref().map_or(0, MemorySize::memory_size);
        if detail == usize::MAX {
            return usize::MAX;
        }
        let fields = [
            self.attempt_id.capacity(),
            self.step_id.capacity(),
            self.started_at.capacity(),
            self.finished_at.as_ref().map_or(0, String::capacity),
            detail.saturating_sub(std::mem::size_of::<crate::diagnostics::Diagnostic>()),
        ];
        fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::execution::result::TestStepAttempt>(), usize::saturating_add)
    }
}

impl TryFrom<&dokime_domain::model::execution::result::TestStepAttempt> for Attempt {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::TestStepAttempt) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("step result encoding memory overflows"));
        }
        Ok(Self {
            attempt_id: value.attempt_id.as_str().to_owned(),
            step_id: value.step_id.as_str().to_owned(),
            started_at: value.started_at.as_str().to_owned(),
            finished_at: value.finished_at.as_ref().map(|at| at.as_str().to_owned()),
            termination: termination_reason(&value.termination).to_owned(),
            detail: value.detail.as_ref().map(Diagnostic::try_from).transpose()?,
        })
    }
}

impl TryFrom<Attempt> for dokime_domain::model::execution::result::TestStepAttempt {
    type Error = Error;
    fn try_from(row: Attempt) -> Result<Self> {
        use dokime_domain::model::execution::termination::TerminationReason;
        if row.projection_memory() == usize::MAX {
            return Err(Error::Invalid("step result projection memory overflows"));
        }
        let termination = match row.termination.as_str() {
            "completed" => TerminationReason::Completed,
            "failed" => TerminationReason::Failed,
            "cancelled" => TerminationReason::Cancelled,
            "interrupted" => TerminationReason::Interrupted,
            _ => return Err(Error::Invalid("invalid step termination")),
        };
        let started_at = dokime_domain::model::time::Timestamp::new(row.started_at);
        let finished_at = row.finished_at.map(dokime_domain::model::time::Timestamp::new);
        started_at.to_datetime()?;
        if let Some(finished) = &finished_at {
            finished.to_datetime()?;
        }
        Ok(Self {
            attempt_id: dokime_domain::model::identity::StepAttemptId::from_uuid(row.attempt_id)?,
            step_id: dokime_domain::model::identity::StepId::from_uuid(row.step_id)?,
            started_at,
            finished_at,
            termination,
            detail: row.detail.map(Into::into),
        })
    }
}

impl TraceabilityRow {
    fn encoding_memory(value: &dokime_domain::model::traceability::TraceabilityRow) -> Result<usize> {
        let scalar = templates::Value::encoding_memory(&value.value)?;
        if scalar == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            value.identifier.as_str().len(),
            value.label.len(),
            value.unit.as_deref().map_or(0, str::len),
            value.source.as_deref().map_or(0, str::len),
            scalar.saturating_sub(std::mem::size_of::<templates::Value>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    fn projection_memory(&self) -> Result<usize> {
        let value = self.value.as_ref().ok_or(Error::Invalid("traceability value is missing"))?.projection_memory()?;
        if value == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            self.identifier.capacity(),
            self.label.capacity(),
            self.unit.as_ref().map_or(0, String::capacity),
            self.source.as_ref().map_or(0, String::capacity),
            value.saturating_sub(std::mem::size_of::<dokime_domain::model::values::Value>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::traceability::TraceabilityRow>(), usize::saturating_add))
    }
}

impl TryFrom<&dokime_domain::model::traceability::TraceabilityRow> for TraceabilityRow {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::traceability::TraceabilityRow) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("traceability row encoding memory overflows"));
        }
        Ok(Self {
            identifier: value.identifier.as_str().to_owned(),
            label: value.label.as_str().to_owned(),
            value: Some(templates::Value::try_from(&value.value)?),
            unit: value.unit.as_deref().map(str::to_owned),
            source: value.source.as_deref().map(str::to_owned),
        })
    }
}

impl TryFrom<TraceabilityRow> for dokime_domain::model::traceability::TraceabilityRow {
    type Error = Error;
    fn try_from(row: TraceabilityRow) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("traceability row projection memory overflows"));
        }
        Ok(Self {
            identifier: dokime_domain::model::identity::Identifier::new(row.identifier)?,
            label: row.label,
            value: row.value.ok_or(Error::Invalid("traceability value is missing"))?.try_into()?,
            unit: row.unit,
            source: row.source,
        })
    }
}

impl Traceability {
    fn encoding_memory(value: &dokime_domain::model::traceability::TraceabilitySection) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.identifier.as_str().len())
            .saturating_add(value.label.len())
            .saturating_add(value.rows.len().saturating_mul(std::mem::size_of::<TraceabilityRow>()));
        for row in &value.rows {
            let memory = TraceabilityRow::encoding_memory(row)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<TraceabilityRow>()));
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        let mut bytes = std::mem::size_of::<dokime_domain::model::traceability::TraceabilitySection>()
            .saturating_add(self.identifier.capacity())
            .saturating_add(self.label.capacity())
            .saturating_add(self.rows.len().saturating_mul(std::mem::size_of::<dokime_domain::model::traceability::TraceabilityRow>()));
        for row in &self.rows {
            let memory = row.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<dokime_domain::model::traceability::TraceabilityRow>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<&dokime_domain::model::traceability::TraceabilitySection> for Traceability {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::traceability::TraceabilitySection) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("traceability section encoding memory overflows"));
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(value.rows.len()).map_err(|_| Error::Invalid("traceability rows allocation failed"))?;
        for row in &value.rows {
            rows.push(TraceabilityRow::try_from(row)?);
        }
        Ok(Self { identifier: value.identifier.as_str().to_owned(), label: value.label.as_str().to_owned(), rows })
    }
}

impl TryFrom<Traceability> for dokime_domain::model::traceability::TraceabilitySection {
    type Error = Error;
    fn try_from(row: Traceability) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("traceability section projection memory overflows"));
        }
        let mut rows = Vec::with_capacity(row.rows.len());
        for row in row.rows {
            rows.push(row.try_into()?);
        }
        Ok(Self { identifier: dokime_domain::model::identity::Identifier::new(row.identifier)?, label: row.label, rows })
    }
}

impl Parameters {
    fn encoding_memory(value: &BTreeMap<String, crate::definition::MaterializedParameter>) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>().saturating_add(value.len().saturating_mul(std::mem::size_of::<ParameterEntry>()));
        for (key, parameter) in value {
            let memory = Parameter::encoding_memory(parameter)?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(key.len()).saturating_add(memory.saturating_sub(std::mem::size_of::<Parameter>()));
        }
        Ok(bytes)
    }

    fn projection_memory(&self) -> Result<usize> {
        if self.entries.windows(2).any(|pair| pair[0].key >= pair[1].key) {
            return Err(Error::Invalid("noncanonical parameter map order"));
        }
        let mut bytes = harmos::btree_map_memory::<String, crate::definition::MaterializedParameter>(self.entries.len());
        for entry in &self.entries {
            let memory = entry.value.as_ref().ok_or(Error::Invalid("parameter map member is missing"))?.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(entry.key.capacity())
                .saturating_add(memory.saturating_sub(std::mem::size_of::<crate::definition::MaterializedParameter>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<&BTreeMap<String, crate::definition::MaterializedParameter>> for Parameters {
    type Error = Error;

    fn try_from(value: &BTreeMap<String, crate::definition::MaterializedParameter>) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter map encoding memory overflows"));
        }
        let mut entries = Vec::new();
        entries.try_reserve_exact(value.len()).map_err(|_| Error::Invalid("parameter entries allocation failed"))?;
        for (key, parameter) in value {
            entries.push(ParameterEntry { key: key.as_str().to_owned(), value: Some(Parameter::try_from(parameter)?) });
        }
        Ok(Self { entries })
    }
}

impl TryFrom<Parameters> for BTreeMap<String, crate::definition::MaterializedParameter> {
    type Error = Error;
    fn try_from(row: Parameters) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("parameter map projection memory overflows"));
        }
        let mut parameters = BTreeMap::new();
        for entry in row.entries {
            let value = entry.value.ok_or(Error::Invalid("parameter map member is missing"))?.try_into()?;
            parameters.insert(entry.key, value);
        }
        Ok(parameters)
    }
}

impl Implementation {
    fn encoding_memory(value: &dokime_domain::model::execution::snapshot::ExecutionImplementation) -> usize {
        use dokime_domain::model::execution::snapshot::ExecutionImplementation;
        let (kind, identity) = match value {
            ExecutionImplementation::Bundle { content_digest } => ("bundle", content_digest),
            ExecutionImplementation::Embedded { generation } => ("embedded", generation),
        };
        std::mem::size_of::<Self>().saturating_add(kind.len()).saturating_add(identity.len())
    }
}

impl TryFrom<&dokime_domain::model::execution::snapshot::ExecutionImplementation> for Implementation {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::snapshot::ExecutionImplementation) -> Result<Self> {
        use dokime_domain::model::execution::snapshot::ExecutionImplementation;
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("implementation encoding memory overflows"));
        }
        Ok(match value {
            ExecutionImplementation::Bundle { content_digest } => {
                Self { kind: "bundle".into(), content_digest: Some(content_digest.as_str().to_owned()), generation: None }
            }
            ExecutionImplementation::Embedded { generation } => {
                Self { kind: "embedded".into(), content_digest: None, generation: Some(generation.as_str().to_owned()) }
            }
        })
    }
}

impl TryFrom<Implementation> for dokime_domain::model::execution::snapshot::ExecutionImplementation {
    type Error = Error;
    fn try_from(row: Implementation) -> Result<Self> {
        match (row.kind.as_str(), row.content_digest, row.generation) {
            ("bundle", Some(content_digest), None) => Ok(Self::Bundle { content_digest }),
            ("embedded", None, Some(generation)) => Ok(Self::Embedded { generation }),
            _ => Err(Error::Invalid("invalid execution implementation identity")),
        }
    }
}

impl Plugin {
    fn encoding_memory(value: &dokime_domain::model::execution::snapshot::ExecutionPluginIdentity) -> usize {
        let implementation = Implementation::encoding_memory(&value.implementation);
        if implementation == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(value.source.len())
            .saturating_add(value.version.as_deref().map_or(0, str::len))
            .saturating_add(implementation.saturating_sub(std::mem::size_of::<Implementation>()))
    }

    fn projection_memory(&self) -> Result<usize> {
        let implementation = self.implementation.as_ref().ok_or(Error::Invalid("execution implementation is missing"))?;
        Ok(std::mem::size_of::<dokime_domain::model::execution::snapshot::ExecutionPluginIdentity>()
            .saturating_add(self.source.capacity())
            .saturating_add(self.version.as_ref().map_or(0, String::capacity))
            .saturating_add(implementation.content_digest.as_ref().map_or(0, String::capacity))
            .saturating_add(implementation.generation.as_ref().map_or(0, String::capacity)))
    }
}

impl TryFrom<&dokime_domain::model::execution::snapshot::ExecutionPluginIdentity> for Plugin {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::snapshot::ExecutionPluginIdentity) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("execution plugin encoding memory overflows"));
        }
        Ok(Self {
            source: value.source.as_str().to_owned(),
            version: value.version.as_deref().map(str::to_owned),
            implementation: Some(Implementation::try_from(&value.implementation)?),
        })
    }
}

impl TryFrom<Plugin> for dokime_domain::model::execution::snapshot::ExecutionPluginIdentity {
    type Error = Error;
    fn try_from(row: Plugin) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution plugin projection memory overflows"));
        }
        Ok(Self {
            source: row.source,
            version: row.version,
            implementation: row.implementation.ok_or(Error::Invalid("execution implementation is missing"))?.try_into()?,
        })
    }
}

impl Invalidation {
    fn encoding_memory(value: &dokime_domain::model::execution::result::ExecutionInvalidation) -> usize {
        let previous = Plugin::encoding_memory(&value.previous);
        let current = value.current.as_ref().map_or(0, Plugin::encoding_memory);
        if [previous, current].contains(&usize::MAX) {
            return usize::MAX;
        }
        std::mem::size_of::<Self>()
            .saturating_add(previous.saturating_sub(std::mem::size_of::<Plugin>()))
            .saturating_add(current.saturating_sub(std::mem::size_of::<Plugin>()))
    }

    fn projection_memory(&self) -> Result<usize> {
        let previous = self.previous.as_ref().ok_or(Error::Invalid("execution invalidation provenance is missing"))?.projection_memory()?;
        let current = self.current.as_ref().map(Plugin::projection_memory).transpose()?.unwrap_or(0);
        if [previous, current].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<dokime_domain::model::execution::result::ExecutionInvalidation>()
            .saturating_add(
                previous.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::snapshot::ExecutionPluginIdentity>()),
            )
            .saturating_add(
                current.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::snapshot::ExecutionPluginIdentity>()),
            ))
    }
}

impl TryFrom<&dokime_domain::model::execution::result::ExecutionInvalidation> for Invalidation {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::ExecutionInvalidation) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("invalidation encoding memory overflows"));
        }
        Ok(Self { previous: Some(Plugin::try_from(&value.previous)?), current: value.current.as_ref().map(Plugin::try_from).transpose()? })
    }
}

impl TryFrom<Invalidation> for dokime_domain::model::execution::result::ExecutionInvalidation {
    type Error = Error;
    fn try_from(row: Invalidation) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("invalidation projection memory overflows"));
        }
        Ok(Self {
            previous: row.previous.ok_or(Error::Invalid("execution invalidation provenance is missing"))?.try_into()?,
            current: row.current.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Diagnostic {
    fn encoding_memory(value: &crate::diagnostics::Diagnostic) -> usize {
        let fields = [
            value.message.len(),
            value.location.len(),
            value.backtrace.len(),
            value.chain.len().saturating_mul(std::mem::size_of::<String>()),
            value.span_trace.len().saturating_mul(std::mem::size_of::<String>()),
        ];
        value
            .chain
            .iter()
            .chain(&value.span_trace)
            .fold(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add), |bytes, text| {
                bytes.saturating_add(text.len())
            })
    }
}

impl TryFrom<&crate::diagnostics::Diagnostic> for Diagnostic {
    type Error = Error;

    fn try_from(value: &crate::diagnostics::Diagnostic) -> Result<Self> {
        if Self::encoding_memory(value) == usize::MAX {
            return Err(Error::Invalid("diagnostic encoding memory overflow"));
        }
        Ok(Self {
            message: value.message.as_str().to_owned(),
            chain: copied_strings(&value.chain)?,
            location: value.location.as_str().to_owned(),
            span_trace: copied_strings(&value.span_trace)?,
            backtrace: value.backtrace.as_str().to_owned(),
        })
    }
}

impl From<Diagnostic> for crate::diagnostics::Diagnostic {
    fn from(row: Diagnostic) -> Self {
        Self { message: row.message, chain: row.chain, location: row.location, span_trace: row.span_trace, backtrace: row.backtrace }
    }
}

impl Measurement {
    fn encoding_memory(value: &dokime_domain::model::execution::result::MeasurementRecord) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let scalar = templates::Value::encoding_memory(&value.value)?;
        let unit = value.unit.as_ref().map(templates::Unit::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, scalar, unit].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(metadata.saturating_sub(std::mem::size_of::<metadata::Row>()))
            .saturating_add(scalar.saturating_sub(std::mem::size_of::<templates::Value>()))
            .saturating_add(unit.saturating_sub(std::mem::size_of::<templates::Unit>())))
    }
}

impl TryFrom<&dokime_domain::model::execution::result::MeasurementRecord> for Measurement {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::MeasurementRecord) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("measurement encoding memory overflow"));
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            value: Some(templates::Value::try_from(&value.value)?),
            unit: value.unit.as_ref().map(templates::Unit::try_from).transpose()?,
        })
    }
}

impl Measurement {
    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("measurement metadata is missing"))?.projection_memory()?;
        let value = self.value.as_ref().ok_or(Error::Invalid("measurement value is missing"))?.projection_memory()?;
        let unit = self.unit.as_ref().map(templates::Unit::projection_memory).transpose()?.unwrap_or(0);
        if [metadata, value, unit].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()),
            value.saturating_sub(std::mem::size_of::<dokime_domain::model::values::Value>()),
            unit.saturating_sub(std::mem::size_of::<dokime_domain::model::values::MeasurementUnit>()),
        ];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<dokime_domain::model::execution::result::MeasurementRecord>(), usize::saturating_add))
    }
}

impl TryFrom<Measurement> for dokime_domain::model::execution::result::MeasurementRecord {
    type Error = Error;
    fn try_from(row: Measurement) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("measurement projection memory overflows"));
        }
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("measurement metadata is missing"))?.try_into()?,
            value: row.value.ok_or(Error::Invalid("measurement value is missing"))?.try_into()?,
            unit: row.unit.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Criterion {
    fn encoding_memory(value: &dokime_domain::model::execution::result::CriterionResult) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let observed = value.observed.as_ref().map(templates::Value::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, observed].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            value.criterion_id.as_str().len(),
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            criterion_status_text(&value.status).len(),
            observed.saturating_sub(std::mem::size_of::<templates::Value>()),
            value.message.as_deref().map_or(0, str::len),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }
}

impl TryFrom<&dokime_domain::model::execution::result::CriterionResult> for Criterion {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::CriterionResult) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("criterion encoding memory overflow"));
        }
        Ok(Self {
            criterion_id: value.criterion_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            status: criterion_status_text(&value.status).to_owned(),
            observed: value.observed.as_ref().map(templates::Value::try_from).transpose()?,
            message: value.message.as_deref().map(str::to_owned),
        })
    }
}

impl Criterion {
    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("criterion metadata is missing"))?.projection_memory()?;
        let observed = self.observed.as_ref().map(templates::Value::projection_memory).transpose()?.unwrap_or(0);
        if [metadata, observed].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.criterion_id.capacity(),
            self.message.as_ref().map_or(0, String::capacity),
            metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()),
            observed.saturating_sub(std::mem::size_of::<dokime_domain::model::values::Value>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::execution::result::CriterionResult>(), usize::saturating_add))
    }
}

impl TryFrom<Criterion> for dokime_domain::model::execution::result::CriterionResult {
    type Error = Error;
    fn try_from(row: Criterion) -> Result<Self> {
        use dokime_domain::model::execution::result::CriterionStatus;
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("criterion projection memory overflows"));
        }
        let status = match row.status.as_str() {
            "pending" => CriterionStatus::Pending,
            "passed" => CriterionStatus::Passed,
            "failed" => CriterionStatus::Failed,
            "indeterminate" => CriterionStatus::Indeterminate,
            "skipped" => CriterionStatus::Skipped,
            _ => return Err(Error::Invalid("invalid criterion status")),
        };
        Ok(Self {
            criterion_id: dokime_domain::model::identity::CriterionId::from_uuid(row.criterion_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("criterion metadata is missing"))?.try_into()?,
            status,
            observed: row.observed.map(TryInto::try_into).transpose()?,
            message: row.message,
        })
    }
}

impl Step {
    fn encoding_memory(value: &dokime_domain::model::execution::result::StepRecord) -> Result<usize> {
        for time in value.started_at.iter().chain(&value.finished_at) {
            time.to_datetime()?;
        }
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let detail = value.detail.as_ref().map_or(0, Diagnostic::encoding_memory);
        if [metadata, detail].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            value.step_id.as_str().len(),
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            step_status_text(&value.status).len(),
            value.started_at.as_ref().map_or(0, |time| time.as_str().len()),
            value.finished_at.as_ref().map_or(0, |time| time.as_str().len()),
            detail.saturating_sub(std::mem::size_of::<Diagnostic>()),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for measurement in &value.measurements {
            let bound = Measurement::encoding_memory(measurement)?;
            if bound == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(bound);
        }
        for criterion in &value.criteria {
            let bound = Criterion::encoding_memory(criterion)?;
            if bound == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(bound);
        }
        Ok(bytes)
    }
}

impl TryFrom<&dokime_domain::model::execution::result::StepRecord> for Step {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::result::StepRecord) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("step encoding memory overflow"));
        }
        let mut measurements = Vec::new();
        measurements.try_reserve_exact(value.measurements.len()).map_err(|_| Error::Invalid("measurement encoding allocation failed"))?;
        for measurement in &value.measurements {
            measurements.push(Measurement::try_from(measurement)?);
        }
        let mut criteria = Vec::new();
        criteria.try_reserve_exact(value.criteria.len()).map_err(|_| Error::Invalid("criterion encoding allocation failed"))?;
        for criterion in &value.criteria {
            criteria.push(Criterion::try_from(criterion)?);
        }
        Ok(Self {
            step_id: value.step_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            status: step_status_text(&value.status).to_owned(),
            started_at: value.started_at.as_ref().map(|time| time.as_str().to_owned()),
            finished_at: value.finished_at.as_ref().map(|time| time.as_str().to_owned()),
            measurements,
            criteria,
            detail: value.detail.as_ref().map(Diagnostic::try_from).transpose()?,
        })
    }
}

impl Step {
    fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("step metadata is missing"))?.projection_memory()?;
        let detail = self.detail.as_ref().map_or(0, MemorySize::memory_size);
        if [metadata, detail].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.step_id.capacity(),
            self.started_at.as_ref().map_or(0, String::capacity),
            self.finished_at.as_ref().map_or(0, String::capacity),
            metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()),
            detail.saturating_sub(std::mem::size_of::<crate::diagnostics::Diagnostic>()),
            self.measurements.len().saturating_mul(std::mem::size_of::<dokime_domain::model::execution::result::MeasurementRecord>()),
            self.criteria.len().saturating_mul(std::mem::size_of::<dokime_domain::model::execution::result::CriterionResult>()),
        ];
        let mut bytes =
            fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::execution::result::StepRecord>(), usize::saturating_add);
        for measurement in &self.measurements {
            let memory = measurement.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(memory.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::result::MeasurementRecord>()));
        }
        for criterion in &self.criteria {
            let memory = criterion.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes
                .saturating_add(memory.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::result::CriterionResult>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<Step> for dokime_domain::model::execution::result::StepRecord {
    type Error = Error;
    fn try_from(row: Step) -> Result<Self> {
        use dokime_domain::model::execution::result::StepStatus;
        use dokime_domain::model::identity::StepId;
        use dokime_domain::model::time::Timestamp;
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("step projection memory overflows"));
        }
        let status = match row.status.as_str() {
            "pending" => StepStatus::Pending,
            "running" => StepStatus::Running,
            "passed" => StepStatus::Passed,
            "failed" => StepStatus::Failed,
            "skipped" => StepStatus::Skipped,
            "cancelled" => StepStatus::Cancelled,
            _ => return Err(Error::Invalid("invalid step status")),
        };
        let started_at = row.started_at.map(Timestamp::new);
        let finished_at = row.finished_at.map(Timestamp::new);
        for at in [&started_at, &finished_at].into_iter().flatten() {
            at.to_datetime()?;
        }
        let mut measurements = Vec::with_capacity(row.measurements.len());
        for measurement in row.measurements {
            measurements.push(measurement.try_into()?);
        }
        let mut criteria = Vec::with_capacity(row.criteria.len());
        for criterion in row.criteria {
            criteria.push(criterion.try_into()?);
        }
        Ok(Self {
            step_id: StepId::from_uuid(row.step_id)?,
            metadata: row.metadata.ok_or(Error::Invalid("step metadata is missing"))?.try_into()?,
            status,
            started_at,
            finished_at,
            measurements,
            criteria,
            detail: row.detail.map(Into::into),
        })
    }
}

impl Parameter {
    fn encoding_memory(value: &crate::definition::MaterializedParameter) -> Result<usize> {
        let metadata = metadata::Row::encoding_memory(&value.metadata)?;
        let scalar = templates::Value::encoding_memory(&value.value)?;
        let unit = value.unit.as_ref().map(templates::Unit::encoding_memory).transpose()?.unwrap_or(0);
        if [metadata, scalar, unit].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            value.group_id.as_ref().map_or(0, |id| id.as_str().len()),
            parameter_source(&value.source).len(),
            scalar.saturating_sub(std::mem::size_of::<templates::Value>()),
            unit.saturating_sub(std::mem::size_of::<templates::Unit>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    fn projection_memory(&self) -> Result<usize> {
        let metadata = self.metadata.as_ref().ok_or(Error::Invalid("parameter metadata is missing"))?.projection_memory()?;
        let value = self.value.as_ref().ok_or(Error::Invalid("parameter value is missing"))?.projection_memory()?;
        let unit = self.unit.as_ref().map(templates::Unit::projection_memory).transpose()?.unwrap_or(0);
        if [metadata, value, unit].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            metadata.saturating_sub(std::mem::size_of::<dokime_domain::model::identity::Metadata>()),
            self.group_id.as_ref().map_or(0, String::capacity),
            value.saturating_sub(std::mem::size_of::<dokime_domain::model::values::Value>()),
            unit.saturating_sub(std::mem::size_of::<dokime_domain::model::values::MeasurementUnit>()),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<crate::definition::MaterializedParameter>(), usize::saturating_add))
    }
}

impl TryFrom<&crate::definition::MaterializedParameter> for Parameter {
    type Error = Error;

    fn try_from(value: &crate::definition::MaterializedParameter) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("parameter encoding memory overflows"));
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(&value.metadata)?),
            group_id: value.group_id.as_ref().map(|id| id.as_str().to_owned()),
            source: parameter_source(&value.source).to_owned(),
            value: Some(templates::Value::try_from(&value.value)?),
            unit: value.unit.as_ref().map(templates::Unit::try_from).transpose()?,
        })
    }
}

impl TryFrom<Parameter> for crate::definition::MaterializedParameter {
    type Error = Error;

    fn try_from(row: Parameter) -> Result<Self> {
        use crate::definition::ParameterSource;
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("parameter projection memory overflows"));
        }
        let source = match row.source.as_str() {
            "definition" => ParameterSource::Definition,
            "traceability" => ParameterSource::Traceability,
            "environment" => ParameterSource::Environment,
            "resource" => ParameterSource::Resource,
            "operator" => ParameterSource::Operator,
            "runtime" => ParameterSource::Runtime,
            "telemetry" => ParameterSource::Telemetry,
            "record" => ParameterSource::Record,
            "output" => ParameterSource::Output,
            _ => return Err(Error::Invalid("invalid parameter source")),
        };
        Ok(Self {
            metadata: row.metadata.ok_or(Error::Invalid("parameter metadata is missing"))?.try_into()?,
            group_id: row.group_id.map(dokime_domain::model::identity::Identifier::new).transpose()?,
            source,
            value: row.value.ok_or(Error::Invalid("parameter value is missing"))?.try_into()?,
            unit: row.unit.map(TryInto::try_into).transpose()?,
        })
    }
}

impl Output {
    fn encoding_memory(value: &dokime_domain::model::execution::saved::ExecutionOutput) -> Result<usize> {
        dokime_domain::model::execution::saved::validate_output_path(&value.relative_path)?;
        value.created_at.to_datetime()?;
        let file = value.file.memory_size();
        let content = artifacts::Content::encoding_memory(&value.content)?;
        if [file, content].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.relative_path.len())
            .saturating_add(value.created_at.as_str().len())
            .saturating_add(file.saturating_sub(std::mem::size_of::<FileReference>()))
            .saturating_add(saved_stream_state(value.stream_state).map_or(0, str::len))
            .saturating_add(content.saturating_sub(std::mem::size_of::<artifacts::Content>())))
    }
}

impl TryFrom<&dokime_domain::model::execution::saved::ExecutionOutput> for Output {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::saved::ExecutionOutput) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("execution output encoding memory overflow"));
        }
        Ok(Self {
            relative_path: value.relative_path.as_str().to_owned(),
            file: Some(value.file.clone()),
            stream_state: saved_stream_state(value.stream_state).map(str::to_owned),
            content: Some(artifacts::Content::try_from(&value.content)?),
            created_at: value.created_at.as_str().to_owned(),
        })
    }
}

impl Output {
    fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        time::OffsetDateTime::parse(&self.created_at, &time::format_description::well_known::Rfc3339)?;
        let file = self.file.as_ref().ok_or(Error::Invalid("execution output file is missing"))?;
        let memory = file.memory_size();
        let content = self.content.as_ref().ok_or(Error::Invalid("saved output content descriptor is missing"))?.projection_memory()?;
        if [memory, content].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<dokime_domain::model::execution::saved::ExecutionOutput>()
            .saturating_add(self.relative_path.capacity())
            .saturating_add(self.created_at.capacity())
            .saturating_add(memory.saturating_sub(std::mem::size_of_val(file)))
            .saturating_add(content.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::saved::ExecutionOutputContent>())))
    }
}

impl TryFrom<Output> for dokime_domain::model::execution::saved::ExecutionOutput {
    type Error = Error;
    fn try_from(row: Output) -> Result<Self> {
        use dokime_domain::model::execution::saved::SavedStreamState;
        use dokime_domain::model::execution::saved::validate_output_path;
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution output projection memory overflows"));
        }
        validate_output_path(&row.relative_path)?;
        let stream_state = match row.stream_state.as_deref() {
            None => None,
            Some("finalized") => Some(SavedStreamState::Finalized),
            Some("interrupted") => Some(SavedStreamState::Interrupted),
            _ => return Err(Error::Invalid("invalid saved stream state")),
        };
        Ok(Self {
            relative_path: row.relative_path,
            created_at: dokime_domain::model::time::Timestamp::new(row.created_at),
            file: row.file.ok_or(Error::Invalid("execution output file is missing"))?,
            stream_state,
            content: row.content.ok_or(Error::Invalid("saved output content descriptor is missing"))?.try_into()?,
        })
    }
}

impl Source {
    fn encoding_memory(value: &dokime_domain::model::execution::saved::ExecutionSource) -> Result<usize> {
        let evidence = value.evidence_through.memory_size();
        let revision = Revision::encoding_memory(&value.execution)?;
        let context = value.context.memory_size();
        if [evidence, revision, context].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            value.owner_node_id.as_str().len(),
            evidence.saturating_sub(std::mem::size_of::<StorageCursor>()),
            revision.saturating_sub(std::mem::size_of::<Revision>()),
            context.saturating_sub(std::mem::size_of::<FileReference>()),
            dokime_domain::model::execution::saved::output_validation_memory(&value.outputs),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<Self>(), usize::saturating_add);
        for output in &value.outputs {
            let bound = Output::encoding_memory(output)?;
            if bound == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(bound);
        }
        Ok(bytes)
    }
}

impl TryFrom<&dokime_domain::model::execution::saved::ExecutionSource> for Source {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::saved::ExecutionSource) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("execution source encoding memory overflow"));
        }
        value.validate(&value.execution.execution_id)?;
        let mut outputs = Vec::new();
        outputs.try_reserve_exact(value.outputs.len()).map_err(|_| Error::Invalid("execution output encoding allocation failed"))?;
        for output in &value.outputs {
            outputs.push(Output::try_from(output)?);
        }
        Ok(Self {
            owner_node_id: value.owner_node_id.as_str().to_owned(),
            evidence_through: Some(value.evidence_through.clone()),
            execution: Some(Revision::try_from(&value.execution)?),
            context: Some(value.context.clone()),
            outputs,
        })
    }
}

impl Source {
    fn projection_memory(&self) -> Result<usize> {
        use dokime_domain::model::execution::saved::ExecutionOutput;
        use dokime_domain::model::execution::saved::ExecutionRevision;
        use dokime_domain::model::execution::saved::ExecutionSource;
        use harmos::MemorySize;
        let evidence = self.evidence_through.as_ref().ok_or(Error::Invalid("execution source evidence frontier is missing"))?;
        let revision = self.execution.as_ref().ok_or(Error::Invalid("execution source revision is missing"))?.projection_memory()?;
        let context = self.context.as_ref().ok_or(Error::Invalid("execution source context is missing"))?;
        if [evidence.memory_size(), revision, context.memory_size()].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.owner_node_id.capacity(),
            evidence.memory_size().saturating_sub(std::mem::size_of_val(evidence)),
            revision.saturating_sub(std::mem::size_of::<ExecutionRevision>()),
            context.memory_size().saturating_sub(std::mem::size_of_val(context)),
            self.outputs.len().saturating_mul(std::mem::size_of::<ExecutionOutput>()),
            harmos::btree_map_memory::<&String, ()>(self.outputs.len()),
            harmos::btree_map_memory::<&str, ()>(self.outputs.len()),
            harmos::btree_map_memory::<&dokime_domain::model::identity::ArtifactId, ()>(
                self.outputs.iter().filter(|output| output.content.as_ref().is_some_and(artifacts::Content::is_artifact)).count(),
            ),
        ];
        let mut bytes = fields.into_iter().fold(std::mem::size_of::<ExecutionSource>(), usize::saturating_add);
        for output in &self.outputs {
            let memory = output.projection_memory()?;
            if memory == usize::MAX {
                return Ok(usize::MAX);
            }
            bytes = bytes.saturating_add(memory.saturating_sub(std::mem::size_of::<ExecutionOutput>()));
        }
        Ok(bytes)
    }
}

impl TryFrom<Source> for dokime_domain::model::execution::saved::ExecutionSource {
    type Error = Error;
    fn try_from(row: Source) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution source projection memory overflows"));
        }
        if row.owner_node_id.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(Error::Invalid("noncanonical execution source owner"));
        }
        let mut outputs = Vec::with_capacity(row.outputs.len());
        for output in row.outputs {
            outputs.push(output.try_into()?);
        }
        let source = Self {
            owner_node_id: dokime_domain::model::identity::NodeId::from_uuid(row.owner_node_id)?,
            evidence_through: row.evidence_through.ok_or(Error::Invalid("execution source evidence frontier is missing"))?,
            execution: row.execution.ok_or(Error::Invalid("execution source revision is missing"))?.try_into()?,
            context: row.context.ok_or(Error::Invalid("execution source context is missing"))?,
            outputs,
        };
        source.validate(&source.execution.execution_id)?;
        Ok(source)
    }
}

impl Seal {
    pub(crate) fn check_publication(&self, expected: PublicationCheck<'_>) -> Result<()> {
        let captured = self.source.as_ref().ok_or(Error::Invalid("execution seal source is missing"))?;
        if captured.execution.as_ref() != Some(expected.source)
            || captured.context.as_ref() != Some(expected.input)
            || captured.evidence_through.as_ref() != Some(expected.evidence)
            || self.report.as_ref() != Some(expected.report)
            || self.dependencies_saved.as_ref() != Some(expected.saved)
            || self.sealed_at != expected.at
        {
            return Err(Error::Invalid("execution seal differs from report publication"));
        }
        Ok(())
    }

    pub(crate) fn encoding_memory(value: &dokime_domain::model::execution::saved::ExecutionSeal) -> Result<usize> {
        value.sealed_at.to_datetime()?;
        let source = Source::encoding_memory(&value.source)?;
        let report = value.report.memory_size();
        let saved = value.dependencies_saved.memory_size();
        if [source, report, saved].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(source.saturating_sub(std::mem::size_of::<Source>()))
            .saturating_add(report.saturating_sub(std::mem::size_of::<FileReference>()))
            .saturating_add(saved.saturating_sub(std::mem::size_of::<StorageCursor>()))
            .saturating_add(value.sealed_at.as_str().len()))
    }
}

impl TryFrom<&dokime_domain::model::execution::saved::ExecutionSeal> for Seal {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::saved::ExecutionSeal) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("execution seal encoding memory overflow"));
        }
        Ok(Self {
            source: Some(Source::try_from(&value.source)?),
            report: Some(value.report.clone()),
            dependencies_saved: Some(value.dependencies_saved.clone()),
            sealed_at: value.sealed_at.as_str().to_owned(),
        })
    }
}

impl Seal {
    pub(crate) fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let source = self.source.as_ref().ok_or(Error::Invalid("execution seal source is missing"))?.projection_memory()?;
        let report = self.report.as_ref().ok_or(Error::Invalid("execution seal report is missing"))?;
        let saved = self.dependencies_saved.as_ref().ok_or(Error::Invalid("execution seal saved frontier is missing"))?;
        if [source, report.memory_size(), saved.memory_size()].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            source.saturating_sub(std::mem::size_of::<dokime_domain::model::execution::saved::ExecutionSource>()),
            report.memory_size().saturating_sub(std::mem::size_of_val(report)),
            saved.memory_size().saturating_sub(std::mem::size_of_val(saved)),
            self.sealed_at.capacity(),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::execution::saved::ExecutionSeal>(), usize::saturating_add))
    }
}

impl TryFrom<Seal> for dokime_domain::model::execution::saved::ExecutionSeal {
    type Error = Error;
    fn try_from(row: Seal) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("execution seal projection memory overflows"));
        }
        let sealed_at = dokime_domain::model::time::Timestamp::new(row.sealed_at);
        sealed_at.to_datetime()?;
        Ok(Self {
            source: row.source.ok_or(Error::Invalid("execution seal source is missing"))?.try_into()?,
            report: row.report.ok_or(Error::Invalid("execution seal report is missing"))?,
            dependencies_saved: row.dependencies_saved.ok_or(Error::Invalid("execution seal saved frontier is missing"))?,
            sealed_at,
        })
    }
}

impl Revision {
    pub fn encoding_memory(value: &dokime_domain::model::execution::saved::ExecutionRevision) -> Result<usize> {
        value.version.check_position(&value.position)?;
        let position = value.position.memory_size();
        let version = value.version.memory_size();
        if [position, version].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        Ok(std::mem::size_of::<Self>()
            .saturating_add(value.execution_id.as_str().len())
            .saturating_add(position.saturating_sub(std::mem::size_of::<StorageCursor>()))
            .saturating_add(version.saturating_sub(std::mem::size_of::<ObjectVersionToken>())))
    }
}

impl TryFrom<&dokime_domain::model::execution::saved::ExecutionRevision> for Revision {
    type Error = Error;

    fn try_from(value: &dokime_domain::model::execution::saved::ExecutionRevision) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(Error::Invalid("execution revision encoding memory overflow"));
        }
        Ok(Self {
            execution_id: value.execution_id.as_str().to_owned(),
            position: Some(value.position.clone()),
            version: Some(value.version.clone()),
        })
    }
}

impl Revision {
    pub(crate) fn check_saved(&self, saved: &StorageCursor) -> Result<()> {
        let position = self.position.as_ref().ok_or(Error::Invalid("execution revision position is missing"))?;
        if !saved.covers(position)? {
            return Err(Error::Invalid("execution revision exceeds durability cut"));
        }
        Ok(())
    }

    pub(crate) fn check_cut(&self, through: &StorageCursor) -> Result<&str> {
        if self.execution_id.len() != 36
            || self.execution_id.bytes().any(|byte| byte.is_ascii_uppercase())
            || TestExecutionId::from_uuid(&self.execution_id).is_err()
        {
            return Err(Error::Invalid("invalid execution revision identity"));
        }
        let position = self.position.as_ref().ok_or(Error::Invalid("execution revision position is missing"))?;
        let version = self.version.as_ref().ok_or(Error::Invalid("execution revision version is missing"))?;
        version.check_position(position)?;
        through.history.check(&position.history)?;
        Ok(&self.execution_id)
    }

    pub fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let position = self.position.as_ref().ok_or(Error::Invalid("execution revision position is missing"))?;
        let version = self.version.as_ref().ok_or(Error::Invalid("execution revision version is missing"))?;
        if [position.memory_size(), version.memory_size()].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.execution_id.capacity(),
            position.memory_size().saturating_sub(std::mem::size_of_val(position)),
            version.memory_size().saturating_sub(std::mem::size_of_val(version)),
        ];
        Ok(fields.into_iter().fold(std::mem::size_of::<dokime_domain::model::execution::saved::ExecutionRevision>(), usize::saturating_add))
    }
}

impl TryFrom<Revision> for dokime_domain::model::execution::saved::ExecutionRevision {
    type Error = Error;

    fn try_from(row: Revision) -> Result<Self> {
        if row.execution_id.bytes().any(|byte| byte.is_ascii_uppercase()) {
            return Err(Error::Invalid("noncanonical execution revision identity"));
        }
        Ok(Self {
            execution_id: TestExecutionId::from_uuid(row.execution_id)?,
            position: row.position.ok_or(Error::Invalid("execution revision position is missing"))?,
            version: row.version.ok_or(Error::Invalid("execution revision version is missing"))?,
        })
    }
}

impl Identified for Execution {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.execution"), 1)
    }
}
impl StoredObject for Execution {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::UntilReleased;
    fn key(&self) -> &str {
        &self.test_execution_id
    }
}

impl Execution {
    pub fn into_execution(self) -> Result<TestExecution> {
        self.try_into()
    }

    pub fn restore(&self) -> Result<TestExecution> {
        self.clone().try_into()
    }
}

/* === Public Functions === */
pub async fn read(storage: &Manager, id: &TestExecutionId, version: ObjectVersion) -> Result<Option<ObjectRevision<TestExecution>>> {
    let Some(stored) = storage.read_object::<Execution>(id.as_str(), version).await? else {
        return Ok(None);
    };
    Ok(Some(ObjectRevision { value: stored.value.try_into()?, position: stored.position, version: stored.version }))
}

pub async fn flush_seal(storage: &Manager, execution: &TestExecution, receipt: &harmos::storage::ObjectReceipt) -> Result<StorageCursor> {
    let seal = execution.saved.as_ref().ok_or(Error::Invalid("execution is unsealed"))?;
    seal.validate(execution)?;
    receipt.version.check_position(&receipt.position)?;
    if receipt.position.history != seal.dependencies_saved.history || receipt.position.sequence <= seal.dependencies_saved.sequence {
        return Err(Error::Invalid("seal admission must follow saved dependencies"));
    }
    Ok(storage.flush_until(&receipt.position).await?)
}

/* === Private Functions === */
fn parameter_source(value: &crate::definition::ParameterSource) -> &'static str {
    match value {
        ParameterSource::Definition => "definition",
        ParameterSource::Traceability => "traceability",
        ParameterSource::Environment => "environment",
        ParameterSource::Resource => "resource",
        ParameterSource::Operator => "operator",
        ParameterSource::Runtime => "runtime",
        ParameterSource::Telemetry => "telemetry",
        ParameterSource::Record => "record",
        ParameterSource::Output => "output",
    }
}

fn termination_reason(value: &dokime_domain::model::execution::termination::TerminationReason) -> &'static str {
    match value {
        TerminationReason::Completed => "completed",
        TerminationReason::Failed => "failed",
        TerminationReason::Cancelled => "cancelled",
        TerminationReason::Interrupted => "interrupted",
    }
}

fn execution_status_text(value: &dokime_domain::model::execution::result::ExecutionStatus) -> &'static str {
    match value {
        dokime_domain::model::execution::result::ExecutionStatus::Skipped => "skipped",
        dokime_domain::model::execution::result::ExecutionStatus::Invalidated => "invalidated",
        dokime_domain::model::execution::result::ExecutionStatus::Pending => "pending",
        dokime_domain::model::execution::result::ExecutionStatus::Running => "running",
        dokime_domain::model::execution::result::ExecutionStatus::Passed => "passed",
        dokime_domain::model::execution::result::ExecutionStatus::Failed => "failed",
        dokime_domain::model::execution::result::ExecutionStatus::Blocked => "blocked",
        dokime_domain::model::execution::result::ExecutionStatus::Cancelled => "cancelled",
        dokime_domain::model::execution::result::ExecutionStatus::Interrupted => "interrupted",
    }
}

fn execution_status(value: &str) -> Result<dokime_domain::model::execution::result::ExecutionStatus> {
    use dokime_domain::model::execution::result::ExecutionStatus;
    match value {
        "skipped" => Ok(ExecutionStatus::Skipped),
        "invalidated" => Ok(ExecutionStatus::Invalidated),
        "pending" => Ok(ExecutionStatus::Pending),
        "running" => Ok(ExecutionStatus::Running),
        "passed" => Ok(ExecutionStatus::Passed),
        "failed" => Ok(ExecutionStatus::Failed),
        "blocked" => Ok(ExecutionStatus::Blocked),
        "cancelled" => Ok(ExecutionStatus::Cancelled),
        "interrupted" => Ok(ExecutionStatus::Interrupted),
        _ => Err(Error::Invalid("invalid execution status")),
    }
}

fn execution_mode_text(value: &dokime_domain::model::execution::ExecutionMode) -> &'static str {
    match value {
        dokime_domain::model::execution::ExecutionMode::Automatic => "automatic",
        dokime_domain::model::execution::ExecutionMode::Interactive => "interactive",
    }
}

fn execution_mode(value: &str) -> Result<dokime_domain::model::execution::ExecutionMode> {
    use dokime_domain::model::execution::ExecutionMode;
    match value {
        "automatic" => Ok(ExecutionMode::Automatic),
        "interactive" => Ok(ExecutionMode::Interactive),
        _ => Err(Error::Invalid("invalid execution mode")),
    }
}

fn capture_phase_text(value: &dokime_domain::model::execution::CapturePhase) -> &'static str {
    match value {
        dokime_domain::model::execution::CapturePhase::Default => "default",
        dokime_domain::model::execution::CapturePhase::Initialization => "initialization",
        dokime_domain::model::execution::CapturePhase::Cleanup => "cleanup",
    }
}

fn capture_phase(value: &str) -> Result<dokime_domain::model::execution::CapturePhase> {
    use dokime_domain::model::execution::CapturePhase;
    match value {
        "default" => Ok(CapturePhase::Default),
        "initialization" => Ok(CapturePhase::Initialization),
        "cleanup" => Ok(CapturePhase::Cleanup),
        _ => Err(Error::Invalid("invalid capture phase")),
    }
}

fn copied_strings(values: &[String]) -> Result<Vec<String>> {
    let mut copied = Vec::new();
    copied.try_reserve_exact(values.len()).map_err(|_| Error::Invalid("diagnostic string slots allocation failed"))?;
    copied.extend(values.iter().map(|text| text.as_str().to_owned()));
    Ok(copied)
}

fn step_status_text(value: &dokime_domain::model::execution::result::StepStatus) -> &'static str {
    match value {
        StepStatus::Pending => "pending",
        StepStatus::Running => "running",
        StepStatus::Passed => "passed",
        StepStatus::Failed => "failed",
        StepStatus::Skipped => "skipped",
        StepStatus::Cancelled => "cancelled",
    }
}

fn criterion_status_text(value: &dokime_domain::model::execution::result::CriterionStatus) -> &'static str {
    match value {
        CriterionStatus::Pending => "pending",
        CriterionStatus::Passed => "passed",
        CriterionStatus::Failed => "failed",
        CriterionStatus::Indeterminate => "indeterminate",
        CriterionStatus::Skipped => "skipped",
    }
}

fn saved_stream_state(value: Option<dokime_domain::model::execution::saved::SavedStreamState>) -> Option<&'static str> {
    use dokime_domain::model::execution::saved::SavedStreamState;
    match value {
        None => None,
        Some(SavedStreamState::Finalized) => Some("finalized"),
        Some(SavedStreamState::Interrupted) => Some("interrupted"),
    }
}

fn validate_owner(source: &TestExecution) -> Result<()> {
    if let Some(saved) = &source.saved {
        saved.validate(source)?;
        saved.sealed_at.to_datetime()?;
    }
    if let Some(saved) = &source.saved {
        for output in &saved.source.outputs {
            if let dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } = &output.content
                && (record.scope.test_cycle_id.as_ref().is_some_and(|id| id != &source.test_cycle_id)
                    || record.scope.test_case_id.as_ref().is_some_and(|id| id != &source.test_case_id))
            {
                return Err(Error::Invalid("saved artifact scope differs from its execution owner"));
            }
        }
    }
    if source.test_case_id != source.snapshot.case().test_case_id {
        return Err(Error::Invalid("execution case differs from its frozen input"));
    }
    if source.scope.test_execution_id.as_ref().is_some_and(|id| id != &source.test_execution_id)
        || source.scope.test_cycle_id.as_ref().is_some_and(|id| id != &source.test_cycle_id)
        || source.scope.test_case_id.as_ref().is_some_and(|id| id != &source.test_case_id)
    {
        return Err(Error::Invalid("execution scope differs from its owner"));
    }
    Ok(())
}

fn validate_attempts(source: &TestExecution) -> Result<()> {
    for at in source.started_at.iter().chain(source.finished_at.iter()) {
        at.to_datetime()?;
    }
    let mut ids = std::collections::BTreeSet::new();
    for attempt in &source.step_attempts {
        if !ids.insert(&attempt.attempt_id) {
            return Err(Error::Invalid("duplicate step attempt identity"));
        }
        source.validate_step_attempt(attempt).map_err(Error::Invalid)?;
    }
    if let Some(start) = &source.active_step_attempt {
        if source.status != dokime_domain::model::execution::result::ExecutionStatus::Running || source.finished_at.is_some() {
            return Err(Error::Invalid("active attempt belongs to a finished execution"));
        }
        if source.capture_phase != dokime_domain::model::execution::CapturePhase::Default {
            return Err(Error::Invalid("active attempt has a non-step capture phase"));
        }
        source.validate_step_start(start).map_err(Error::Invalid)?;
        if ids.contains(&start.attempt_id) {
            return Err(Error::Invalid("completed step attempt is also active"));
        }
    }
    for scope in source.records.subscribed_records().values() {
        if scope.test_execution_id.as_ref() != Some(&source.test_execution_id)
            || scope.test_cycle_id.as_ref() != Some(&source.test_cycle_id)
            || scope.test_case_id.as_ref() != Some(&source.test_case_id)
        {
            return Err(Error::Invalid("captured record scope differs from its execution"));
        }
        if let dokime_domain::model::execution::EvidenceOrigin::Step { attempt_id } = &scope.origin {
            let step =
                source.active_step_attempt.as_ref().filter(|start| &start.attempt_id == attempt_id).map(|start| &start.step_id).or_else(
                    || source.step_attempts.iter().find(|attempt| &attempt.attempt_id == attempt_id).map(|attempt| &attempt.step_id),
                );
            if step.is_none() || step != scope.step_id.as_ref() {
                return Err(Error::Invalid("captured record has invalid attempt provenance"));
            }
        }
    }
    for step in &source.steps {
        for at in step.started_at.iter().chain(step.finished_at.iter()) {
            at.to_datetime()?;
        }
    }
    Ok(())
}
/* === Tests === */
