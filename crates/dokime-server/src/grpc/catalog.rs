/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod parameters;
pub(super) mod schemas;

/* === Re-Exports === */
pub(super) use parameters::{memory as parameters_memory, unit, unit_memory, wire as parameters};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use harmos::operations::{Context, Operation};
use std::{collections::BTreeSet, sync::Arc};
use tonic::Status;
// Local Imports
use super::value::{self, Budget, canonical};
use dokime_runtime::RuntimeHandle;
use dokime_contract::operations::{catalog as wire, common, schema, value as wire_value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    max_message_bytes: usize,
}
struct ListSuites {
    resources: Arc<Resources>,
}
struct GetSuite {
    resources: Arc<Resources>,
}
struct ListRecordSchemas {
    resources: Arc<Resources>,
}
struct GetRecordSchema {
    resources: Arc<Resources>,
}
struct ListTelemetrySchemas {
    resources: Arc<Resources>,
}
struct GetTelemetrySchema {
    resources: Arc<Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    ListSuites(ListSuites),
    GetSuite(GetSuite),
    ListRecordSchemas(ListRecordSchemas),
    GetRecordSchema(GetRecordSchema),
    ListTelemetrySchemas(ListTelemetrySchemas),
    GetTelemetrySchema(GetTelemetrySchema),
}

/* === Traits === */

/* === Implementations === */
impl Operation for ListSuites {
    type Declaration = wire::ListSuites;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListSuitesRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListSuitesReply>, Status> {
        list_suites(&self.resources, request).await.map(Into::into)
    }
}

impl Operation for GetSuite {
    type Declaration = wire::GetSuite;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetSuiteRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::SuiteDefinition>, Status> {
        get_suite(&self.resources, request).await.map(Into::into)
    }
}

impl Operation for ListRecordSchemas {
    type Declaration = wire::ListRecordSchemas;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListRecordSchemasRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListRecordSchemasReply>, Status> {
        list_record_schemas(&self.resources, request)
            .await
            .map(Into::into)
    }
}

impl Operation for GetRecordSchema {
    type Declaration = wire::GetRecordSchema;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetRecordSchemaRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<schema::RecordSchema>, Status> {
        get_record_schema(&self.resources, request)
            .await
            .map(Into::into)
    }
}

impl Operation for ListTelemetrySchemas {
    type Declaration = wire::ListTelemetrySchemas;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListTelemetrySchemasRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListTelemetrySchemasReply>, Status> {
        list_telemetry_schemas(&self.resources, request)
            .await
            .map(Into::into)
    }
}

impl Operation for GetTelemetrySchema {
    type Declaration = wire::GetTelemetrySchema;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetTelemetrySchemaRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<schema::TelemetrySchema>, Status> {
        get_telemetry_schema(&self.resources, request)
            .await
            .map(Into::into)
    }
}
/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), super::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, Resources>(
            "dokime",
            dokime_runtime::app::Lanes::Open,
            Arc::new(Resources {
                runtime: runtime.clone(),
                max_message_bytes,
            }),
        )?;
    Ok(())
}

/* === Private Functions === */
fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = canonical::slots(source.len())?;
    for value in source {
        target.push(value?);
    }
    Ok(target)
}

fn check(resources: &Resources, bytes: usize) -> Result<(), Status> {
    if bytes == usize::MAX || bytes > resources.max_message_bytes {
        Err(Status::resource_exhausted(
            "canonical catalog construction exceeds message memory limit",
        ))
    } else {
        Ok(())
    }
}

fn page(
    limit: u32,
    after: &str,
    given: Option<&dokime_contract::operations::cursor::Cursor>,
    snapshot: &dokime_runtime::runtime::query::RuntimeStateSnapshot,
) -> Result<usize, Status> {
    canonical::cursor(snapshot)
        .page(limit, after, given)
        .map_err(canonical::cursor_failure)
}

fn metadata_heap(metadata: &dokime_runtime::core::Metadata) -> Result<usize, Status> {
    Ok(canonical::metadata_memory(metadata)? - std::mem::size_of::<common::Metadata>())
}

fn identity_memory(entry: &dokime_runtime::domains::execution::SuiteSnapshot) -> Result<usize, Status> {
    value::text(&entry.suite_id)?;
    value::text(&entry.source)?;
    Ok(std::mem::size_of::<wire::SuiteIdentity>()
        + entry.suite_id.len()
        + entry.source.len()
        + metadata_heap(&entry.suite.metadata)?)
}

