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
use std::collections::BTreeMap;

// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::prompts::{
    PromptAction, PromptActionEmphasis, PromptDefinition, PromptField, PromptFieldKind, PromptInstance, PromptKind, PromptRequest,
    PromptResponse,
};
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::records::ValidationMode;
use crate::records::validate_record;
use crate::runtime::Error;
use crate::state::RuntimeState;
use crate::transactions::records::draft::Drafts;
use dokime_domain::model::diagnostics::ValidationIssue;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::PromptId;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.prompts.request", irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestPrompt {
    pub prompt: PromptRequest,
    pub command_id: CommandId,
    pub accepted_at: Timestamp,
}

#[harmos::transaction(id = "dokime.prompts.response.submit", irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmitPromptResponse {
    pub prompt_id: PromptId,
    pub action: String,
    pub responder: String,
    pub values: BTreeMap<String, Value>,
    pub accepted: bool,
    pub command_id: CommandId,
    pub accepted_at: Timestamp,
}

#[derive(Debug, Default)]
pub(crate) struct SettledPrompts {
    ids: Vec<PromptId>,
    at: Option<Timestamp>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PromptSettlement {
    Stopped,
    Ended,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */

impl PromptSettlement {
    fn action(self) -> &'static str {
        match self {
            Self::Stopped => "cancel",
            Self::Ended | Self::Interrupted => "expire",
        }
    }

    fn record_type(self) -> BuiltinRecordType {
        match self {
            Self::Stopped => BuiltinRecordType::PromptResponseCancelled,
            Self::Ended | Self::Interrupted => BuiltinRecordType::PromptResponseExpired,
        }
    }

    fn severity(self) -> RecordSeverity {
        match self {
            Self::Stopped => RecordSeverity::Info,
            Self::Ended | Self::Interrupted => RecordSeverity::Warning,
        }
    }

    fn reason(self) -> &'static str {
        match self {
            Self::Stopped => "execution_stopped",
            Self::Ended => "execution_ended",
            Self::Interrupted => "runtime_restarted",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::Stopped => "Prompt cancelled because its execution stopped.",
            Self::Ended => "Prompt expired because its execution ended.",
            Self::Interrupted => "Prompt expired because the runtime restarted.",
        }
    }
}

impl Transaction<RuntimeState> for RequestPrompt {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        prepare_request(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_request(state, self).expect("checked prompt plan").1.collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        request_prompt(state, self).expect("Harmos checked the prompt request before applying it");
    }
}

impl Transaction<RuntimeState> for SubmitPromptResponse {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        // A stopping runtime still settles the questions its stop cancels.
        if self.accepted {
            ensure_runtime_active(state)?;
        }
        prepare_response(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_response(state, self).expect("checked response plan").1.collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        submit_prompt_response(state, self).expect("Harmos checked the prompt response before applying it");
    }
}

/* === Public Functions === */

pub fn validate_request(state: &RuntimeState, request: &RequestPrompt) -> Result<PromptId, Error> {
    prepare_request(state, request).map(|(prompt, _)| prompt.prompt_id)
}

pub fn validate_response(state: &RuntimeState, response: &SubmitPromptResponse) -> Result<PromptId, Error> {
    prepare_response(state, response).map(|_| response.prompt_id.clone())
}

pub fn request_prompt(state: &mut RuntimeState, request: &RequestPrompt) -> Result<PromptId, Error> {
    let (prompt, records) = prepare_request(state, request)?;
    let id = prompt.prompt_id.clone();
    records.apply(state);
    state.prompt_requests.insert(id.clone(), prompt);
    Ok(id)
}

pub fn submit_prompt_response(state: &mut RuntimeState, response: &SubmitPromptResponse) -> Result<PromptId, Error> {
    let (answer, records) = prepare_response(state, response)?;
    records.apply(state);
    state.prompt_responses.insert(response.prompt_id.clone(), answer);
    Ok(response.prompt_id.clone())
}

