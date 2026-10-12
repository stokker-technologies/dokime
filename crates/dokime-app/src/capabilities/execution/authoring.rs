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
use std::{
    any::{Any, type_name},
    collections::BTreeMap,
    time::Duration,
};
// Third Party Imports
use serde::{Deserialize, Serialize};
use thiserror::Error;
// Local Imports
use crate::definition::ParameterSource;
use crate::records::RecordSeverity;
use crate::{
    error::Error,
    evidence::{PublishReceipt, RecordPublishRequest},
    handle::RuntimeHandle,
    parameters::ScopedParameterUpdate,
};
use dokime::core::{Identifier, RecordScope, Value};
use dokime::telemetry::{SamplingScope, TelemetryCaptureLease, TelemetrySamplingRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionFileTarget {
    pub relative_path: String,
    pub absolute_path: String,
    #[serde(default)]
    pub text_hint: bool,
    #[serde(default)]
    pub streamed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRegexMatch {
    pub full_match: String,
    #[serde(default)]
    pub captures: Vec<Option<String>>,
    #[serde(default)]
    pub named_captures: BTreeMap<String, String>,
    pub byte_offset: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u64>,
}

pub struct SidecarStepContext {
    runtime: RuntimeHandle,
    scope: RecordScope,
    parameters: BTreeMap<String, Value>,
    resources: RuntimeResourceStore,
}

pub struct RuntimeResource<T> {
    pub resource_id: String,
    pub resource_kind: String,
    pub resource_name: String,
    pub value: T,
    pub details: BTreeMap<String, Value>,
}

pub struct RuntimeResourceStore {
    runtime: RuntimeHandle,
    scope: RecordScope,
    source: String,
    entries: BTreeMap<String, RuntimeResourceEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeResourceMetadata {
    pub resource_id: String,
    pub resource_kind: String,
    pub resource_name: String,
    pub details: BTreeMap<String, Value>,
}

struct RuntimeResourceEntry {
    metadata: RuntimeResourceMetadata,
    lifecycle: crate::records::ResourceLifecycle,
    handle: Box<dyn Any + Send + Sync>,
}

/* === Enums === */
#[derive(Debug, Error)]
pub enum StepContextError {
    #[error("resource {resource_id} is not stored")]
    MissingResource { resource_id: String },

    #[error("resource {resource_id} is not a {expected_type}")]
    ResourceTypeMismatch {
        resource_id: String,
        expected_type: &'static str,
    },

    #[error("runtime rejected {operation}: {message}")]
    RecordRejected {
        operation: &'static str,
        code: String,
        message: String,
    },

    #[error("{capability} requires execution scope")]
    MissingExecutionScope { capability: &'static str },

    #[error("execution file {relative_path:?} was not found")]
    ExecutionFileNotFound { relative_path: String },

    #[error("runtime operation failed: {0}")]
    Runtime(String),
}

/* === Traits === */

/* === Implementations === */
impl SidecarStepContext {
    pub fn new(runtime: RuntimeHandle, scope: RecordScope) -> Self {
        Self::with_source(runtime, scope, "runtime.step")
    }

    pub fn with_source(
        runtime: RuntimeHandle,
        scope: RecordScope,
        source: impl Into<String>,
    ) -> Self {
        let source = source.into();

        Self {
            resources: RuntimeResourceStore::new(runtime.clone(), scope.clone(), source.clone()),
            runtime,
            scope,
            parameters: BTreeMap::new(),
        }
    }

    pub fn scope(&self) -> &RecordScope {
        &self.scope
    }

    pub fn resources(&mut self) -> &mut RuntimeResourceStore {
        &mut self.resources
    }

    pub fn set_local_parameters(&mut self, parameters: BTreeMap<String, Value>) {
        self.parameters = parameters;
    }

    pub fn set_local_parameter(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.parameters.insert(key.into(), value.into());
    }

    pub fn get_local_parameter(&self, key: &str) -> Option<&Value> {
        self.parameters.get(key)
    }

    pub fn get_local_parameters(&self) -> &BTreeMap<String, Value> {
        &self.parameters
    }

    pub async fn set_runtime_parameter_async(
        &self,
        key: impl Into<String>,
        value: impl Into<Value>,
    ) -> Result<bool, StepContextError> {
        Ok(self
            .runtime
            .upsert_scoped_parameter(
                &self.scope,
                ScopedParameterUpdate::new(key, value, ParameterSource::Runtime),
            )
            .await)
    }

    pub async fn set_traceability_parameter_async(
        &self,
        key: impl Into<String>,
        value: impl Into<Value>,
    ) -> Result<bool, StepContextError> {
        Ok(self
            .runtime
            .upsert_scoped_parameter(
                &self.scope,
                ScopedParameterUpdate::new(key, value, ParameterSource::Traceability)
                    .with_group_id(traceability_group_id()),
            )
            .await)
    }

    pub async fn publish_record_async(
        &self,
        record_type: impl Into<String>,
        severity: RecordSeverity,
        message: impl Into<String>,
        details: BTreeMap<String, Value>,
    ) -> Result<PublishReceipt, StepContextError> {
        let mut request = RecordPublishRequest::new(identifier(record_type.into()), severity);
        request.scope = Some(self.scope.clone());
        request.message = Some(message.into());
        request.details = details;
        self.runtime
            .publish_record(request)
            .await
            .map_err(|error| StepContextError::record_rejected("publish record", error))
    }

    pub async fn wait_async(&self, duration: Duration) -> Result<(), StepContextError> {
        tokio::select! {
            _ = tokio::time::sleep(duration) => {}
            _ = self.runtime.wait_until_cycle_stopped(self.scope.test_cycle_id.as_ref()) => {}
        }
        Ok(())
    }

    pub async fn wait_until_stopped_async(&self) -> Result<(), StepContextError> {
        self.runtime
            .wait_until_cycle_stopped(self.scope.test_cycle_id.as_ref())
            .await;
        Ok(())
    }
    pub async fn capture_async<S>(
        &self,
        params: TelemetrySamplingRequest,
        sampler: S,
    ) -> Result<TelemetryCaptureLease, StepContextError>
    where
        S: crate::telemetry_runtime::Sampler,
    {
        self.runtime
            .request_sampling(sampling_scope(&self.scope), params, sampler)
            .await
            .map_err(StepContextError::from)
    }

    pub async fn capture_host_async(
        &self,
        params: TelemetrySamplingRequest,
    ) -> Result<TelemetryCaptureLease, StepContextError> {
        leased_host_sampling(&self.runtime, sampling_scope(&self.scope), params)
            .await
            .map_err(StepContextError::from)
    }

    pub async fn stop_async(
        &self,
        capture: &TelemetryCaptureLease,
    ) -> Result<bool, StepContextError> {
        self.runtime
            .stop_telemetry_capture(&capture.lease_id)
            .await
            .map_err(StepContextError::from)
    }
    pub async fn target_async(
        &self,
        relative_path: impl Into<String>,
    ) -> Result<ExecutionFileTarget, StepContextError> {
        self.target_with_options_async(relative_path, true, true)
            .await
    }

    pub async fn target_with_options_async(
        &self,
        relative_path: impl Into<String>,
        text_hint: bool,
        streamed: bool,
    ) -> Result<ExecutionFileTarget, StepContextError> {
        let relative_path = relative_path.into();
        let test_execution_id = self.execution_id("execution files")?;
        self.runtime
            .execution_file_target(
                &test_execution_id,
                relative_path.clone(),
                text_hint,
                streamed,
            )
            .await?
            .ok_or(StepContextError::ExecutionFileNotFound { relative_path })
    }

    pub async fn extract_regex_async(
        &self,
        relative_path: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Result<Vec<FileRegexMatch>, StepContextError> {
        self.extract_regex_inner(relative_path.into(), pattern.into(), false)
            .await
    }

    pub async fn extract_first_regex_async(
        &self,
        relative_path: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Result<Option<FileRegexMatch>, StepContextError> {
        self.extract_regex_inner(relative_path.into(), pattern.into(), true)
            .await
            .map(|mut matches| matches.pop())
    }

    async fn extract_regex_inner(
        &self,
        relative_path: String,
        pattern: String,
        first_only: bool,
    ) -> Result<Vec<FileRegexMatch>, StepContextError> {
        let test_execution_id = self.execution_id("execution files")?;
        self.runtime
            .extract_execution_file_regex(
                &test_execution_id,
                relative_path.clone(),
                pattern,
                first_only,
            )
            .await?
            .ok_or(StepContextError::ExecutionFileNotFound { relative_path })
    }

    fn execution_id(
        &self,
        capability: &'static str,
    ) -> Result<dokime::core::TestExecutionId, StepContextError> {
        self.scope
            .test_execution_id
            .clone()
            .ok_or(StepContextError::MissingExecutionScope { capability })
    }
}

impl<T> RuntimeResource<T> {
    pub fn new(
        resource_id: impl Into<String>,
        resource_kind: impl Into<String>,
        resource_name: impl Into<String>,
        value: T,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_kind: resource_kind.into(),
            resource_name: resource_name.into(),
            value,
            details: BTreeMap::new(),
        }
    }

    pub fn with_details(
        resource_id: impl Into<String>,
        resource_kind: impl Into<String>,
        resource_name: impl Into<String>,
        value: T,
        details: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_kind: resource_kind.into(),
            resource_name: resource_name.into(),
            value,
            details,
        }
    }

    fn into_parts(self) -> (RuntimeResourceMetadata, T) {
        (
            RuntimeResourceMetadata {
                resource_id: self.resource_id,
                resource_kind: self.resource_kind,
                resource_name: self.resource_name,
                details: self.details,
            },
            self.value,
        )
    }
}

impl RuntimeResourceStore {
    fn new(runtime: RuntimeHandle, scope: RecordScope, source: impl Into<String>) -> Self {
        Self {
            runtime,
            scope,
            source: source.into(),
            entries: BTreeMap::new(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, resource_id: &str) -> bool {
        self.entries.contains_key(resource_id)
    }

    pub fn metadata(&self, resource_id: &str) -> Option<&RuntimeResourceMetadata> {
        self.entries.get(resource_id).map(|entry| &entry.metadata)
    }

    pub fn get<T: Send + Sync + 'static>(&self, resource_id: &str) -> Option<&T> {
        self.entries
            .get(resource_id)
            .and_then(|entry| entry.handle.downcast_ref())
    }

    pub fn get_mut<T: Send + Sync + 'static>(&mut self, resource_id: &str) -> Option<&mut T> {
        self.entries
            .get_mut(resource_id)
            .and_then(|entry| entry.handle.downcast_mut())
    }

    pub async fn set_async<T: Send + Sync + 'static>(
        &mut self,
        resource: RuntimeResource<T>,
    ) -> Result<PublishReceipt, StepContextError> {
        let (metadata, value) = resource.into_parts();
        let receipt = self
            .publish_resource_record(
                "resource.registered",
                RecordSeverity::Info,
                format!("Resource {} registered.", metadata.resource_name),
                &metadata,
            )
            .await?;
        self.entries.insert(
            metadata.resource_id.clone(),
            RuntimeResourceEntry {
                metadata,
                lifecycle: crate::records::ResourceLifecycle::Registered,
                handle: Box::new(value),
            },
        );

        Ok(receipt)
    }

    pub async fn mark_opened_async(
        &mut self,
        resource_id: &str,
        details: BTreeMap<String, Value>,
    ) -> Result<PublishReceipt, StepContextError> {
        self.mark_lifecycle(
            resource_id,
            crate::records::ResourceLifecycle::Opened,
            "resource.opened",
            RecordSeverity::Info,
            details,
        )
        .await
    }

    pub async fn mark_failed_async(
        &mut self,
        resource_id: &str,
        error: impl Into<String>,
        mut details: BTreeMap<String, Value>,
    ) -> Result<PublishReceipt, StepContextError> {
        details.insert("error".to_owned(), Value::from(error.into()));
        self.mark_lifecycle(
            resource_id,
            crate::records::ResourceLifecycle::Failed,
            "resource.failed",
            RecordSeverity::Error,
            details,
        )
        .await
    }

    pub async fn close_async(
        &mut self,
        resource_id: &str,
        details: BTreeMap<String, Value>,
    ) -> Result<PublishReceipt, StepContextError> {
        self.mark_lifecycle(
            resource_id,
            crate::records::ResourceLifecycle::Closed,
            "resource.closed",
            RecordSeverity::Info,
            details,
        )
        .await
    }

    pub async fn remove_async<T: Send + Sync + 'static>(
        &mut self,
        resource_id: &str,
        details: BTreeMap<String, Value>,
    ) -> Result<Option<T>, StepContextError> {
        let Some(existing) = self.entries.get(resource_id) else {
            return Ok(None);
        };
        if !existing.handle.is::<T>() {
            return Err(StepContextError::ResourceTypeMismatch {
                resource_id: resource_id.to_owned(),
                expected_type: type_name::<T>(),
            });
        }
        let mut metadata = existing.metadata.clone();
        merge_details(&mut metadata.details, details);
        self.publish_resource_record(
            "resource.removed",
            RecordSeverity::Info,
            format!("Resource {} removed.", metadata.resource_name),
            &metadata,
        )
        .await?;
        let entry = self
            .entries
            .remove(resource_id)
            .expect("resource existed before removal event");
        let value = entry
            .handle
            .downcast::<T>()
            .expect("resource type was checked before removal");

        Ok(Some(*value))
    }

    async fn mark_lifecycle(
        &mut self,
        resource_id: &str,
        lifecycle: crate::records::ResourceLifecycle,
        record_type: &'static str,
        severity: RecordSeverity,
        details: BTreeMap<String, Value>,
    ) -> Result<PublishReceipt, StepContextError> {
        let mut metadata = self.entry_metadata(resource_id)?;
        merge_details(&mut metadata.details, details);
        let receipt = self
            .publish_resource_record(
                record_type,
                severity,
                format!("Resource {} updated.", metadata.resource_name),
                &metadata,
            )
            .await?;
        let entry = self
            .entries
            .get_mut(resource_id)
            .expect("resource metadata was cloned from an existing entry");
        entry.metadata = metadata;
        entry.lifecycle = lifecycle;

        Ok(receipt)
    }

    fn entry_metadata(
        &self,
        resource_id: &str,
    ) -> Result<RuntimeResourceMetadata, StepContextError> {
        self.entries
            .get(resource_id)
            .map(|entry| entry.metadata.clone())
            .ok_or_else(|| StepContextError::MissingResource {
                resource_id: resource_id.to_owned(),
            })
    }

    async fn publish_resource_record(
        &self,
        record_type: &'static str,
        severity: RecordSeverity,
        message: String,
        metadata: &RuntimeResourceMetadata,
    ) -> Result<PublishReceipt, StepContextError> {
        let mut request = RecordPublishRequest::new(identifier(record_type), severity);
        request.scope = Some(self.scope.clone());
        request.message = Some(message);
        request.details = resource_record_details(metadata);

        self.runtime
            .publish_record(request)
            .await
            .map_err(|error| StepContextError::record_rejected("publish resource record", error))
    }
}

impl StepContextError {
    fn record_rejected(operation: &'static str, error: crate::Error) -> Self {
        Self::RecordRejected {
            operation,
            code: error.code().unwrap_or("internal").to_owned(),
            message: error.to_string(),
        }
    }
}

impl From<Error> for StepContextError {
    fn from(error: Error) -> Self {
        Self::Runtime(error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn leased_host_sampling(
    runtime: &RuntimeHandle,
    scope: SamplingScope,
    params: TelemetrySamplingRequest,
) -> crate::Result<TelemetryCaptureLease> {
    runtime
        .request_host_sampling(scope, vec![params])
        .await?
        .pop()
        .ok_or_else(|| crate::Error::Operation("host telemetry capture was not leased".to_owned()))
}

fn resource_record_details(metadata: &RuntimeResourceMetadata) -> BTreeMap<String, Value> {
    let mut details = metadata.details.clone();
    details.insert(
        "resource_id".to_owned(),
        Value::from(metadata.resource_id.clone()),
    );
    details.insert(
        "resource_kind".to_owned(),
        Value::from(metadata.resource_kind.clone()),
    );
    details.insert(
        "resource_name".to_owned(),
        Value::from(metadata.resource_name.clone()),
    );
    details
}

fn merge_details(target: &mut BTreeMap<String, Value>, source: BTreeMap<String, Value>) {
    for (key, value) in source {
        target.insert(key, value);
    }
}

fn sampling_scope(scope: &RecordScope) -> SamplingScope {
    SamplingScope {
        test_cycle_id: scope.test_cycle_id.as_ref().map(ToString::to_string),
        test_case_id: scope.test_case_id.as_ref().map(ToString::to_string),
        test_execution_id: scope.test_execution_id.as_ref().map(ToString::to_string),
    }
}

fn identifier(value: impl Into<String>) -> Identifier {
    Identifier::new(value).expect("runtime-authored identifier is valid")
}

fn traceability_group_id() -> Option<Identifier> {
    Some(identifier("traceability.runtime"))
}

/* === Tests === */