fn strings_memory(values: &[String]) -> Result<usize, Status> {
    value::collection(values.len())?;
    let mut bytes = values.len().saturating_mul(std::mem::size_of::<String>());
    for value in values {
        value::text(value)?;
        bytes = bytes.saturating_add(value.len());
    }
    Ok(bytes)
}

fn groups_memory(values: &[dokime_runtime::definition::ParameterGroup]) -> Result<usize, Status> {
    value::collection(values.len())?;
    let mut bytes = values
        .len()
        .saturating_mul(std::mem::size_of::<wire::ParameterGroupDefinition>());
    for value in values {
        bytes = bytes
            .saturating_add(value.group_id.as_str().len())
            .saturating_add(metadata_heap(&value.metadata)?);
    }
    Ok(bytes)
}

fn suite_memory(entry: &dokime_runtime::domains::execution::SuiteSnapshot) -> Result<usize, Status> {
    value::collection(entry.suite.test_case_templates.len())?;
    let mut bytes = std::mem::size_of::<wire::SuiteDefinition>()
        .saturating_add(identity_memory(entry)? - std::mem::size_of::<wire::SuiteIdentity>())
        .saturating_add(suite_tags_memory(entry)?)
        .saturating_add(groups_memory(&entry.suite.parameter_groups)?)
        .saturating_add(
            entry
                .suite
                .test_case_templates
                .len()
                .saturating_mul(std::mem::size_of::<wire::TemplateDefinition>()),
        );
    let mut budget = Budget::new();
    for template in &entry.suite.test_case_templates {
        bytes = bytes
            .saturating_add(template.test_case_template_id.as_str().len())
            .saturating_add(metadata_heap(&template.metadata)?)
            .saturating_add(
                parameters_memory(&template.parameters, &mut budget).map_err(|_| {
                    Status::resource_exhausted("canonical parameter construction exceeds budget")
                })? - std::mem::size_of::<Vec<wire::ParameterSchema>>(),
            )
            .saturating_add(strings_memory(&template.tags)?)
            .saturating_add(groups_memory(&template.parameter_groups)?);
        value::collection(template.phases.len())?;
        bytes = bytes.saturating_add(
            template
                .phases
                .len()
                .saturating_mul(std::mem::size_of::<wire::StepDefinition>()),
        );
        for step in &template.phases {
            bytes = bytes
                .saturating_add(step.step_id.as_str().len())
                .saturating_add(metadata_heap(&step.metadata)?);
        }
        value::collection(template.files.len())?;
        bytes = bytes.saturating_add(
            template
                .files
                .len()
                .saturating_mul(std::mem::size_of::<wire::FileParameterDefinition>()),
        );
        for file in &template.files {
            if let Some(default) = &file.default {
                value::text(default)?;
            }
            bytes = bytes
                .saturating_add(metadata_heap(&file.metadata)?)
                .saturating_add(file.default.as_ref().map_or(0, String::len))
                .saturating_add(strings_memory(&file.media_types)?)
                .saturating_add(
                    canonical::object_memory(&file.attributes, &mut budget)?
                        - std::mem::size_of::<wire_value::Value>(),
                );
        }
    }
    Ok(bytes)
}