pub(crate) fn settle_prompts(
    state: &RuntimeState, records: &mut Drafts, settlement: PromptSettlement, at: &Timestamp, owned: impl Fn(&RecordScope) -> bool,
) -> SettledPrompts {
    let contract = settlement.record_type().contract();
    let mut settled = SettledPrompts { ids: Vec::new(), at: Some(at.clone()) };
    for prompt in state.open_prompts().filter(|prompt| owned(&prompt.scope)) {
        let closure = PromptResponse {
            prompt_id: prompt.prompt_id.clone(),
            action: settlement.action().to_owned(),
            responder: "runtime".to_owned(),
            submitted_at: at.clone(),
            values: BTreeMap::new(),
            accepted: false,
        };
        let mut details = prompt_response_details(&closure);
        details.insert("source".to_owned(), Value::from("runtime"));
        details.insert("reason".to_owned(), Value::from(settlement.reason()));
        let mut record = Record::new(
            records.next_id(),
            at.clone(),
            settlement.record_type().identifier(),
            contract.schema_version,
            "runtime",
            settlement.severity(),
        );
        record.message = Some(settlement.message().to_owned());
        record.scope = Some(prompt.scope.clone());
        record.details = details;
        debug_assert!(
            validate_record(Some(&contract), &record, ValidationMode::Strict).is_ok(),
            "prompt settlement record matches its contract"
        );
        records.push(record);
        settled.ids.push(prompt.prompt_id.clone());
    }
    settled
}

pub(crate) fn commit_settled_prompts(state: &mut RuntimeState, settled: SettledPrompts) {
    let Some(at) = settled.at else {
        return;
    };
    for id in settled.ids {
        state.closed_prompts.insert(id, at.clone());
    }
}

/* === Private Functions === */

fn prepare_request(
    state: &RuntimeState, request: &RequestPrompt,
) -> Result<(PromptInstance, crate::transactions::records::draft::Drafts), Error> {
    let invocation = &request.prompt;
    let definition = state
        .prompt_definitions
        .get(invocation.prompt_type.as_str())
        .cloned()
        .ok_or_else(|| Error::not_found("prompt_definition_not_found", "question is not installed"))?;
    validate_prompt_definition(&definition)?;
    let scope = admitted_scope(state, invocation, &definition)?;
    let mut fields = definition.fields.clone();
    for field in &mut fields {
        // Opening an empty control is allowed. Provided values must be valid.
        field.required = false;
    }
    let issues = dokime_domain::model::prompts::validate_fields(&fields, &invocation.values);
    if !issues.is_empty() {
        return Err(Error::validation("validation_failed", "invalid question initial values", issues));
    }
    let prompt = PromptInstance {
        prompt_id: invocation.prompt_id.clone(),
        contract_id: dokime_domain::model::identity::ContractId::from_seed(invocation.prompt_id.as_str()),
        kind: definition.kind(),
        definition,
        allow_timeout: false,
        expires_at: None,
        values: invocation.values.clone(),
        scope,
        created_at: request.accepted_at.clone(),
    };
    let records = prepare_insert(state, &prompt, &request.accepted_at)?;
    Ok((prompt, records))
}

