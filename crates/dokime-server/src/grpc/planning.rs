/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod bounds;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use dokime::harmos::{Context, RegistrationExt};
use tonic::Status;

// Local Imports
use super::transcode::Transcode;
use super::{
    execution,
    value::{self, Budget, canonical},
};
use dokime::definition::{CaseMaterializationOptions, CaseMaterializationSelection, TestCase, materialize_template_cases};
use dokime::domain::model::identity::CaseId;
use dokime::domain::model::identity::CommandId;
use dokime::domain::model::identity::TemplateId;
use dokime::domain::model::identity::TestCycleId;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::planning as wire;
use dokime::state::RuntimeState;
use dokime::{RuntimeHandle, runtime::Error};

/* === Constants === */

const PARAMETER_MATERIALIZED_CASES: u64 = 1_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

pub(super) struct ResolvedPlan {
    pub(super) cases: Vec<TestCase>,
    pub(super) preview: Vec<dokime::domain::model::planning::wire::PlannedCase>,
    pub(super) revisions: BTreeMap<CaseId, String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<wire::MaterializeTemplate> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::MaterializeTemplate, _: Context<'a, wire::MaterializeTemplate>,
    ) -> Served<'a, dokime::domain::model::planning::wire::PreviewSelectionReply> {
        Box::pin(async move {
            let selection = request.selection.take().ok_or_else(|| Status::invalid_argument("template selection is required"))?;
            let registration_id = dokime::model::identity::new_id::<CommandId>();
            let snapshot = self.runtime.watch_state().latest();
            let resolved = resolve(
                snapshot.state(),
                Some(dokime::domain::model::planning::wire::SelectionPlan {
                    selections: vec![selection],
                    ..Default::default()
                }),
                &registration_id,
            )?;
            let mut reply = checked_preview(
                dokime::domain::model::planning::wire::PreviewSelectionReply {
                    cases: resolved.preview,
                    observed_cursor: Some(canonical::cursor(&snapshot)),
                },
                self.maximum,
            )?;
            self.runtime.register_cases_at(registration_id, resolved.cases, resolved.revisions).await.map_err(execution::status)?;
            reply.observed_cursor = Some(canonical::cursor(&self.runtime.watch_state().latest()));
            Ok::<_, Status>(checked_preview(reply, self.maximum)?.into())
        })
    }
}

impl Serve<wire::RegisterSelection> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::RegisterSelection, _: Context<'a, wire::RegisterSelection>,
    ) -> Served<'a, dokime::domain::model::planning::wire::PreviewSelectionReply> {
        Box::pin(async move {
            dokime::domain::model::execution::command::validate_idempotency_key(&request.idempotency_key)
                .map_err(|error| Status::invalid_argument(error.to_string()))?;
            if request.plan.as_ref().is_some_and(|plan| !plan.case_ids.is_empty()) {
                return Err(Status::invalid_argument("registration requires template selections"));
            }
            let snapshot = self.runtime.watch_state().latest();
            let registration_id =
                CommandId::from_seed(format!("suite-registration:{}:{}", snapshot.state().identity.node.node_id, request.idempotency_key,));
            let resolved = resolve(snapshot.state(), request.plan.take(), &registration_id)?;
            let mut reply = checked_preview(
                dokime::domain::model::planning::wire::PreviewSelectionReply {
                    cases: resolved.preview,
                    observed_cursor: Some(canonical::cursor(&snapshot)),
                },
                self.maximum,
            )?;
            self.runtime.register_cases_at(registration_id, resolved.cases, resolved.revisions).await.map_err(execution::status)?;
            reply.observed_cursor = Some(canonical::cursor(&self.runtime.watch_state().latest()));
            Ok::<_, Status>(checked_preview(reply, self.maximum)?.into())
        })
    }
}

