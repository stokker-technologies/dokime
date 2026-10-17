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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{Message, storage::SchemaKey};

// Local Imports
use super::{Error, Result, values};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;
use crate::records::{Record, RecordEntry};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct RecordRow {
    sequence: u64,
    record_id: String,
    timestamp: String,
    received_at: String,
    record_type: String,
    schema_version: u32,
    source: String,
    severity: String,
    message: Option<String>,
    error: Option<ErrorRow>,
    details: Vec<values::Cell>,
    scope: Option<ScopeRow>,
    related_sample_id: Option<String>,
    related_stream_id: Option<String>,
    related_artifact_id: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScopeRow {
    pub(crate) session_id: Option<String>,
    pub(crate) test_cycle_id: Option<String>,
    pub(crate) test_case_id: Option<String>,
    pub(crate) test_execution_id: Option<String>,
    pub(crate) step_id: Option<String>,
    pub(crate) criterion_id: Option<String>,
    pub(crate) template_id: Option<String>,
    pub(crate) suite_id: Option<String>,
    pub(crate) origin: OriginRow,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct OriginRow {
    phase: u32,
    attempt_id: Option<String>,
}

impl From<&dokime_domain::model::execution::RecordScope> for ScopeRow {
    fn from(scope: &dokime_domain::model::execution::RecordScope) -> Self {
        use dokime_domain::model::execution::EvidenceOrigin;
        let (phase, attempt_id) = match &scope.origin {
            EvidenceOrigin::Default => (0, None),
            EvidenceOrigin::Initialization => (1, None),
            EvidenceOrigin::Step { attempt_id } => (2, Some(attempt_id.as_str().to_owned())),
            EvidenceOrigin::Cleanup => (3, None),
        };
        Self {
            session_id: scope.session_id.as_ref().map(|id| id.as_str().to_owned()),
            test_cycle_id: scope
                .test_cycle_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            test_case_id: scope.test_case_id.as_ref().map(|id| id.as_str().to_owned()),
            test_execution_id: scope
                .test_execution_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            step_id: scope.step_id.as_ref().map(|id| id.as_str().to_owned()),
            criterion_id: scope.criterion_id.as_ref().map(|id| id.as_str().to_owned()),
            template_id: scope.template_id.as_ref().map(|id| id.as_str().to_owned()),
            suite_id: scope.suite_id.as_ref().map(|id| id.as_str().to_owned()),
            origin: OriginRow { phase, attempt_id },
        }
    }
}

impl TryFrom<ScopeRow> for dokime_domain::model::execution::RecordScope {
    type Error = Error;

    fn try_from(row: ScopeRow) -> Result<Self> {
        use dokime_domain::model::identity::CaseId;
        use dokime_domain::model::identity::CriterionId;
        use dokime_domain::model::identity::SessionId;
        use dokime_domain::model::identity::StepId;
        use dokime_domain::model::identity::SuiteId;
        use dokime_domain::model::identity::TemplateId;
        use dokime_domain::model::identity::TestCycleId;
        use dokime_domain::model::identity::TestExecutionId;

        for id in [&row.test_cycle_id, &row.test_execution_id]
            .into_iter()
            .flatten()
        {
            if id.bytes().any(|byte| byte.is_ascii_uppercase()) {
                return Err(Error::Invalid(
                    "noncanonical scope execution or cycle identity",
                ));
            }
        }
        Ok(Self {
            session_id: row.session_id.map(SessionId::from_uuid).transpose()?,
            test_cycle_id: row.test_cycle_id.map(TestCycleId::from_uuid).transpose()?,
            test_case_id: row.test_case_id.map(CaseId::from_wire).transpose()?,
            test_execution_id: row
                .test_execution_id
                .map(TestExecutionId::from_uuid)
                .transpose()?,
            step_id: row.step_id.map(StepId::from_uuid).transpose()?,
            criterion_id: row.criterion_id.map(CriterionId::from_uuid).transpose()?,
            template_id: row.template_id.map(TemplateId::from_uuid).transpose()?,
            suite_id: row.suite_id.map(SuiteId::from_uuid).transpose()?,
            origin: row.origin.try_into()?,
        })
    }
}

impl ScopeRow {
    pub fn encoding_memory(scope: &dokime_domain::model::execution::RecordScope) -> usize {
        let fields = [
            scope.session_id.as_ref().map(|id| id.as_str()),
            scope.test_cycle_id.as_ref().map(|id| id.as_str()),
            scope.test_case_id.as_ref().map(|id| id.as_str()),
            scope.test_execution_id.as_ref().map(|id| id.as_str()),
            scope.step_id.as_ref().map(|id| id.as_str()),
            scope.criterion_id.as_ref().map(|id| id.as_str()),
            scope.template_id.as_ref().map(|id| id.as_str()),
            scope.suite_id.as_ref().map(|id| id.as_str()),
            match &scope.origin {
                dokime_domain::model::execution::EvidenceOrigin::Step { attempt_id } => Some(attempt_id.as_str()),
                _ => None,
            },
        ];
        fields
            .into_iter()
            .flatten()
            .fold(std::mem::size_of::<Self>(), |bytes, text| {
                bytes.saturating_add(text.len())
            })
    }

    pub fn projection_memory(&self) -> usize {
        let fields = [
            &self.session_id,
            &self.test_cycle_id,
            &self.test_case_id,
            &self.test_execution_id,
            &self.step_id,
            &self.criterion_id,
            &self.template_id,
            &self.suite_id,
            &self.origin.attempt_id,
        ];
        fields.into_iter().fold(
            std::mem::size_of::<dokime_domain::model::execution::RecordScope>(),
            |bytes, field| bytes.saturating_add(field.as_ref().map_or(0, String::capacity)),
        )
    }
}

impl TryFrom<OriginRow> for dokime_domain::model::execution::EvidenceOrigin {
    type Error = Error;

    fn try_from(row: OriginRow) -> Result<Self> {
        use dokime_domain::model::identity::StepAttemptId;
        match (row.phase, row.attempt_id) {
            (0, None) => Ok(Self::Default),
            (1, None) => Ok(Self::Initialization),
            (2, Some(id)) => Ok(Self::Step {
                attempt_id: StepAttemptId::from_uuid(id)?,
            }),
            (3, None) => Ok(Self::Cleanup),
            _ => Err(Error::Invalid("invalid native capture origin")),
        }
    }
}

impl serde::Serialize for OriginRow {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use dokime_domain::model::execution::EvidenceOrigin;
        use dokime_domain::model::identity::StepAttemptId;
        let origin = match (self.phase, self.attempt_id.as_deref()) {
            (0, None) => EvidenceOrigin::Default,
            (1, None) => EvidenceOrigin::Initialization,
            (2, Some(id)) => EvidenceOrigin::Step {
                attempt_id: StepAttemptId::from_uuid(id).map_err(serde::ser::Error::custom)?,
            },
            (3, None) => EvidenceOrigin::Cleanup,
            _ => return Err(serde::ser::Error::custom("invalid native capture origin")),
        };
        serde::Serialize::serialize(&origin, serializer)
    }
}
impl<'de> serde::Deserialize<'de> for OriginRow {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        use dokime_domain::model::execution::EvidenceOrigin;
        let (phase, attempt_id) =
            match <EvidenceOrigin as serde::Deserialize>::deserialize(deserializer)? {
                EvidenceOrigin::Default => (0, None),
                EvidenceOrigin::Initialization => (1, None),
                EvidenceOrigin::Step { attempt_id } => (2, Some(attempt_id.to_string())),
                EvidenceOrigin::Cleanup => (3, None),
            };
        Ok(Self { phase, attempt_id })
    }
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ErrorRow {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<CauseRow>,
    pub stacktrace_attachment: Option<String>,
    pub trace_id: Option<String>,
    pub correlation_id: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct CauseRow {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordRow {
    pub fn decoding_memory(bytes: &[u8]) -> Result<usize> {
        let native = <Self as Message>::decoding_memory(bytes)?;
        native
            .checked_add(std::mem::size_of::<Vec<u8>>())
            .and_then(|bound| bound.checked_add(bytes.len()))
            .ok_or_else(|| error("record decoding memory overflows"))
    }

    /// Decode one stored record payload into its entry.
    pub fn decode_entry(bytes: &[u8]) -> Result<RecordEntry> {
        decode(bytes, usize::MAX)
    }

    /// Project a decoded stored record into its entry.
    pub fn entry(self) -> Result<RecordEntry> {
        let bound = self.projection_memory()?;
        let allowance =
            NonZeroUsize::new(bound).ok_or_else(|| error("record projection memory is invalid"))?;
        self.into_entry(allowance)
    }

    pub fn from_bytes(bytes: &[u8], max_bytes: usize) -> Result<Self> {
        if bytes.len() > max_bytes {
            return Err(error("record capture frame exceeds byte limit"));
        }
        let row = Self::decode(bytes)?;
        let length = row.encoded_len();
        if length > bytes.len() {
            return Err(error("record capture frame is not canonical protobuf"));
        }
        let mut canonical = exact_slots(length)?;
        row.encode_into(&mut canonical);
        if canonical != bytes {
            return Err(error("record capture frame is not canonical protobuf"));
        }
        Ok(row)
    }

    pub fn encoding_memory(entry: &RecordEntry) -> Result<usize> {
        entry.timestamp.to_datetime()?;
        entry.received_at.to_datetime()?;
        if entry.schema_version == 0 {
            return Err(error("record schema version must be positive"));
        }
        let mut bytes = [
            entry.record_id.as_str(),
            entry.timestamp.as_str(),
            entry.received_at.as_str(),
            entry.record_type.as_str(),
            entry.source.as_str(),
            severity_name(entry.severity),
        ]
        .into_iter()
        .fold(std::mem::size_of::<Self>(), |bytes, text| {
            bytes.saturating_add(text.len())
        });
        for text in [
            entry.message.as_deref(),
            entry.related_sample_id.as_ref().map(|id| id.as_str()),
            entry.related_stream_id.as_ref().map(|id| id.as_str()),
            entry.related_artifact_id.as_ref().map(|id| id.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(text.len());
        }
        bytes = excess(
            bytes,
            values::encoding_memory(&entry.details)?,
            std::mem::size_of::<Vec<values::Cell>>(),
        );
        if let Some(scope) = &entry.scope {
            bytes = excess(
                bytes,
                ScopeRow::encoding_memory(scope),
                std::mem::size_of::<ScopeRow>(),
            );
        }
        if let Some(detail) = &entry.error {
            bytes = excess(
                bytes,
                ErrorRow::encoding_memory(detail),
                std::mem::size_of::<ErrorRow>(),
            );
        }
        Ok(bytes)
    }

    pub fn from_entry(entry: &RecordEntry) -> Result<Self> {
        if Self::encoding_memory(entry)? == usize::MAX {
            return Err(error("record encoding memory overflows"));
        }
        Ok(Self {
            sequence: entry.sequence,
            record_id: copy_text(entry.record_id.as_str())?,
            timestamp: copy_text(entry.timestamp.as_str())?,
            received_at: copy_text(entry.received_at.as_str())?,
            record_type: copy_text(entry.record_type.as_str())?,
            schema_version: entry.schema_version,
            source: copy_text(&entry.source)?,
            severity: copy_text(severity_name(entry.severity))?,
            message: entry.message.as_deref().map(copy_text).transpose()?,
            error: entry
                .error
                .as_ref()
                .map(ErrorRow::from_detail)
                .transpose()?,
            details: values::encode(&entry.details)?,
            scope: entry.scope.as_ref().map(ScopeRow::from),
            related_sample_id: entry
                .related_sample_id
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
            related_stream_id: entry
                .related_stream_id
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
            related_artifact_id: entry
                .related_artifact_id
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
        })
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let strings = [
            &self.record_id,
            &self.timestamp,
            &self.received_at,
            &self.record_type,
            &self.source,
        ];
        let mut bytes = strings
            .into_iter()
            .fold(std::mem::size_of::<RecordEntry>(), |bytes, value| {
                bytes.saturating_add(value.capacity())
            });
        for value in [
            &self.message,
            &self.related_sample_id,
            &self.related_stream_id,
            &self.related_artifact_id,
        ] {
            bytes = bytes.saturating_add(value.as_ref().map_or(0, String::capacity));
        }
        let details = values::owned_memory(&self.details)?;
        if details == usize::MAX {
            return Ok(usize::MAX);
        }
        bytes = bytes.saturating_add(details.saturating_sub(std::mem::size_of::<
            std::collections::BTreeMap<String, dokime_domain::model::values::Value>,
        >()));
        if let Some(error) = &self.error {
            bytes = bytes.saturating_add(
                error
                    .projection_memory()
                    .saturating_sub(std::mem::size_of::<crate::records::RecordErrorDetail>()),
            );
        }
        if let Some(scope) = &self.scope {
            bytes = bytes.saturating_add(
                scope
                    .projection_memory()
                    .saturating_sub(std::mem::size_of::<dokime_domain::model::execution::RecordScope>()),
            );
        }
        Ok(bytes)
    }

    pub fn into_entry(self, allowance: NonZeroUsize) -> Result<RecordEntry> {
        use dokime_domain::model::identity::ArtifactId;
        use dokime_domain::model::identity::RecordId;
        use dokime_domain::model::identity::TelemetrySampleId;
        use dokime_domain::model::identity::TelemetryStreamId;
        use crate::records::RecordSeverity;
        let bound = self.projection_memory()?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(error("record projection exceeds memory allowance"));
        }
        let timestamp = Timestamp::new(self.timestamp);
        let received_at = Timestamp::new(self.received_at);
        received_at.to_datetime()?;
        timestamp.to_datetime()?;
        if self.schema_version == 0 {
            return Err(error("record schema version must be positive"));
        }
        let severity = match self.severity.as_str() {
            "debug" => RecordSeverity::Debug,
            "info" => RecordSeverity::Info,
            "warning" => RecordSeverity::Warning,
            "error" => RecordSeverity::Error,
            _ => return Err(error("invalid native record severity")),
        };
        let details = values::decode_owned(self.details, allowance)?;
        Ok(RecordEntry::from_record(
            self.sequence,
            Record {
                record_id: RecordId::from_uuid(self.record_id)?,
                timestamp,
                received_at,
                record_type: Identifier::new(self.record_type)?,
                schema_version: self.schema_version,
                source: self.source,
                severity,
                message: self.message,
                error: self.error.map(TryInto::try_into).transpose()?,
                details,
                scope: self.scope.map(TryInto::try_into).transpose()?,
                related_sample_id: self
                    .related_sample_id
                    .map(TelemetrySampleId::from_uuid)
                    .transpose()?,
                related_stream_id: self
                    .related_stream_id
                    .map(TelemetryStreamId::from_uuid)
                    .transpose()?,
                related_artifact_id: self
                    .related_artifact_id
                    .map(ArtifactId::from_uuid)
                    .transpose()?,
            },
        ))
    }
}

impl ErrorRow {
    fn encoding_memory(detail: &crate::records::RecordErrorDetail) -> usize {
        let mut bytes = std::mem::size_of::<Self>().saturating_add(detail.message.len());
        for text in [
            detail.code.as_deref(),
            detail.source.as_deref(),
            detail.stacktrace_attachment.as_ref().map(|id| id.as_str()),
            detail.trace_id.as_ref().map(|id| id.as_str()),
            detail.correlation_id.as_ref().map(|id| id.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(text.len());
        }
        for cause in &detail.cause_chain {
            bytes = bytes
                .saturating_add(std::mem::size_of::<CauseRow>())
                .saturating_add(cause.message.len())
                .saturating_add(cause.code.as_deref().map_or(0, str::len))
                .saturating_add(cause.source.as_deref().map_or(0, str::len));
        }
        bytes
    }
    fn from_detail(detail: &crate::records::RecordErrorDetail) -> Result<Self> {
        if Self::encoding_memory(detail) == usize::MAX {
            return Err(error("record error encoding memory overflows"));
        }
        let mut cause_chain = exact_slots(detail.cause_chain.len())?;
        for cause in &detail.cause_chain {
            cause_chain.push(CauseRow {
                message: copy_text(&cause.message)?,
                code: cause.code.as_deref().map(copy_text).transpose()?,
                source: cause.source.as_deref().map(copy_text).transpose()?,
            });
        }
        Ok(Self {
            message: copy_text(&detail.message)?,
            code: detail.code.as_deref().map(copy_text).transpose()?,
            source: detail.source.as_deref().map(copy_text).transpose()?,
            cause_chain,
            stacktrace_attachment: detail
                .stacktrace_attachment
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
            trace_id: detail
                .trace_id
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
            correlation_id: detail
                .correlation_id
                .as_ref()
                .map(|id| copy_text(id.as_str()))
                .transpose()?,
        })
    }

    fn projection_memory(&self) -> usize {
        let mut bytes = std::mem::size_of::<crate::records::RecordErrorDetail>()
            .saturating_add(self.message.capacity())
            .saturating_add(
                self.cause_chain
                    .len()
                    .saturating_mul(std::mem::size_of::<dokime_domain::model::diagnostics::ErrorCause>()),
            );
        for value in [
            &self.code,
            &self.source,
            &self.stacktrace_attachment,
            &self.trace_id,
            &self.correlation_id,
        ] {
            bytes = bytes.saturating_add(value.as_ref().map_or(0, String::capacity));
        }
        for cause in &self.cause_chain {
            bytes = bytes.saturating_add(cause.message.capacity());
            for value in [&cause.code, &cause.source] {
                bytes = bytes.saturating_add(value.as_ref().map_or(0, String::capacity));
            }
        }
        bytes
    }
}

impl TryFrom<ErrorRow> for crate::records::RecordErrorDetail {
    type Error = Error;

    fn try_from(row: ErrorRow) -> Result<Self> {
        use dokime_domain::model::identity::ArtifactId;
        use dokime_domain::model::identity::CorrelationId;
        use dokime_domain::model::diagnostics::ErrorCause;
        use dokime_domain::model::identity::TraceId;
        let mut cause_chain = exact_slots(row.cause_chain.len())?;
        for cause in row.cause_chain {
            cause_chain.push(ErrorCause {
                message: cause.message,
                code: cause.code,
                source: cause.source,
            });
        }
        Ok(Self {
            message: row.message,
            code: row.code,
            source: row.source,
            cause_chain,
            stacktrace_attachment: row
                .stacktrace_attachment
                .map(ArtifactId::from_uuid)
                .transpose()?,
            trace_id: row.trace_id.map(TraceId::from_uuid).transpose()?,
            correlation_id: row
                .correlation_id
                .map(CorrelationId::from_uuid)
                .transpose()?,
        })
    }
}

/* === Public Functions === */
pub fn encode(entry: &RecordEntry, max_bytes: usize) -> Result<Vec<u8>> {
    let row = RecordRow::from_entry(entry)?;
    if row.encoded_len() > max_bytes {
        return Err(error("record capture frame exceeds byte limit"));
    }
    Ok(row.encode_to_vec())
}

pub fn decode(bytes: &[u8], max_bytes: usize) -> Result<RecordEntry> {
    RecordRow::from_bytes(bytes, max_bytes)?.entry()
}

pub fn schema() -> SchemaKey {
    SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: Identifier::new("dokime.capture.record").unwrap(),
        version: 1.try_into().unwrap(),
    }
}

/* === Private Functions === */
fn excess(bytes: usize, owned: usize, inline: usize) -> usize {
    if owned == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_add(owned.saturating_sub(inline))
    }
}
fn severity_name(value: crate::records::RecordSeverity) -> &'static str {
    use crate::records::RecordSeverity;
    match value {
        RecordSeverity::Debug => "debug",
        RecordSeverity::Info => "info",
        RecordSeverity::Warning => "warning",
        RecordSeverity::Error => "error",
    }
}
fn exact_slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| error("record allocation failed"))?;
    if values.capacity() > count {
        return Err(error("record slot capacity exceeds admission"));
    }
    Ok(values)
}
fn copy_text(source: &str) -> Result<String> {
    let mut text = String::new();
    text.try_reserve_exact(source.len())
        .map_err(|_| error("record text allocation failed"))?;
    if text.capacity() > source.len() {
        return Err(error("record text capacity exceeds admission"));
    }
    text.push_str(source);
    Ok(text)
}

fn error(reason: &'static str) -> Error {
    Error::Invalid(reason)
}

/* === Tests === */
