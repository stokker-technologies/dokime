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
use tonic::{Request, Response, Status};
// Local Imports
use super::{
    observation,
    value::{self, Budget},
};
use dokime_app::RuntimeHandle;
use dokime_protocol::v1::{self, catalog_server::Catalog};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Catalog for Service {
    async fn list_suites(
        &self,
        request: Request<v1::ListSuitesRequest>,
    ) -> Result<Response<v1::ListSuitesReply>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
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
        let suites: Vec<_> = entries
            .by_ref()
            .take(limit)
            .map(identity)
            .collect::<Result<_, Status>>()?;
        let more = entries.next().is_some();
        let next_after_suite_id = more.then(|| suites.last().unwrap().suite_id.clone());
        Ok(Response::new(v1::ListSuitesReply {
            suites,
            next_after_suite_id,
            cursor: Some(observation::cursor(&snapshot)),
        }))
    }
    async fn get_suite(
        &self,
        request: Request<v1::GetSuiteRequest>,
    ) -> Result<Response<v1::SuiteDefinition>, Status> {
        value::text(&request.get_ref().suite_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let entry = snapshot
            .state()
            .suites
            .get(&request.get_ref().suite_id)
            .ok_or_else(|| Status::not_found("registered suite not found"))?;
        value::collection(entry.suite.test_case_templates.len())?;
        let mut budget = Budget::new();
        let templates = entry
            .suite
            .test_case_templates
            .iter()
            .map(|template| {
                Ok(v1::TemplateDefinition {
                    template_id: template.test_case_template_id.to_string(),
                    metadata: Some(value::metadata(&template.metadata)?),
                    parameters: parameters(&template.parameters, &mut budget)?,
                    tags: strings(&template.tags)?,
                    files: files(&template.files, &mut budget)?,
                    steps: steps(&template.phases)?,
                    parameter_groups: groups(&template.parameter_groups)?,
                    continue_on_failure: template.continue_on_failure,
                })
            })
            .collect::<Result<_, Status>>()?;
        Ok(Response::new(v1::SuiteDefinition {
            identity: Some(identity(entry)?),
            templates,
            tags: suite_tags(entry)?,
            parameter_groups: groups(&entry.suite.parameter_groups)?,
        }))
    }
    async fn list_record_schemas(
        &self,
        request: Request<v1::ListRecordSchemasRequest>,
    ) -> Result<Response<v1::ListRecordSchemasReply>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
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
        let schemas: Vec<_> = entries
            .by_ref()
            .take(limit)
            .map(|c| {
                Ok(v1::RecordSchemaIdentity {
                    record_type: c.record_type.to_string(),
                    schema_version: c.schema_version,
                    metadata: Some(value::metadata(&c.metadata)?),
                })
            })
            .collect::<Result<_, Status>>()?;
        let next_after_record_type = entries
            .next()
            .is_some()
            .then(|| schemas.last().unwrap().record_type.clone());
        Ok(Response::new(v1::ListRecordSchemasReply {
            schemas,
            next_after_record_type,
            cursor: Some(observation::cursor(&snapshot)),
        }))
    }
    async fn get_record_schema(
        &self,
        request: Request<v1::GetRecordSchemaRequest>,
    ) -> Result<Response<v1::RecordSchema>, Status> {
        value::text(&request.get_ref().record_type)?;
        let snapshot = self.runtime.watch_state().latest();
        let contracts = &snapshot.state().record_definitions;
        let contract = contracts
            .values()
            .find(|c| c.record_type.as_str() == request.get_ref().record_type)
            .ok_or_else(|| Status::not_found("registered record schema not found"))?;
        Ok(Response::new(schemas::record(contract)?))
    }

    async fn list_telemetry_schemas(
        &self,
        request: Request<v1::ListTelemetrySchemasRequest>,
    ) -> Result<Response<v1::ListTelemetrySchemasReply>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
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
        let schemas: Vec<_> = entries
            .by_ref()
            .take(limit)
            .map(|c| {
                Ok(v1::TelemetrySchemaIdentity {
                    stream: c.stream.to_string(),
                    schema_version: c.schema_version,
                    metadata: Some(value::metadata(&c.metadata)?),
                })
            })
            .collect::<Result<_, Status>>()?;
        let next_after_stream = entries
            .next()
            .is_some()
            .then(|| schemas.last().unwrap().stream.clone());
        Ok(Response::new(v1::ListTelemetrySchemasReply {
            schemas,
            next_after_stream,
            cursor: Some(observation::cursor(&snapshot)),
        }))
    }
    async fn get_telemetry_schema(
        &self,
        request: Request<v1::GetTelemetrySchemaRequest>,
    ) -> Result<Response<v1::TelemetrySchema>, Status> {
        value::text(&request.get_ref().stream)?;
        let snapshot = self.runtime.watch_state().latest();
        let contracts = &snapshot.state().telemetry_definitions;
        let contract = contracts
            .values()
            .find(|c| c.stream.as_str() == request.get_ref().stream)
            .ok_or_else(|| Status::not_found("registered telemetry schema not found"))?;
        Ok(Response::new(schemas::telemetry(contract)?))
    }
}

/* === Public Functions === */
/* === Private Functions === */
fn identity(
    entry: &dokime_app::capabilities::execution::SuiteSnapshot,
) -> Result<v1::SuiteIdentity, Status> {
    Ok(v1::SuiteIdentity {
        suite_id: entry.suite_id.clone(),
        source: entry.source.clone(),
        metadata: Some(value::metadata(&entry.suite.metadata)?),
        product: entry.product.clone(),
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
    values: &[dokime_app::definition::ParameterGroup],
) -> Result<Vec<v1::ParameterGroupDefinition>, Status> {
    value::collection(values.len())?;
    values
        .iter()
        .map(|group| {
            Ok(v1::ParameterGroupDefinition {
                group_id: group.group_id.to_string(),
                metadata: Some(value::metadata(&group.metadata)?),
            })
        })
        .collect::<Result<_, Status>>()
}

fn steps(
    values: &[dokime_app::definition::ExecutionPhaseDefinition],
) -> Result<Vec<v1::StepDefinition>, Status> {
    value::collection(values.len())?;
    values
        .iter()
        .map(|step| {
            Ok(v1::StepDefinition {
                step_id: step.step_id.to_string(),
                repeatable: step.repeatable,
                metadata: Some(value::metadata(&step.metadata)?),
            })
        })
        .collect::<Result<_, Status>>()
}

fn files(
    values: &[dokime_app::definition::FileDefinition],
    budget: &mut Budget,
) -> Result<Vec<v1::FileParameterDefinition>, Status> {
    value::collection(values.len())?;
    values
        .iter()
        .map(|file| {
            if let Some(default) = &file.default {
                value::text(default)?;
            }
            value::collection(file.attributes.len())?;
            let fields = file
                .attributes
                .iter()
                .map(|(key, v)| {
                    value::text(key)?;
                    Ok((key.clone(), value::encode(v, budget, 0)?))
                })
                .collect::<Result<_, Status>>()?;
            Ok(v1::FileParameterDefinition {
                metadata: Some(value::metadata(&file.metadata)?),
                default_value: file.default.clone(),
                required: file.required,
                media_types: strings(&file.media_types)?,
                attributes: Some(v1::ValueObject { fields }),
            })
        })
        .collect()
}

fn suite_tags(entry: &dokime_app::definition::SuiteSnapshot) -> Result<Vec<String>, Status> {
    let mut tags = entry.tags.clone();
    tags.extend(entry.suite.tags.iter().cloned());
    tags.sort();
    tags.dedup();
    strings(&tags)
}

/* === Tests === */