impl Serve<wire::PreviewSelection> for Resources {
    fn serve<'a>(
        &'a self, mut request: wire::PreviewSelection, _: Context<'a, wire::PreviewSelection>,
    ) -> Served<'a, dokime::domain::model::planning::wire::PreviewSelectionReply> {
        Box::pin(async move {
            let snapshot = self.runtime.watch_state().latest();
            let resolved = resolve(snapshot.state(), request.plan.take(), &dokime::model::identity::new_id::<CommandId>())?;
            Ok::<_, Status>(
                checked_preview(
                    dokime::domain::model::planning::wire::PreviewSelectionReply {
                        cases: resolved.preview,
                        observed_cursor: Some(canonical::cursor(&snapshot)),
                    },
                    self.maximum,
                )?
                .into(),
            )
        })
    }
}

/* === Public Functions === */

pub(super) async fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::MaterializeTemplate>::new(served.clone()))?;
    dependencies.insert(Host::<wire::RegisterSelection>::new(served.clone()))?;
    dependencies.insert(Host::<wire::PreviewSelection>::new(served.clone()))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::MaterializeTemplate>()
        .operation::<wire::RegisterSelection>()
        .operation::<wire::PreviewSelection>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(super) fn resolve(
    state: &RuntimeState, plan: Option<dokime::domain::model::planning::wire::SelectionPlan>, registration_id: &CommandId,
) -> Result<ResolvedPlan, Status> {
    let registry = &state.suites;
    let plan = plan.ok_or_else(|| Status::invalid_argument("selection plan is required"))?;
    if !plan.case_ids.is_empty() {
        return resolve_registered(state, plan);
    }
    let mut label_numbers = next_label_numbers(state, &plan.cycle_id)?;
    value::collection(plan.selections.len())?;
    if plan.selections.is_empty() {
        return Err(Status::invalid_argument("at least one selection is required"));
    }
    let case_cap = usize::try_from(PARAMETER_MATERIALIZED_CASES).expect("validated materialized case cap");
    let limit = if plan.max_cases == 0 { case_cap } else { plan.max_cases as usize };
    if limit > case_cap {
        return Err(Status::resource_exhausted("plan exceeds configured materialized case limit"));
    }
    let mut budget = Budget::new()?;
    let mut output_budget = Budget::new()?;
    let mut expression_budget = bounds::Budget::new().map_err(bounds::status)?;
    let mut target = None;
    let mut cases = Vec::new();
    let mut preview = Vec::new();
    for (selection_index, selection) in plan.selections.into_iter().enumerate() {
        value::collection(selection.parameters.len())?;
        value::collection(selection.files.len())?;
        let parameters = entries(selection.parameters.into_iter().map(|entry| (entry.key, entry.value)))?;
        let files = entries(selection.files.into_iter().map(|entry| (entry.key, entry.value)))?;
        let entry = registry.get(&selection.suite_id).ok_or_else(|| rejection("suite_not_found", "registered suite not found"))?;
        let template = entry
            .suite
            .test_case_templates
            .iter()
            .find(|template| template.test_case_template_id.as_str() == selection.template_id)
            .ok_or_else(|| rejection("template_not_found", "registered template ID not found in suite"))?;
        // Provider ownership uses the case's suite registration ID. Template
        // domain lookup still requires an unambiguous authored identifier.
        if registry.suites().flat_map(|candidate| &candidate.suite.test_case_templates).any(|candidate| {
            candidate.test_case_template_id != template.test_case_template_id
                && candidate.metadata.identifier == template.metadata.identifier
        }) {
            return Err(rejection("ambiguous_registered_identity", "selected template has ambiguous native lookup identity"));
        }
        let workflow =
            entry.workflow.as_ref().ok_or_else(|| rejection("provider_unavailable", "selected suite has no execution workflow"))?;
        let selected_target = (&entry.source, workflow);
        if target.is_some_and(|target| target != selected_target) {
            return Err(rejection("mixed_provider_targets", "all selections must share one provider source and workflow"));
        }
        target = Some(selected_target);
        for parameter in &template.parameters {
            if !parameters.contains_key(parameter.metadata.identifier.as_str()) {
                bounds::expression(&parameter.expression, &mut expression_budget, 0)?;
            }
        }
        let parameters = parameters
            .into_iter()
            .map(|(name, values)| {
                value::collection(values.values.len())?;
                let values = values
                    .values
                    .into_iter()
                    .map(|value| {
                        let value = canonical::decode(value, &mut budget, 0)?;
                        expression_budget.value(&value, 0).map_err(bounds::status)?;
                        Ok(value)
                    })
                    .collect::<Result<Vec<_>, Status>>()?;
                Ok((name, values))
            })
            .collect::<Result<BTreeMap<_, _>, Status>>()?;
        let files = files
            .into_iter()
            .map(|(name, file)| {
                let file = match file.kind {
                    Some(dokime::domain::model::planning::wire::SelectedFileKind::Value(value)) => Some(value),
                    Some(dokime::domain::model::planning::wire::SelectedFileKind::Clear) => None,
                    _ => {
                        return Err(Status::invalid_argument("file selection kind must be value or clear"));
                    }
                };
                Ok((name, file))
            })
            .collect::<Result<BTreeMap<_, _>, Status>>()?;
        let case_limit = limit - cases.len();
        let generated = materialize_template_cases(
            &entry.suite_id,
            template,
            CaseMaterializationSelection {
                case_overrides: parameters,
                file_overrides: files,
                ..Default::default()
            },
            CaseMaterializationOptions {
                max_cases: case_limit,
                force: false,
                registration_id: CommandId::from_seed(format!("{registration_id}:selection:{selection_index}")),
                first_label_number: *label_numbers.entry(template.test_case_template_id.clone()).or_insert(1),
            },
        )
        .map_err(|error| rejection("invalid_selection", error.to_string()))?;
        *label_numbers.entry(template.test_case_template_id.clone()).or_insert(1) += generated.test_cases.len();
        let definition_revision = generated
            .test_cases
            .first()
            .map(|case| {
                let mut case = case.clone();
                case.suite = Some(entry.suite.metadata.clone());
                state.case_definition_revision(&case)
            })
            .transpose()
            .map_err(Status::failed_precondition)?
            .unwrap_or_default();
        for mut case in generated.test_cases {
            case.suite = Some(entry.suite.metadata.clone());
            case.generation_index = cases.len() as u64;
            let parameters = canonical::object(&case.parameters, &mut output_budget).map_err(payload_failure)?;
            preview.push(dokime::domain::model::planning::wire::PlannedCase {
                case_id: case.test_case_id.to_string(),
                suite_id: selection.suite_id.clone(),
                template_id: selection.template_id.clone(),
                metadata: Some(canonical::metadata(&case.metadata)?),
                parameters: Some(parameters),
                ordinal: cases.len() as u64,
                selection_index: selection_index as u64,
                definition_revision: definition_revision.clone(),
                provider: entry.source.clone(),
            });
            cases.push(case);
        }
    }
    if cases.is_empty() {
        return Err(rejection("empty_selection", "selection generated no cases"));
    }
    Ok(resolved(cases, preview))
}

