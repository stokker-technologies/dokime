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
mod payload;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Third Party Imports
use harmos::operations::{Context, Operation};
use tonic::Status;
// Local Imports
use super::{
    execution,
    value::{self, Budget, canonical},
};
use dokime_app::core::{CaseId, CommandId};
use dokime_app::definition::{
    CaseMaterializationOptions, CaseMaterializationSelection, SuiteRegistry, TestCase, TestSuite,
    materialize_template_cases,
};
use dokime_app::{Error, RuntimeHandle};
use dokime_contract::operations::planning as wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Register {
    runtime: Arc<RuntimeHandle>,
}

struct Preview {
    runtime: Arc<RuntimeHandle>,
}

pub(super) struct ResolvedPlan {
    pub(super) suite: TestSuite,
    pub(super) cases: Vec<TestCase>,
    pub(super) preview: Vec<wire::PlannedCase>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Register(Register),
    Preview(Preview),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Register {
    type Declaration = wire::RegisterSelection;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(runtime: Arc<RuntimeHandle>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::RegisterSelectionRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::PreviewSelectionReply>, Status> {
        dokime_contract::core::command::validate_idempotency_key(&request.idempotency_key)
            .map_err(|error| Status::invalid_argument(error.to_string()))?;
        if request
            .plan
            .as_ref()
            .is_some_and(|plan| !plan.case_ids.is_empty())
        {
            return Err(Status::invalid_argument(
                "registration requires template selections",
            ));
        }
        let snapshot = self.runtime.watch_state().latest();
        let registration_id = CommandId::from_seed(format!(
            "suite-registration:{}:{}",
            snapshot.state().identity.node.node_id,
            request.idempotency_key,
        ));
        let resolved = resolve(
            &snapshot.state().suites,
            request.plan.take(),
            &registration_id,
        )?;
        self.runtime
            .register_cases(registration_id, resolved.cases)
            .await
            .map_err(execution::status)?;
        Ok((wire::PreviewSelectionReply {
            cases: resolved.preview,
            observed_cursor: Some(canonical::cursor(&self.runtime.watch_state().latest())),
        })
        .into())
    }
}

impl Operation for Preview {
    type Declaration = wire::PreviewSelection;
    type Resources = RuntimeHandle;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    fn new(runtime: Arc<RuntimeHandle>) -> Result<Self, Status> {
        Ok(Self { runtime })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::PreviewSelectionRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::PreviewSelectionReply>, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let resolved = resolve(
            &snapshot.state().suites,
            request.plan.take(),
            &CommandId::new(),
        )?;
        Ok((wire::PreviewSelectionReply {
            cases: resolved.preview,
            observed_cursor: Some(canonical::cursor(&snapshot)),
        })
        .into())
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle) -> Result<(), super::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, RuntimeHandle>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(runtime.clone()),
        )?;
    Ok(())
}

pub(super) fn resolve(
    registry: &SuiteRegistry,
    plan: Option<wire::SelectionPlan>,
    registration_id: &CommandId,
) -> Result<ResolvedPlan, Status> {
    let plan = plan.ok_or_else(|| Status::invalid_argument("selection plan is required"))?;
    if !plan.case_ids.is_empty() {
        return resolve_registered(registry, plan);
    }
    value::collection(plan.selections.len())?;
    if plan.selections.is_empty() {
        return Err(Status::invalid_argument(
            "at least one selection is required",
        ));
    }
    let limit = if plan.max_cases == 0 {
        1000
    } else {
        plan.max_cases as usize
    };
    if limit > 1000 {
        return Err(Status::resource_exhausted("plan exceeds 1000 case limit"));
    }
    let mut budget = Budget::new();
    let mut output_budget = Budget::new();
    let mut expression_budget = bounds::Budget::new();
    let mut payload_budget = payload::BUDGET;
    let mut suite: Option<TestSuite> = None;
    let mut target = None;
    let mut cases = Vec::new();
    let mut preview = Vec::new();
    let mut templates = BTreeSet::new();
    for (selection_index, selection) in plan.selections.into_iter().enumerate() {
        value::text(&selection.suite_id)?;
        value::text(&selection.template_id)?;
        value::collection(selection.parameters.len())?;
        value::collection(selection.files.len())?;
        let parameters = entries(
            selection
                .parameters
                .into_iter()
                .map(|entry| (entry.key, entry.value)),
        )?;
        let files = entries(
            selection
                .files
                .into_iter()
                .map(|entry| (entry.key, entry.value)),
        )?;
        let entry = registry
            .get(&selection.suite_id)
            .ok_or_else(|| rejection("suite_not_found", "registered suite not found"))?;
        let template = entry
            .suite
            .test_case_templates
            .iter()
            .find(|template| template.test_case_template_id.as_str() == selection.template_id)
            .ok_or_else(|| {
                rejection(
                    "template_not_found",
                    "registered template ID not found in suite",
                )
            })?;
        // Provider ownership uses the case's suite registration ID. Template
        // capability lookup still requires an unambiguous authored identifier.
        if registry
            .suites()
            .flat_map(|candidate| &candidate.suite.test_case_templates)
            .any(|candidate| {
                candidate.test_case_template_id != template.test_case_template_id
                    && candidate.metadata.identifier == template.metadata.identifier
            })
        {
            return Err(rejection(
                "ambiguous_registered_identity",
                "selected template has ambiguous native lookup identity",
            ));
        }
        let workflow = entry.workflow.as_ref().ok_or_else(|| {
            rejection(
                "provider_unavailable",
                "selected suite has no execution workflow",
            )
        })?;
        let selected_target = (&entry.source, workflow);
        if target.is_some_and(|target| target != selected_target) {
            return Err(rejection(
                "mixed_provider_targets",
                "all selections must share one provider source and workflow",
            ));
        }
        target = Some(selected_target);
        value::collection(template.parameters.len())?;
        for parameter in &template.parameters {
            if !parameters.contains_key(parameter.metadata.identifier.as_str()) {
                bounds::expression(&parameter.expression, &mut expression_budget, 0)?;
            }
        }
        let combined = suite.get_or_insert_with(|| TestSuite {
            test_case_templates: vec![],
            ..entry.suite.clone()
        });
        if templates.insert(template.test_case_template_id.clone()) {
            combined.test_case_templates.push(template.clone());
        }
        let parameters = parameters
            .into_iter()
            .map(|(name, values)| {
                value::text(&name)?;
                value::collection(values.values.len())?;
                Ok((
                    name,
                    values
                        .values
                        .into_iter()
                        .map(|v| canonical::decode(v, &mut budget, 0))
                        .collect::<Result<Vec<_>, Status>>()?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, Status>>()?;
        let files = files
            .into_iter()
            .map(|(name, file)| {
                value::text(&name)?;
                let file = match file.kind {
                    Some(wire::SelectedFileKind::Value(value)) => {
                        value::text(&value)?;
                        Some(value)
                    }
                    Some(wire::SelectedFileKind::Clear) => None,
                    _ => {
                        return Err(Status::invalid_argument(
                            "file selection kind must be value or clear",
                        ));
                    }
                };
                Ok((name, file))
            })
            .collect::<Result<BTreeMap<_, _>, Status>>()?;
        let case_cost = payload::case_cost(
            template,
            &entry.suite.metadata,
            &entry.suite_id,
            &parameters,
            &files,
            &mut payload_budget,
        )?;
        let payload_limit = payload_budget / case_cost;
        let case_limit = (limit - cases.len()).min(payload_limit);
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
                registration_id: CommandId::from_seed(format!(
                    "{registration_id}:selection:{selection_index}"
                )),
            },
        )
        .map_err(|error| {
            if matches!(
                error,
                dokime_app::ApiError::CaseMaterialization(
                    dokime_app::definition::CaseMaterializationError::CaseLimitExceeded { .. }
                )
            ) && payload_limit < limit - cases.len()
            {
                payload::limit()
            } else {
                rejection("invalid_selection", error.to_string())
            }
        })?;
        payload::charge(
            &mut payload_budget,
            generated.test_cases.len().saturating_mul(case_cost),
        )?;
        for mut case in generated.test_cases {
            case.suite = Some(entry.suite.metadata.clone());
            case.generation_index = cases.len() as u64;
            let parameters = canonical::object(&case.parameters, &mut output_budget)?;
            preview.push(wire::PlannedCase {
                case_id: case.test_case_id.to_string(),
                suite_id: selection.suite_id.clone(),
                template_id: selection.template_id.clone(),
                metadata: Some(canonical::metadata(&case.metadata)?),
                parameters: Some(parameters),
                ordinal: cases.len() as u64,
                selection_index: selection_index as u64,
            });
            cases.push(case);
        }
    }
    if cases.is_empty() {
        return Err(rejection("empty_selection", "selection generated no cases"));
    }
    let suite = suite.expect("nonempty selections");
    Ok(ResolvedPlan {
        suite,
        cases,
        preview,
    })
}

/* === Private Functions === */
fn entries<T>(
    entries: impl Iterator<Item = (String, Option<T>)>,
) -> Result<BTreeMap<String, T>, Status> {
    let mut values = BTreeMap::new();
    for (key, value) in entries {
        value::text(&key)?;
        let value =
            value.ok_or_else(|| Status::invalid_argument("selection entry has no value"))?;
        if values.insert(key, value).is_some() {
            return Err(Status::invalid_argument("duplicate selection key"));
        }
    }
    Ok(values)
}

fn resolve_registered(
    registry: &SuiteRegistry,
    plan: wire::SelectionPlan,
) -> Result<ResolvedPlan, Status> {
    if !plan.selections.is_empty() {
        return Err(Status::invalid_argument(
            "case IDs and template selections are mutually exclusive",
        ));
    }
    let limit = if plan.max_cases == 0 {
        1000
    } else {
        plan.max_cases as usize
    };
    if limit > 1000 || plan.case_ids.len() > limit {
        return Err(Status::resource_exhausted("plan exceeds its case limit"));
    }
    let mut suite: Option<TestSuite> = None;
    let mut target = None;
    let mut templates = BTreeSet::new();
    let mut budget = Budget::new();
    let mut payload_budget = payload::BUDGET;
    let mut cases = Vec::new();
    let mut preview = Vec::new();

    for id in plan.case_ids {
        value::text(&id)?;
        let id =
            CaseId::from_wire(id).map_err(|error| Status::invalid_argument(error.to_string()))?;
        let case = registry
            .case(&id)
            .ok_or_else(|| Status::not_found("registered case not found"))?;
        let entry = registry.suite_for_case(case).ok_or_else(|| {
            rejection("provider_unavailable", "case owning suite is not installed")
        })?;
        if case.suite.as_ref() != Some(&entry.suite.metadata) {
            return Err(rejection(
                "invalid_selection",
                "case owning suite differs from installed definition",
            ));
        }
        let template = entry
            .suite
            .test_case_templates
            .iter()
            .find(|template| template.test_case_template_id == case.test_case_template_id)
            .ok_or_else(|| rejection("template_not_found", "case template is not installed"))?;
        let workflow = entry.workflow.as_ref().ok_or_else(|| {
            rejection(
                "provider_unavailable",
                "case suite has no execution workflow",
            )
        })?;
        let selected_target = (&entry.source, workflow);
        if target.is_some_and(|target| target != selected_target) {
            return Err(rejection(
                "mixed_provider_targets",
                "all cases must share one provider source and workflow",
            ));
        }
        target = Some(selected_target);
        payload::registered_case(case, &mut payload_budget)?;
        let parameters = canonical::object(&case.parameters, &mut budget)?;
        let combined = suite.get_or_insert_with(|| TestSuite {
            test_case_templates: vec![],
            ..entry.suite.clone()
        });
        if templates.insert(template.test_case_template_id.clone()) {
            combined.test_case_templates.push(template.clone());
        }
        preview.push(wire::PlannedCase {
            case_id: id.to_string(),
            suite_id: entry.suite_id.clone(),
            template_id: template.test_case_template_id.to_string(),
            metadata: Some(canonical::metadata(&case.metadata)?),
            parameters: Some(parameters),
            ordinal: cases.len() as u64,
            selection_index: cases.len() as u64,
        });
        cases.push(case.clone());
    }
    Ok(ResolvedPlan {
        suite: suite.expect("nonempty registered case IDs"),
        cases,
        preview,
    })
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
