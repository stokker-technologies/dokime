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

pub(super) use parameters::{unit, wire as parameters};

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeSet, sync::Arc};

// Third Party Imports
use dokime::harmos::operations::{Context, Operation};
use tonic::Status;

// Local Imports
use super::value::{Budget, canonical};
use dokime::RuntimeHandle;
use dokime::domain::model::definition::wire as schema;
use dokime::domain::operations::catalog as wire;

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

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListSuites, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::catalog::wire::ListSuitesReply>, Status> {
        let reply = list_suites(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

impl Operation for GetSuite {
    type Declaration = wire::GetSuite;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetSuite, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::catalog::wire::SuiteDefinition>, Status> {
        let reply = get_suite(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

impl Operation for ListRecordSchemas {
    type Declaration = wire::ListRecordSchemas;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListRecordSchemas, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::catalog::wire::ListRecordSchemasReply>, Status> {
        let reply = list_record_schemas(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

impl Operation for GetRecordSchema {
    type Declaration = wire::GetRecordSchema;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetRecordSchema, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<schema::RecordSchema>, Status> {
        let reply = get_record_schema(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

impl Operation for ListTelemetrySchemas {
    type Declaration = wire::ListTelemetrySchemas;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::ListTelemetrySchemas, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<dokime::domain::model::catalog::wire::ListTelemetrySchemasReply>, Status> {
        let reply = list_telemetry_schemas(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

impl Operation for GetTelemetrySchema {
    type Declaration = wire::GetTelemetrySchema;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::GetTelemetrySchema, _: &mut (),
    ) -> Result<dokime::harmos::operations::Response<schema::TelemetrySchema>, Status> {
        let reply = get_telemetry_schema(&self.resources, request).await?;
        check(&self.resources, crate::grpc::transcode::Transcode::wire_len(&reply)?)?;
        Ok(reply.into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime::app::Lanes::Open,
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
        Err(Status::resource_exhausted("canonical catalog construction exceeds message memory limit"))
    } else {
        Ok(())
    }
}

fn page(
    limit: u32, after: &str, given: Option<&dokime::domain::model::execution::wire_cursor::Cursor>,
    snapshot: &dokime::runtime::query::RuntimeStateSnapshot,
) -> Result<usize, Status> {
    canonical::cursor(snapshot).page(limit, after, given).map_err(canonical::cursor_failure)
}

async fn list_suites(
    resources: &Resources, request: &wire::ListSuites,
) -> Result<dokime::domain::model::catalog::wire::ListSuitesReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(request.limit, &request.after_suite_id, request.cursor.as_ref(), &snapshot)?;
    if !request.after_suite_id.is_empty() && snapshot.state().suites.get(&request.after_suite_id).is_none() {
        return Err(Status::invalid_argument("after suite ID not found"));
    }
    let mut entries = snapshot.state().suites.suites().filter(|suite| suite.suite_id > request.after_suite_id);
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut suites = canonical::slots(rows.len())?;
    for entry in rows {
        suites.push(identity(entry)?);
    }
    let more = entries.next().is_some();
    let next_after_suite_id = if more {
        let id = &suites.last().unwrap().suite_id;
        Some(id.clone())
    } else {
        None
    };
    Ok(dokime::domain::model::catalog::wire::ListSuitesReply {
        suites,
        next_after_suite_id,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_suite(
    resources: &Resources, request: &wire::GetSuite,
) -> Result<dokime::domain::model::catalog::wire::SuiteDefinition, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let entry = snapshot.state().suites.get(&request.suite_id).ok_or_else(|| Status::not_found("registered suite not found"))?;
    let mut budget = Budget::new()?;
    let templates = collect(entry.suite.test_case_templates.iter().map(|template| {
        Ok(dokime::domain::model::catalog::wire::TemplateDefinition {
            template_id: template.test_case_template_id.to_string(),
            metadata: Some(canonical::metadata(&template.metadata)?),
            parameters: parameters(&template.parameters, &mut budget)?,
            tags: strings(&template.tags)?,
            files: files(&template.files, &mut budget)?,
            steps: steps(&template.phases)?,
            parameter_groups: groups(&template.parameter_groups)?,
        })
    }))?;
    Ok(dokime::domain::model::catalog::wire::SuiteDefinition {
        identity: Some(identity(entry)?),
        templates,
        tags: suite_tags(entry)?,
        parameter_groups: groups(&entry.suite.parameter_groups)?,
    })
}

async fn list_record_schemas(
    resources: &Resources, request: &wire::ListRecordSchemas,
) -> Result<dokime::domain::model::catalog::wire::ListRecordSchemasReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(request.limit, &request.after_record_type, request.cursor.as_ref(), &snapshot)?;
    let contracts = &snapshot.state().record_definitions;
    if !request.after_record_type.is_empty() && !contracts.values().any(|c| c.record_type.as_str() == request.after_record_type) {
        return Err(Status::invalid_argument("after record type not found"));
    }
    let mut entries = contracts.values().filter(|c| c.record_type.as_str() > request.after_record_type.as_str());
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut schemas = canonical::slots(rows.len())?;
    for entry in rows {
        schemas.push(dokime::domain::model::catalog::wire::RecordSchemaIdentity {
            record_type: entry.record_type.to_string(),
            schema_version: entry.schema_version,
            metadata: Some(canonical::metadata(&entry.metadata)?),
        });
    }
    let next_after_record_type = if entries.next().is_some() {
        let id = &schemas.last().unwrap().record_type;
        Some(id.clone())
    } else {
        None
    };
    Ok(dokime::domain::model::catalog::wire::ListRecordSchemasReply {
        schemas,
        next_after_record_type,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_record_schema(resources: &Resources, request: &wire::GetRecordSchema) -> Result<schema::RecordSchema, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let contracts = &snapshot.state().record_definitions;
    let contract = contracts
        .values()
        .find(|c| c.record_type.as_str() == request.record_type)
        .ok_or_else(|| Status::not_found("registered record schema not found"))?;
    schemas::record(contract)
}

async fn list_telemetry_schemas(
    resources: &Resources, request: &wire::ListTelemetrySchemas,
) -> Result<dokime::domain::model::catalog::wire::ListTelemetrySchemasReply, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let limit = page(request.limit, &request.after_stream, request.cursor.as_ref(), &snapshot)?;
    let contracts = &snapshot.state().telemetry_definitions;
    if !request.after_stream.is_empty() && !contracts.values().any(|c| c.stream.as_str() == request.after_stream) {
        return Err(Status::invalid_argument("after stream not found"));
    }
    let mut entries = contracts.values().filter(|c| c.stream.as_str() > request.after_stream.as_str());
    let rows: Vec<_> = entries.by_ref().take(limit).collect();
    let mut schemas = canonical::slots(rows.len())?;
    for entry in rows {
        schemas.push(dokime::domain::model::catalog::wire::TelemetrySchemaIdentity {
            stream: entry.stream.to_string(),
            schema_version: entry.schema_version,
            metadata: Some(canonical::metadata(&entry.metadata)?),
        });
    }
    let next_after_stream = if entries.next().is_some() {
        let id = &schemas.last().unwrap().stream;
        Some(id.clone())
    } else {
        None
    };
    Ok(dokime::domain::model::catalog::wire::ListTelemetrySchemasReply {
        schemas,
        next_after_stream,
        cursor: Some(canonical::cursor(&snapshot)),
    })
}

async fn get_telemetry_schema(resources: &Resources, request: &wire::GetTelemetrySchema) -> Result<schema::TelemetrySchema, Status> {
    let snapshot = resources.runtime.watch_state().latest();
    let contracts = &snapshot.state().telemetry_definitions;
    let contract = contracts
        .values()
        .find(|c| c.stream.as_str() == request.stream)
        .ok_or_else(|| Status::not_found("registered telemetry schema not found"))?;
    schemas::telemetry(contract, resources.runtime.schema_settings())
}

fn identity(entry: &dokime::domains::execution::SuiteSnapshot) -> Result<dokime::domain::model::catalog::wire::SuiteIdentity, Status> {
    Ok(dokime::domain::model::catalog::wire::SuiteIdentity {
        suite_id: entry.suite_id.clone(),
        source: entry.source.clone(),
        metadata: Some(canonical::metadata(&entry.suite.metadata)?),
    })
}

fn strings(values: &[String]) -> Result<Vec<String>, Status> {
    Ok(values.to_vec())
}

fn groups(
    values: &[dokime::definition::ParameterGroup],
) -> Result<Vec<dokime::domain::model::catalog::wire::ParameterGroupDefinition>, Status> {
    collect(values.iter().map(|group| {
        Ok(dokime::domain::model::catalog::wire::ParameterGroupDefinition {
            group_id: group.group_id.to_string(),
            metadata: Some(canonical::metadata(&group.metadata)?),
        })
    }))
}

fn steps(
    values: &[dokime::definition::ExecutionPhaseDefinition],
) -> Result<Vec<dokime::domain::model::catalog::wire::StepDefinition>, Status> {
    collect(values.iter().map(|step| {
        Ok(dokime::domain::model::catalog::wire::StepDefinition {
            step_id: step.step_id.to_string(),
            repeatable: step.repeatable,
            metadata: Some(canonical::metadata(&step.metadata)?),
        })
    }))
}

fn files(
    values: &[dokime::definition::FileDefinition], budget: &mut Budget,
) -> Result<Vec<dokime::domain::model::catalog::wire::FileParameterDefinition>, Status> {
    collect(values.iter().map(|file| {
        let attributes = canonical::object(&file.attributes, budget)?;
        Ok(dokime::domain::model::catalog::wire::FileParameterDefinition {
            metadata: Some(canonical::metadata(&file.metadata)?),
            default_value: file.default.clone(),
            required: file.required,
            media_types: strings(&file.media_types)?,
            attributes: Some(attributes),
        })
    }))
}

fn tag_refs(entry: &dokime::definition::SuiteSnapshot) -> Result<BTreeSet<&String>, Status> {
    let tags: BTreeSet<_> = entry.tags.iter().chain(&entry.suite.tags).collect();
    Ok(tags)
}

fn suite_tags(entry: &dokime::definition::SuiteSnapshot) -> Result<Vec<String>, Status> {
    let tags = tag_refs(entry)?;
    let mut values = canonical::slots(tags.len())?;
    for tag in tags {
        values.push(tag.clone());
    }
    Ok(values)
}

/* === Tests === */