/* === Private Functions === */

fn checked_preview(
    reply: dokime::domain::model::planning::wire::PreviewSelectionReply, maximum: usize,
) -> Result<dokime::domain::model::planning::wire::PreviewSelectionReply, Status> {
    if reply.wire_len().map_err(Status::from)? > maximum {
        return Err(rejection("planning_payload_limit", "plan exceeds its encoded response limit"));
    }
    Ok(reply)
}

fn payload_failure(error: Status) -> Status {
    if error.code() == tonic::Code::ResourceExhausted { rejection("planning_payload_limit", error.message()) } else { error }
}

fn entries<T>(entries: impl Iterator<Item = (String, Option<T>)>) -> Result<BTreeMap<String, T>, Status> {
    let mut values = BTreeMap::new();
    for (key, value) in entries {
        let value = value.ok_or_else(|| Status::invalid_argument("selection entry has no value"))?;
        if values.insert(key, value).is_some() {
            return Err(Status::invalid_argument("duplicate selection key"));
        }
    }
    Ok(values)
}

// Each template's next label number, continuing the stored case ordinals of
// the cycle the plan extends.
fn next_label_numbers(state: &RuntimeState, cycle_id: &str) -> Result<BTreeMap<TemplateId, usize>, Status> {
    let mut numbers = BTreeMap::new();
    if cycle_id.is_empty() {
        return Ok(numbers);
    }
    let id = TestCycleId::from_uuid(cycle_id.to_owned()).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
    let cycle = state.cycles.get(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
    for case in state.cases_for_cycle(cycle) {
        // The Stored Ordinal, Not The Label: An Operator May Rename "X #2"
        if case.label_number > 0 {
            let next = numbers.entry(case.test_case_template_id.clone()).or_insert(1);
            *next = (*next).max(case.label_number as usize + 1);
        }
    }
    Ok(numbers)
}

fn resolve_registered(state: &RuntimeState, plan: dokime::domain::model::planning::wire::SelectionPlan) -> Result<ResolvedPlan, Status> {
    let registry = &state.suites;
    if !plan.selections.is_empty() {
        return Err(Status::invalid_argument("case IDs and template selections are mutually exclusive"));
    }
    if !plan.cycle_id.is_empty() {
        return Err(Status::invalid_argument("registered cases keep their labels; cycle_id numbers template selections"));
    }
    let case_cap = usize::try_from(PARAMETER_MATERIALIZED_CASES).expect("validated materialized case cap");
    let limit = if plan.max_cases == 0 { case_cap } else { plan.max_cases as usize };
    if limit > case_cap || plan.case_ids.len() > limit {
        return Err(Status::resource_exhausted("plan exceeds its case limit"));
    }
    let mut target = None;
    let mut budget = Budget::new()?;
    let mut cases = Vec::new();
    let mut preview = Vec::new();

    for id in plan.case_ids {
        let id = CaseId::from_wire(id).map_err(|error| Status::invalid_argument(error.to_string()))?;
        let case = registry.case(&id).ok_or_else(|| Status::not_found("registered case not found"))?;
        let entry = registry.suite_for_case(case).ok_or_else(|| rejection("provider_unavailable", "case owning suite is not installed"))?;
        if case.suite.as_ref() != Some(&entry.suite.metadata) {
            return Err(rejection("invalid_selection", "case owning suite differs from installed definition"));
        }
        let template = entry
            .suite
            .test_case_templates
            .iter()
            .find(|template| template.test_case_template_id == case.test_case_template_id)
            .ok_or_else(|| rejection("template_not_found", "case template is not installed"))?;
        let workflow = entry.workflow.as_ref().ok_or_else(|| rejection("provider_unavailable", "case suite has no execution workflow"))?;
        let selected_target = (&entry.source, workflow);
        if target.is_some_and(|target| target != selected_target) {
            return Err(rejection("mixed_provider_targets", "all cases must share one provider source and workflow"));
        }
        target = Some(selected_target);
        let parameters = canonical::object(&case.parameters, &mut budget)?;
        preview.push(dokime::domain::model::planning::wire::PlannedCase {
            case_id: id.to_string(),
            suite_id: entry.suite_id.clone(),
            template_id: template.test_case_template_id.to_string(),
            metadata: Some(canonical::metadata(&case.metadata)?),
            parameters: Some(parameters),
            ordinal: cases.len() as u64,
            selection_index: cases.len() as u64,
            definition_revision: state.case_definition_revision(case).map_err(Status::failed_precondition)?,
            provider: entry.source.clone(),
        });
        cases.push(case.clone());
    }
    Ok(resolved(cases, preview))
}

fn resolved(cases: Vec<TestCase>, preview: Vec<dokime::domain::model::planning::wire::PlannedCase>) -> ResolvedPlan {
    let revisions = cases.iter().zip(&preview).map(|(case, row)| (case.test_case_id.clone(), row.definition_revision.clone())).collect();
    ResolvedPlan { cases, preview, revisions }
}

fn rejection(code: &str, message: impl Into<String>) -> Status {
    let error = if matches!(code, "suite_not_found" | "template_not_found") {
        Error::not_found(code, message)
    } else {
        Error::validation(code, message, vec![])
    };
    execution::status(error)
}

/* === Tests === */
