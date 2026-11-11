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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use crate::definition::{CaseMaterializationOptions, CaseMaterializationSelection, TestCase, materialize_template_cases};
use crate::state::RuntimeState;
use crate::transactions::CycleDefinition;
use crate::{CommandContext, RuntimeHandle, runtime::Error};
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::identity::{CaseId, CommandId, Identifier, Metadata, TestCycleId, TestExecutionId};
use dokime_domain::model::values::Value;

/* === Constants === */

// The Cycle Metadata Attribute Naming The Cycle A Rerun Was Created From.
pub const RERUN_OF_ATTRIBUTE: &str = "rerun_of";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The New Cycle And What The Installed Plugin No Longer Declares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RerunCycleOutcome {
    pub cycle_id: TestCycleId,
    pub skipped: Vec<RerunCaseNotice>,
    pub dropped_parameters: Vec<RerunCaseNotice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RerunCaseNotice {
    // The Case's Authored Identifier, As Its Plugin Declares It
    pub case_id: String,
    pub suite_id: String,
    pub reason: String,
}

// Source Cases Materialized Against The Installed Suites, Before Any Commit.
struct Resolved {
    cases: Vec<TestCase>,
    revisions: BTreeMap<CaseId, String>,
    skipped: Vec<RerunCaseNotice>,
    dropped_parameters: Vec<RerunCaseNotice>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn rerun_cycle(
        &self, source_cycle_id: TestCycleId, label: Option<String>, context: CommandContext,
    ) -> Result<RerunCycleOutcome, Error> {
        // Like Copy, This Mutation Offers No Retry-Key Contract.
        if context.idempotency_key.is_some() {
            return Err(Error::validation("unsupported_idempotency", "cycle rerun does not support idempotency keys", vec![]));
        }
        let label = match label {
            Some(label) if label.trim().is_empty() => {
                return Err(Error::validation("invalid_label", "rerun label must not be blank; omit it to keep the source label", vec![]));
            }
            Some(label) => Some(label.trim().to_owned()),
            None => None,
        };
        let inspected = self.inspect_cycle(&source_cycle_id).await?;
        let source =
            inspected.state().cycle_view(&source_cycle_id).ok_or_else(|| Error::not_found("cycle_not_found", "source cycle not found"))?;
        let registration_id = crate::model::identity::new_id::<CommandId>();
        let resolved = resolve(&self.state_read(), &source, &registration_id)?;
        if resolved.cases.is_empty() {
            return Err(Error::conflict("rerun_no_cases", refusal(&source, &resolved.skipped)));
        }

        self.register_cases_at(registration_id, resolved.cases.clone(), resolved.revisions).await?;
        let mut metadata = Metadata::new(
            Identifier::new(format!("cycle.{}", crate::model::identity::new_id::<CommandId>()))
                .map_err(|error| Error::Operation(error.to_string()))?,
            label.unwrap_or_else(|| source.metadata.label.clone()),
            source.metadata.description.clone(),
        );
        metadata.tags = source.metadata.tags.clone();
        metadata.attributes = source.metadata.attributes.clone();
        metadata.attributes.insert(RERUN_OF_ATTRIBUTE.to_owned(), Value::from(source_cycle_id.to_string()));
        let cycle_id = self
            .create_cycle(
                CycleDefinition {
                    metadata,
                    continue_automatically: source.continue_automatically,
                },
                CommandContext::default(),
            )
            .await?;
        let executions =
            resolved.cases.iter().map(|case| (crate::model::identity::new_id::<TestExecutionId>(), case.test_case_id.clone())).collect();
        self.append_executions(cycle_id.clone(), executions).await?;
        Ok(RerunCycleOutcome {
            cycle_id,
            skipped: resolved.skipped,
            dropped_parameters: resolved.dropped_parameters,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

// Resolve Each Planned Case, In Ordinal Order, Against The Installed Suites.
fn resolve(state: &RuntimeState, source: &CycleView, registration_id: &CommandId) -> Result<Resolved, Error> {
    let mut resolved = Resolved {
        cases: Vec::new(),
        revisions: BTreeMap::new(),
        skipped: Vec::new(),
        dropped_parameters: Vec::new(),
    };
    for (index, case) in source.cases().enumerate() {
        let notice = |reason: String| RerunCaseNotice {
            case_id: case.metadata.identifier.as_str().to_owned(),
            suite_id: case.suite_id.clone(),
            reason,
        };
        let Some(entry) = state.suites.get(&case.suite_id) else {
            resolved.skipped.push(notice("its suite is no longer installed".to_owned()));
            continue;
        };
        let Some(template) =
            entry.suite.test_case_templates.iter().find(|template| template.test_case_template_id == case.test_case_template_id).or_else(
                || entry.suite.test_case_templates.iter().find(|template| template.metadata.identifier == case.metadata.identifier),
            )
        else {
            resolved.skipped.push(notice("its suite no longer declares this case".to_owned()));
            continue;
        };

        let parameters = template.parameters.iter().map(|parameter| parameter.metadata.identifier.as_str()).collect::<BTreeSet<_>>();
        let files = template.files.iter().map(|file| file.metadata.identifier.as_str()).collect::<BTreeSet<_>>();
        let mut selection = CaseMaterializationSelection::default();
        let mut dropped = Vec::new();
        for (name, value) in &case.parameters {
            if parameters.contains(name.as_str()) {
                selection.case_overrides.insert(name.clone(), vec![value.clone()]);
            } else if files.contains(name.as_str()) {
                // File Inputs Materialize As Text Paths.
                if let Value::Text(path) = value {
                    selection.file_overrides.insert(name.clone(), Some(path.clone()));
                }
            } else {
                dropped.push(name.as_str());
            }
        }
        if !dropped.is_empty() {
            resolved.dropped_parameters.push(notice(format!("no longer declares {}", dropped.join(", "))));
        }

        let generated = materialize_template_cases(
            &entry.suite_id,
            template,
            selection,
            CaseMaterializationOptions {
                max_cases: 1,
                force: false,
                registration_id: CommandId::from_seed(format!("{registration_id}:rerun:{index}")),
                first_label_number: usize::try_from(case.label_number.max(1)).unwrap_or(1),
            },
        );
        let mut materialized = match generated {
            Ok(output) if output.test_cases.len() == 1 => output.test_cases.into_iter().next().expect("one materialized case"),
            Ok(output) => {
                resolved.skipped.push(notice(format!("its parameters now select {} cases", output.test_cases.len())));
                continue;
            }
            Err(error) => {
                resolved.skipped.push(notice(error.to_string()));
                continue;
            }
        };
        materialized.suite = Some(entry.suite.metadata.clone());
        materialized.generation_index = resolved.cases.len() as u64;
        let revision = state.case_definition_revision(&materialized).map_err(|reason| Error::conflict("rerun_refused", reason))?;
        resolved.revisions.insert(materialized.test_case_id.clone(), revision);
        resolved.cases.push(materialized);
    }
    Ok(resolved)
}

// Name The Plugin And Every Missing Case, So The Operator Knows What To Install.
fn refusal(source: &CycleView, skipped: &[RerunCaseNotice]) -> String {
    if skipped.is_empty() {
        return "The cycle has no cases to rerun; add cases from the Library to a new cycle instead.".to_owned();
    }
    let plugins = source
        .test_executions
        .iter()
        .filter_map(|execution| execution.snapshot.plugin().map(|plugin| plugin.source.clone()))
        .collect::<BTreeSet<_>>();
    let plugins = if plugins.is_empty() { skipped.iter().map(|notice| notice.suite_id.clone()).collect::<BTreeSet<_>>() } else { plugins };
    let cases = skipped.iter().map(|notice| format!("{} ({})", notice.case_id, notice.reason)).collect::<Vec<_>>();
    format!(
        "No case of this cycle exists in the installed plugin {}: {}. Install a plugin revision that declares them, or create a new cycle from the Library.",
        plugins.into_iter().collect::<Vec<_>>().join(", "),
        cases.join("; ")
    )
}

/* === Tests === */
