/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Runtime Regex Admission Before Pure Prompt And Catalog Transactions.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::plugins::InstalledPlugin;
use dokime_domain::model::definition::SuiteSet;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::prompts::{PromptDefinition, checks::collect_definitions};
use crate::runtime::Error;
use crate::state::RuntimeState;
use crate::transactions::prompts::{self, RequestPrompt, SubmitPromptResponse};

/* === Constants === */

/* === Public Functions === */
/// Untrusted declarations are checked before the pure installation transaction.
pub(crate) fn validate_installation(installation: &InstalledPlugin) -> Result<(), String> {
    validate_definitions(&installation.source, installation.prompts.iter().chain(
        installation.suites.suites.iter().flat_map(|suite| &suite.suite.test_case_templates)
            .flat_map(|template| &template.prompt_definitions)))
}

pub(crate) fn validate_suites(source: &str, suites: &SuiteSet) -> Result<(), String> {
    validate_definitions(source, suites.suites.iter().flat_map(|suite| &suite.suite.test_case_templates)
        .flat_map(|template| &template.prompt_definitions))
}

pub(crate) fn validate_request(state: &RuntimeState, request: &RequestPrompt) -> Result<PromptId, Error> {
    let definition = state.prompt_definitions.get(request.prompt.prompt_type.as_str());
    if let Some(definition) = definition {
        let issues = super::validate_definition(definition);
        if !issues.is_empty() { return Err(Error::validation("validation_failed", "prompt definition validation failed", issues)); }
    }
    let checked = prompts::validate_request(state, request);
    let values_reached = checked.as_ref().err().is_none_or(|error| {
        error.code() == Some("prompt_already_exists") || matches!(error,
            Error::Validation { message, .. } if message == "invalid question initial values" || message == "prompt record validation failed")
    });
    if values_reached && let Some(definition) = definition {
        let mut fields = definition.fields.clone();
        for field in &mut fields { field.required = false; }
        let issues = super::validate_fields(&fields, &request.prompt.values);
        if !issues.is_empty() { return Err(Error::validation("validation_failed", "invalid question initial values", issues)); }
    }
    checked
}

pub(crate) fn validate_response(state: &RuntimeState, response: &SubmitPromptResponse) -> Result<PromptId, Error> {
    let checked = prompts::validate_response(state, response);
    let values_reached = checked.as_ref().err().is_none_or(|error| matches!(error,
        Error::Validation { message, .. } if message == "prompt response validation failed" || message == "prompt record validation failed"));
    if response.accepted && values_reached && let Some(prompt) = state.prompt_requests.get(&response.prompt_id) {
        let issues = super::validate_answers(&prompt.definition, &response.values);
        if !issues.is_empty() { return Err(Error::validation("validation_failed", "prompt response validation failed", issues)); }
    }
    checked
}

/* === Private Functions === */
fn validate_definitions<'a>(source: &str, definitions: impl IntoIterator<Item = &'a PromptDefinition>) -> Result<(), String> {
    for definition in collect_definitions(source, definitions)? {
        let issues = super::validate_definition(definition);
        if !issues.is_empty() {
            return Err(format!("invalid prompt {} from {}: {}", definition.prompt_type, source,
                issues.into_iter().map(|issue| issue.message).collect::<Vec<_>>().join("; ")));
        }
    }
    Ok(())
}

/* === Tests === */
