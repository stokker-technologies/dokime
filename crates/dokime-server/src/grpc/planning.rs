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
use std::collections::{BTreeMap, BTreeSet};
// Third Party Imports
use tonic::{Request, Response, Status};
// Local Imports
use super::{
    execution, observation,
    value::{self, Budget},
};
use dokime_app::core::{CaseId, CommandId};
use dokime_app::definition::{
    CaseMaterializationOptions, CaseMaterializationSelection, SuiteRegistry, TestCase, TestSuite,
    materialize_template_cases,
};
use dokime_app::{Error, RuntimeHandle};
use dokime_protocol::v1::{self, planning_server::Planning};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
}

pub(super) struct ResolvedPlan {
    pub(super) suite: TestSuite,
    pub(super) cases: Vec<TestCase>,
    pub(super) preview: Vec<v1::PlannedCase>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Planning for Service {
    async fn register_selection(
        &self,
        request: Request<v1::RegisterSelectionRequest>,
    ) -> Result<Response<v1::PreviewSelectionReply>, Status> {
        let request = request.into_inner();
        let key = &request.idempotency_key;
        if key.is_empty()
            || key.len() > 256
            || key.trim().is_empty()
            || key.chars().any(char::is_control)
        {
            return Err(Status::invalid_argument(
                "registration requires a nonblank 1–256 byte idempotency key without control characters",
            ));
        }
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
            "suite-registration:{}:{key}",
            snapshot.state().identity.node.node_id
        ));
        let resolved = resolve(&snapshot.state().suites, request.plan, &registration_id)?;
        self.runtime
            .register_cases(registration_id, resolved.cases)
            .await
            .map_err(execution::status)?;

        Ok(Response::new(v1::PreviewSelectionReply {
            cases: resolved.preview,
            observed_cursor: Some(observation::cursor(&self.runtime.watch_state().latest())),
        }))
    }

    async fn preview_selection(
        &self,
        request: Request<v1::PreviewSelectionRequest>,
    ) -> Result<Response<v1::PreviewSelectionReply>, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let resolved = resolve(
            &snapshot.state().suites,
            request.into_inner().plan,
            &CommandId::new(),
        )?;
        Ok(Response::new(v1::PreviewSelectionReply {
            cases: resolved.preview,
            observed_cursor: Some(observation::cursor(&snapshot)),
        }))
    }
}

/* === Public Functions === */
pub(super) fn resolve(
    registry: &SuiteRegistry,
    plan: Option<v1::SelectionPlan>,
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
            if !selection
                .parameters
                .contains_key(parameter.metadata.identifier.as_str())
            {
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
        value::collection(selection.parameters.len())?;
        let parameters = selection
            .parameters
            .into_iter()
            .map(|(name, values)| {
                value::text(&name)?;
                value::collection(values.values.len())?;
                Ok((
                    name,
                    values
                        .values
                        .into_iter()
                        .map(|v| value::decode(v, &mut budget, 0))
                        .collect::<Result<Vec<_>, Status>>()?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>, Status>>()?;
        value::collection(selection.files.len())?;
        let files = selection
            .files
            .into_iter()
            .map(|(name, file)| {
                value::text(&name)?;
                let file = match file.kind {
                    Some(v1::selected_file::Kind::Value(value)) => {
                        value::text(&value)?;
                        Some(value)
                    }
                    Some(v1::selected_file::Kind::Clear(0)) => None,
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
            let fields = case
                .parameters
                .iter()
                .map(|(name, v)| Ok((name.clone(), value::encode(v, &mut output_budget, 0)?)))
                .collect::<Result<_, Status>>()?;
            preview.push(v1::PlannedCase {
                case_id: case.test_case_id.to_string(),
                suite_id: selection.suite_id.clone(),
                template_id: selection.template_id.clone(),
                metadata: Some(value::metadata(&case.metadata)?),
                parameters: Some(v1::ValueObject { fields }),
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
fn resolve_registered(
    registry: &SuiteRegistry,
    plan: v1::SelectionPlan,
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
        let fields = case
            .parameters
            .iter()
            .map(|(name, value)| {
                value::text(name)?;
                Ok((name.clone(), value::encode(value, &mut budget, 0)?))
            })
            .collect::<Result<_, Status>>()?;
        let combined = suite.get_or_insert_with(|| TestSuite {
            test_case_templates: vec![],
            ..entry.suite.clone()
        });
        if templates.insert(template.test_case_template_id.clone()) {
            combined.test_case_templates.push(template.clone());
        }
        preview.push(v1::PlannedCase {
            case_id: id.to_string(),
            suite_id: entry.suite_id.clone(),
            template_id: template.test_case_template_id.to_string(),
            metadata: Some(value::checked_metadata(&case.metadata)?),
            parameters: Some(v1::ValueObject { fields }),
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