fn admitted_scope(state: &RuntimeState, request: &PromptRequest, definition: &PromptDefinition) -> Result<RecordScope, Error> {
    let invalid = || Error::validation("prompt_scope_invalid", "question scope does not match an admitted execution", vec![]);
    let scope = &request.scope;
    let standalone = state
        .installed_plugins
        .values()
        .any(|installation| installation.enabled && installation.prompts.iter().any(|declared| declared == definition));
    let Some(cycle_id) = &scope.test_cycle_id else {
        if scope.test_execution_id.is_some() || scope.test_case_id.is_some() || scope.template_id.is_some() {
            return Err(invalid());
        }
        // A trusted host may open a standalone installed question.
        if !standalone {
            return Err(invalid());
        }
        return Ok(scope.clone());
    };
    state.cycles.get(cycle_id).ok_or_else(invalid)?;
    let Some(execution_id) = &scope.test_execution_id else {
        if scope.test_case_id.is_some() || scope.template_id.is_some() {
            return Err(invalid());
        }
        // Host/Python cycle-level questions use the installed top-level catalog.
        if !standalone {
            return Err(invalid());
        }
        return Ok(scope.clone());
    };
    let execution = state.executions.get(execution_id).filter(|execution| &execution.test_cycle_id == cycle_id).ok_or_else(invalid)?;
    let case = execution.snapshot.case();
    if scope.test_case_id.as_ref().is_some_and(|id| id != &case.test_case_id)
        || scope.template_id.as_ref().is_some_and(|id| id != &case.test_case_template_id)
        || scope.session_id.as_ref().is_some_and(|id| id != &state.session_id)
    {
        return Err(invalid());
    }
    if scope.suite_id.as_ref().is_some_and(|id| Some(id) != execution.scope.suite_id.as_ref()) {
        return Err(invalid());
    }
    let suite = state.suites.suite_for_case(case).ok_or_else(invalid)?;
    let template = suite
        .suite
        .test_case_templates
        .iter()
        .find(|template| template.test_case_template_id == case.test_case_template_id)
        .ok_or_else(invalid)?;
    if !template.prompt_definitions.iter().any(|declared| declared == definition) {
        return Err(invalid());
    }
    let mut admitted = execution.scope.clone();
    admitted.step_id.clone_from(&scope.step_id);
    admitted.criterion_id.clone_from(&scope.criterion_id);
    Ok(admitted)
}

fn prepare_insert(
    state: &RuntimeState, prompt: &PromptInstance, at: &Timestamp,
) -> Result<crate::transactions::records::draft::Drafts, Error> {
    if state.prompt_requests.contains_key(&prompt.prompt_id) {
        return Err(Error::conflict("prompt_already_exists", "prompt already exists"));
    }
    // The request record carries the prompt's declared level, so the timeline
    // row and the sealed report state how urgent the question was.
    let records = prepare_prompt_record(
        state,
        request_record_type(&prompt.kind),
        prompt.definition.level,
        Some(prompt.definition.title.clone()),
        Some(prompt.scope.clone()),
        prompt_request_details(prompt),
        at.clone(),
    )?;
    Ok(records)
}

fn prepare_response(
    state: &RuntimeState, response: &SubmitPromptResponse,
) -> Result<(PromptResponse, crate::transactions::records::draft::Drafts), Error> {
    if state.prompt_responses.contains_key(&response.prompt_id) {
        return Err(Error::conflict("prompt_already_answered", "prompt already has a response"));
    }
    let prompt =
        state.prompt_requests.get(&response.prompt_id).cloned().ok_or_else(|| Error::not_found("prompt_not_found", "prompt not found"))?;
    if !state.prompt_open(&prompt) {
        return Err(Error::conflict("prompt_settled", "prompt was settled because its execution is no longer running"));
    }
    let action = response.action.trim().to_owned();
    prompt.definition.actions.iter().find(|candidate| candidate.id == action).ok_or_else(|| unsupported_action(&prompt, &action))?;
    if response.accepted {
        let issues = validate_prompt_response_values(&prompt, &response.values);
        if !issues.is_empty() {
            return Err(Error::validation("validation_failed", "prompt response validation failed", issues));
        }
    }
    let answer = PromptResponse {
        prompt_id: response.prompt_id.clone(),
        action: action.clone(),
        responder: response.responder.clone(),
        submitted_at: response.accepted_at.clone(),
        values: response.values.clone(),
        accepted: response.accepted,
    };
    let record_type =
        if response.accepted { BuiltinRecordType::PromptResponseSubmitted } else { BuiltinRecordType::PromptResponseCancelled };
    let mut details = prompt_response_details(&answer);
    if !response.accepted {
        details.insert("reason".to_owned(), Value::from(action));
    }
    let records = prepare_prompt_record(
        state,
        record_type,
        RecordSeverity::Info,
        Some("Prompt response submitted.".to_owned()),
        Some(prompt.scope),
        details,
        response.accepted_at.clone(),
    )?;
    Ok((answer, records))
}