async fn list_suites(
    resources: &Resources,
    request: &wire::ListSuitesRequest,
) -> Result<wire::ListSuitesReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(
        request.limit,
        &request.after_suite_id,
        request.cursor.as_ref(),
        &snapshot,
    )?;
    if !request.after_suite_id.is_empty()
        && snapshot
            .state()
            .suites
            .get(&request.after_suite_id)
            .is_none()
    {
        return Err(Status::invalid_argument("after suite ID not found"));
    }
    let mut entries = snapshot
        .state()
        .suites
        .suites()
        .filter(|suite| suite.suite_id > request.after_suite_id);
    let mut bytes = std::mem::size_of::<wire::ListSuitesReply>()
        + canonical::cursor(&snapshot).epoch.len()
        + "runtime.state".len();
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut suites = canonical::slots(rows.len())?;
    for entry in rows {
        bytes = bytes.saturating_add(identity_memory(entry)?);
        check(resources, bytes)?;
        suites.push(identity(entry)?);
    }
    let more = entries.next().is_some();
    let next_after_suite_id = if more {
        let id = &suites.last().unwrap().suite_id;
        check(resources, bytes.saturating_add(id.len()))?;
        Some(id.clone())
    } else {
        None
    };
    Ok(wire::ListSuitesReply {
        suites,
        next_after_suite_id,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_suite(
    resources: &Resources,
    request: &wire::GetSuiteRequest,
) -> Result<wire::SuiteDefinition, Status> {
    value::text(&request.suite_id)?;
    let snapshot = resources.runtime.watch_state().latest();
    let entry = snapshot
        .state()
        .suites
        .get(&request.suite_id)
        .ok_or_else(|| Status::not_found("registered suite not found"))?;
    value::collection(entry.suite.test_case_templates.len())?;
    check(resources, suite_memory(entry)?)?;
    let mut budget = Budget::new();
    let templates = collect(entry.suite.test_case_templates.iter().map(|template| {
        Ok(wire::TemplateDefinition {
            template_id: template.test_case_template_id.to_string(),
            metadata: Some(canonical::metadata(&template.metadata)?),
            parameters: parameters(&template.parameters, &mut budget)?,
            tags: strings(&template.tags)?,
            files: files(&template.files, &mut budget)?,
            steps: steps(&template.phases)?,
            parameter_groups: groups(&template.parameter_groups)?,
            continue_on_failure: template.continue_on_failure,
        })
    }))?;
    Ok(wire::SuiteDefinition {
        identity: Some(identity(entry)?),
        templates,
        tags: suite_tags(entry)?,
        parameter_groups: groups(&entry.suite.parameter_groups)?,
    })
}

async fn list_record_schemas(
    resources: &Resources,
    request: &wire::ListRecordSchemasRequest,
) -> Result<wire::ListRecordSchemasReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(
        request.limit,
        &request.after_record_type,
        request.cursor.as_ref(),
        &snapshot,
    )?;
    let contracts = &snapshot.state().record_definitions;
    if !request.after_record_type.is_empty()
        && !contracts
            .values()
            .any(|c| c.record_type.as_str() == request.after_record_type)
    {
        return Err(Status::invalid_argument("after record type not found"));
    }
    let mut entries = contracts
        .values()
        .filter(|c| c.record_type.as_str() > request.after_record_type.as_str());
    let mut bytes = std::mem::size_of::<wire::ListRecordSchemasReply>()
        + canonical::cursor(&snapshot).epoch.len()
        + "runtime.state".len();
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut schemas = canonical::slots(rows.len())?;
    for entry in rows {
        value::text(entry.record_type.as_str())?;
        bytes = bytes
            .saturating_add(std::mem::size_of::<wire::RecordSchemaIdentity>())
            .saturating_add(entry.record_type.as_str().len())
            .saturating_add(
                canonical::metadata_memory(&entry.metadata)?
                    - std::mem::size_of::<common::Metadata>(),
            );
        check(resources, bytes)?;
        schemas.push(wire::RecordSchemaIdentity {
            record_type: entry.record_type.to_string(),
            schema_version: entry.schema_version,
            metadata: Some(canonical::metadata(&entry.metadata)?),
        });
    }
    let next_after_record_type = if entries.next().is_some() {
        let id = &schemas.last().unwrap().record_type;
        check(resources, bytes.saturating_add(id.len()))?;
        Some(id.clone())
    } else {
        None
    };
    Ok(wire::ListRecordSchemasReply {
        schemas,
        next_after_record_type,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_record_schema(
    resources: &Resources,
    request: &wire::GetRecordSchemaRequest,
) -> Result<schema::RecordSchema, Status> {
    value::text(&request.record_type)?;
    let snapshot = resources.runtime.watch_state().latest();
    let contracts = &snapshot.state().record_definitions;
    let contract = contracts
        .values()
        .find(|c| c.record_type.as_str() == request.record_type)
        .ok_or_else(|| Status::not_found("registered record schema not found"))?;
    check(resources, schemas::record_memory(contract)?)?;
    schemas::record(contract)
}

async fn list_telemetry_schemas(
    resources: &Resources,
    request: &wire::ListTelemetrySchemasRequest,
) -> Result<wire::ListTelemetrySchemasReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(
        request.limit,
        &request.after_stream,
        request.cursor.as_ref(),
        &snapshot,
    )?;
    let contracts = &snapshot.state().telemetry_definitions;
    if !request.after_stream.is_empty()
        && !contracts
            .values()
            .any(|c| c.stream.as_str() == request.after_stream)
    {
        return Err(Status::invalid_argument("after stream not found"));
    }
    let mut entries = contracts
        .values()
        .filter(|c| c.stream.as_str() > request.after_stream.as_str());
    let mut bytes = std::mem::size_of::<wire::ListTelemetrySchemasReply>()
        + canonical::cursor(&snapshot).epoch.len()
        + "runtime.state".len();
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut schemas = canonical::slots(rows.len())?;
    for entry in rows {
        value::text(entry.stream.as_str())?;
        bytes = bytes
            .saturating_add(std::mem::size_of::<wire::TelemetrySchemaIdentity>())
            .saturating_add(entry.stream.as_str().len())
            .saturating_add(
                canonical::metadata_memory(&entry.metadata)?
                    - std::mem::size_of::<common::Metadata>(),
            );
        check(resources, bytes)?;
        schemas.push(wire::TelemetrySchemaIdentity {
            stream: entry.stream.to_string(),
            schema_version: entry.schema_version,
            metadata: Some(canonical::metadata(&entry.metadata)?),
        });
    }
    let next_after_stream = if entries.next().is_some() {
        let id = &schemas.last().unwrap().stream;
        check(resources, bytes.saturating_add(id.len()))?;
        Some(id.clone())
    } else {
        None
    };
    Ok(wire::ListTelemetrySchemasReply {
        schemas,
        next_after_stream,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_telemetry_schema(
    resources: &Resources,
    request: &wire::GetTelemetrySchemaRequest,
) -> Result<schema::TelemetrySchema, Status> {
    value::text(&request.stream)?;
    let snapshot = resources.runtime.watch_state().latest();
    let contracts = &snapshot.state().telemetry_definitions;
    let contract = contracts
        .values()
        .find(|c| c.stream.as_str() == request.stream)
        .ok_or_else(|| Status::not_found("registered telemetry schema not found"))?;
    check(resources, schemas::telemetry_memory(contract)?)?;
    schemas::telemetry(contract)
}

fn identity(
    entry: &dokime_runtime::domains::execution::SuiteSnapshot,
) -> Result<wire::SuiteIdentity, Status> {
    Ok(wire::SuiteIdentity {
        suite_id: entry.suite_id.clone(),
        source: entry.source.clone(),
        metadata: Some(canonical::metadata(&entry.suite.metadata)?),
    })
}

fn strings(values: &[String]) -> Result<Vec<String>, Status> {
    value::collection(values.len())?;
    for item in values {
        value::text(item)?;
    }
    Ok(values.to_vec())
}

fn groups(
    values: &[dokime_runtime::definition::ParameterGroup],
) -> Result<Vec<wire::ParameterGroupDefinition>, Status> {
    value::collection(values.len())?;
    collect(values.iter().map(|group| {
        Ok(wire::ParameterGroupDefinition {
            group_id: group.group_id.to_string(),
            metadata: Some(canonical::metadata(&group.metadata)?),
        })
    }))
}

fn steps(
    values: &[dokime_runtime::definition::ExecutionPhaseDefinition],
) -> Result<Vec<wire::StepDefinition>, Status> {
    value::collection(values.len())?;
    collect(values.iter().map(|step| {
        Ok(wire::StepDefinition {
            step_id: step.step_id.to_string(),
            repeatable: step.repeatable,
            metadata: Some(canonical::metadata(&step.metadata)?),
        })
    }))
}

fn files(
    values: &[dokime_runtime::definition::FileDefinition],
    budget: &mut Budget,
) -> Result<Vec<wire::FileParameterDefinition>, Status> {
    value::collection(values.len())?;
    collect(values.iter().map(|file| {
        if let Some(default) = &file.default {
            value::text(default)?;
        }
        value::collection(file.attributes.len())?;
        let attributes = canonical::object(&file.attributes, budget)?;
        Ok(wire::FileParameterDefinition {
            metadata: Some(canonical::metadata(&file.metadata)?),
            default_value: file.default.clone(),
            required: file.required,
            media_types: strings(&file.media_types)?,
            attributes: Some(attributes),
        })
    }))
}

fn suite_tags_memory(entry: &dokime_runtime::definition::SuiteSnapshot) -> Result<usize, Status> {
    let tags = tag_refs(entry)?;
    let mut bytes = tags.len().saturating_mul(std::mem::size_of::<String>());
    for tag in tags {
        value::text(tag)?;
        bytes = bytes.saturating_add(tag.len());
    }
    Ok(bytes)
}

fn tag_refs(entry: &dokime_runtime::definition::SuiteSnapshot) -> Result<BTreeSet<&String>, Status> {
    value::collection(entry.tags.len())?;
    value::collection(entry.suite.tags.len())?;
    let tags: BTreeSet<_> = entry.tags.iter().chain(&entry.suite.tags).collect();
    value::collection(tags.len())?;
    Ok(tags)
}

fn suite_tags(entry: &dokime_runtime::definition::SuiteSnapshot) -> Result<Vec<String>, Status> {
    let tags = tag_refs(entry)?;
    let mut values = canonical::slots(tags.len())?;
    for tag in tags {
        value::text(tag)?;
        values.push(tag.clone());
    }
    Ok(values)
}

/* === Tests === */