fn prepare_prompt_record(
    state: &RuntimeState, record_type: BuiltinRecordType, severity: RecordSeverity, message: Option<String>, scope: Option<RecordScope>,
    details: BTreeMap<String, Value>, at: Timestamp,
) -> Result<crate::transactions::records::draft::Drafts, Error> {
    let contract = record_type.contract();
    let mut record = Record::new(state.next_record_id(), at, record_type.identifier(), contract.schema_version, "runtime", severity);
    record.message = message;
    record.scope = scope;
    record.details = details;
    validate_record(Some(&contract), &record, ValidationMode::Strict).map_err(|error| {
        Error::validation(
            "validation_failed",
            "prompt record validation failed",
            vec![ValidationIssue {
                path: format!("records.{}", record.record_type.as_str()),
                message: error.to_string(),
                expected: Some(format!("schema_version {}", contract.schema_version)),
            }],
        )
    })?;
    let mut records = crate::transactions::records::draft::Drafts::new(state);
    records.push(record);
    Ok(records)
}

fn validate_prompt_definition(definition: &PromptDefinition) -> Result<(), Error> {
    let issues = dokime_domain::model::prompts::validate_definition(definition);
    if issues.is_empty() { Ok(()) } else { Err(Error::validation("validation_failed", "prompt definition validation failed", issues)) }
}

fn validate_prompt_response_values(prompt: &PromptInstance, values: &BTreeMap<String, Value>) -> Vec<ValidationIssue> {
    dokime_domain::model::prompts::validate_answers(&prompt.definition, values)
}

fn unsupported_action(prompt: &PromptInstance, action: &str) -> Error {
    Error::validation(
        "validation_failed",
        format!("unsupported prompt action {action:?}"),
        vec![ValidationIssue {
            path: "action".to_owned(),
            message: "prompt action is not declared by the prompt".to_owned(),
            expected: Some(prompt.definition.actions.iter().map(|action| action.id.as_str()).collect::<Vec<_>>().join(", ")),
        }],
    )
}

fn request_record_type(kind: &PromptKind) -> BuiltinRecordType {
    match kind {
        PromptKind::Confirmation | PromptKind::Verification => BuiltinRecordType::PromptRequestConfirmation,
        PromptKind::Form => BuiltinRecordType::PromptRequestForm,
        PromptKind::Choice => BuiltinRecordType::PromptRequestChoice,
        PromptKind::Status => BuiltinRecordType::PromptRequestStatus,
    }
}

fn prompt_request_details(prompt: &PromptInstance) -> BTreeMap<String, Value> {
    let mut details = BTreeMap::from([
        ("prompt_id".to_owned(), Value::from(prompt.prompt_id.as_str())),
        ("contract_id".to_owned(), Value::from(prompt.contract_id.as_str())),
        ("contract_identifier".to_owned(), Value::from(prompt.definition.prompt_type.as_str())),
        ("prompt_type".to_owned(), Value::from(prompt.definition.prompt_type.as_str())),
        ("title".to_owned(), Value::from(prompt.definition.title.clone())),
        ("kind".to_owned(), Value::from(prompt_kind_wire(&prompt.kind))),
        (
            "fields".to_owned(),
            Value::Array(
                prompt
                    .definition
                    .fields
                    .iter()
                    .map(|field| {
                        let mut displayed = field.clone();
                        if let Some(value) = prompt.values.get(field.metadata.identifier.as_str()) {
                            displayed.default = Some(value.clone());
                        }
                        field_value(&displayed)
                    })
                    .collect(),
            ),
        ),
        ("actions".to_owned(), Value::Array(prompt.definition.actions.iter().map(action_value).collect())),
        ("allow_timeout".to_owned(), Value::from(prompt.allow_timeout)),
        ("created_at".to_owned(), Value::from(prompt.created_at.as_str())),
        ("context".to_owned(), Value::Object(BTreeMap::new())),
    ]);
    if let Some(message) = &prompt.definition.message {
        details.insert("message".to_owned(), Value::from(message.clone()));
    }
    if let Some(template) = &prompt.definition.template_identifier {
        details.insert("template_identifier".to_owned(), Value::from(template.as_str()));
    }
    if let Some(expires_at) = &prompt.expires_at {
        details.insert("expires_at".to_owned(), Value::from(expires_at.as_str()));
    }
    details
}

fn prompt_response_details(response: &PromptResponse) -> BTreeMap<String, Value> {
    BTreeMap::from([
        ("prompt_id".to_owned(), Value::from(response.prompt_id.as_str())),
        ("action".to_owned(), Value::from(response.action.clone())),
        ("responder".to_owned(), Value::from(response.responder.clone())),
        ("source".to_owned(), Value::from("api")),
        ("accepted".to_owned(), Value::from(response.accepted)),
        ("values".to_owned(), Value::Object(response.values.clone())),
    ])
}

fn field_value(field: &PromptField) -> Value {
    let mut object = BTreeMap::from([
        ("name".to_owned(), Value::from(field.metadata.identifier.as_str())),
        ("label".to_owned(), Value::from(field.metadata.label.clone())),
        ("value_type".to_owned(), Value::from(value_type_wire(&field.value_type))),
        (
            "kind".to_owned(),
            Value::from(match field.kind {
                PromptFieldKind::Text => "text",
                PromptFieldKind::Float => "float",
                PromptFieldKind::Int => "int",
                PromptFieldKind::Choice => "choice",
                PromptFieldKind::Boolean => "boolean",
            }),
        ),
        ("required".to_owned(), Value::from(field.required)),
        ("multiple".to_owned(), Value::from(field.multiple)),
        (
            "validation".to_owned(),
            Value::Object(BTreeMap::from([
                ("min_length".into(), field.validation.min_length.map(|value| Value::from(value as u64)).unwrap_or(Value::Null)),
                ("max_length".into(), field.validation.max_length.map(|value| Value::from(value as u64)).unwrap_or(Value::Null)),
                ("pattern".into(), field.validation.pattern.clone().map(Value::from).unwrap_or(Value::Null)),
                ("minimum".into(), field.validation.minimum.clone().unwrap_or(Value::Null)),
                ("maximum".into(), field.validation.maximum.clone().unwrap_or(Value::Null)),
                ("min_items".into(), field.validation.min_items.map(|value| Value::from(value as u64)).unwrap_or(Value::Null)),
                ("max_items".into(), field.validation.max_items.map(|value| Value::from(value as u64)).unwrap_or(Value::Null)),
            ])),
        ),
    ]);
    if let Some(description) = &field.metadata.description {
        object.insert("description".to_owned(), Value::from(description.clone()));
    }
    if let Some(default) = &field.default {
        object.insert("default".to_owned(), default.clone());
    }
    if !field.choices.is_empty() {
        object.insert("choices".to_owned(), Value::Array(field.choices.iter().cloned().map(Value::from).collect()));
    }
    Value::Object(object)
}

fn action_value(action: &PromptAction) -> Value {
    Value::Object(BTreeMap::from([
        ("id".to_owned(), Value::from(action.id.clone())),
        ("label".to_owned(), Value::from(action.label.clone())),
        (
            "emphasis".to_owned(),
            Value::from(match action.emphasis {
                PromptActionEmphasis::Primary => "primary",
                PromptActionEmphasis::Secondary => "secondary",
                PromptActionEmphasis::Danger => "danger",
            }),
        ),
        ("requires_condition".to_owned(), Value::from(action.requires_condition)),
    ]))
}

fn prompt_kind_wire(kind: &PromptKind) -> &'static str {
    match kind {
        PromptKind::Confirmation => "confirmation",
        PromptKind::Form => "form",
        PromptKind::Choice => "choice",
        PromptKind::Status => "status",
        PromptKind::Verification => "verification",
    }
}

fn value_type_wire(value_type: &ValueType) -> &'static str {
    match value_type {
        ValueType::Boolean => "boolean",
        ValueType::Integer => "integer",
        ValueType::Number => "number",
        ValueType::Text => "text",
        ValueType::Datetime => "datetime",
        ValueType::Array => "array",
        ValueType::Object => "object",
        ValueType::File => "file",
    }
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}

/* === Tests === */
