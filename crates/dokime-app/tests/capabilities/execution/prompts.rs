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
use tempfile::tempdir;
// Local Imports
use super::*;
use dokime::core::{Identifier, Metadata, PromptId, RecordScope, Value, ValueType};
use dokime::definition::TestSuite;
use dokime::prompts::{PromptField, PromptFieldKind, PromptRequest};
use dokime::records::{ProductConnectionDetails, ProductConnectionState, RecordCondition};
use dokime_app::RuntimeConfig;
use dokime_app::definition::RuntimeCondition;
use dokime_app::runtime::command::{CommandContext, CommandStatus};
use dokime_app::{execution::CycleStatus, transactions::CycleDefinition};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn prompt_commands_records() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(prompt_config(root.path())).await;
    let initial_revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let prompt = prompt_request();
    let prompt_id = prompt.prompt_id.clone();

    // The prompt a request names is the one its installation declared, and the
    // only one.
    let definitions = runtime.prompt_definitions().await;
    assert_eq!(definitions.len(), 1);
    assert!(
        definitions
            .iter()
            .any(|definition| definition.prompt_type == prompt_definition().prompt_type)
    );

    runtime.request_prompt(prompt).await.unwrap();
    let after_request_revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let pending = runtime.pending_prompt_projection().await;
    let request_records = runtime.prompt_request_records().await.unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].prompt_id, prompt_id);
    assert_eq!(
        request_records[0].record_type.as_str(),
        "prompt.request.form"
    );
    assert!(after_request_revision > initial_revision);

    let response = runtime
        .submit_prompt_response(
            prompt_id.clone(),
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
        )
        .await
        .unwrap();
    let evidence = runtime.evidence_snapshot().await.unwrap();

    assert!(response.accepted);
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert_eq!(
        runtime.prompt_response_records().await.unwrap()[0]
            .record_type
            .as_str(),
        "prompt.response.submitted"
    );
    assert_eq!(evidence.prompts.len(), 0);
    assert!(
        runtime
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .revision
            > after_request_revision
    );
    assert!(
        evidence
            .records
            .iter()
            .any(|record| record.record_type.as_str() == "prompt.response.submitted")
    );

    // The same flow through the command variant answers with a runtime receipt:
    // the revision it committed at, and how long it took.
    let second = prompt_request();
    let second_id = second.prompt_id.clone();
    runtime.request_prompt(second).await.unwrap();
    let before_response_revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let receipt = runtime
        .submit_prompt_response_command(
            second_id,
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
            CommandContext::default(),
        )
        .await
        .unwrap();

    assert_eq!(receipt.status, CommandStatus::Accepted);
    assert!(receipt.duration_ms.is_some());
    assert!(receipt.runtime_revision > before_response_revision);
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert_eq!(runtime.prompt_response_records().await.unwrap().len(), 2);
}

pub(crate) async fn duplicate_invalid_refused() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(prompt_config(root.path())).await;
    let prompt = prompt_request();
    let prompt_id = prompt.prompt_id.clone();

    runtime.request_prompt(prompt).await.unwrap();
    let invalid = runtime
        .submit_prompt_response(
            prompt_id.clone(),
            "missing".to_owned(),
            "operator".to_owned(),
            BTreeMap::new(),
            true,
        )
        .await
        .unwrap_err();

    assert_eq!(invalid.code(), Some("validation_failed"));

    runtime
        .submit_prompt_response(
            prompt_id.clone(),
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
        )
        .await
        .unwrap();
    let duplicate = runtime
        .submit_prompt_response(
            prompt_id,
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
        )
        .await
        .unwrap_err();

    assert_eq!(duplicate.code(), Some("prompt_already_answered"));
}

// condition gated prompt actions require runtime condition match.

pub(crate) async fn condition_gated_runtime() {
    let root = tempdir().unwrap();
    let prompt = prompt_request();
    let prompt_id = prompt.prompt_id.clone();
    let mut definition = prompt_definition();
    definition.actions[0].requires_condition = true;
    definition.condition = Some(RuntimeCondition::Record {
        condition: RecordCondition {
            record_type: Identifier::new("tool.connection.acquired").unwrap(),
            match_fields: BTreeMap::from([
                ("resource_id".to_owned(), Value::from("plt400")),
                ("connection_state".to_owned(), Value::from("connected")),
            ]),
        },
    });

    let (_owner, runtime) = started(question_config(root.path(), definition.clone())).await;
    runtime.request_prompt(prompt).await.unwrap();
    assert_eq!(
        runtime.pending_prompt_projection().await[0].definition,
        definition
    );
    let blocked = runtime
        .submit_prompt_response(
            prompt_id.clone(),
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
        )
        .await
        .unwrap_err();

    assert_eq!(blocked.code(), Some("prompt_condition_not_satisfied"));

    runtime
        .publish_product_record(
            ProductConnectionDetails::new("plt400", ProductConnectionState::Connected)
                .into_draft("test"),
        )
        .await
        .unwrap();
    let response = runtime
        .submit_prompt_response(
            prompt_id,
            "submit".to_owned(),
            "operator".to_owned(),
            BTreeMap::from([("confirmed".to_owned(), Value::from(true))]),
            true,
        )
        .await
        .unwrap();

    assert!(response.accepted);
}

pub(crate) async fn stopping_cycle_waits() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(prompt_config(root.path())).await;
    let cycle_id = runtime
        .start_cycle(
            CycleDefinition {
                suite: prompt_test_suite(),
                metadata: Metadata::new(
                    Identifier::new("cycle.prompt.stop").unwrap(),
                    "Prompt Stop Cycle",
                    None,
                ),
                cases: Vec::new(),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let mut prompt = prompt_request();
    prompt.scope = RecordScope::default().with_cycle(cycle_id.clone());

    let prompt_id = prompt.prompt_id.clone();
    runtime.request_prompt(prompt).await.unwrap();

    let waiter = tokio::spawn({
        let runtime = runtime.clone();
        let cycle_id = cycle_id.clone();
        async move {
            runtime
                .wait_for_prompt_response_for_cycle(prompt_id.clone(), cycle_id)
                .await
        }
    });

    let stop = runtime
        .stop_cycle(cycle_id.clone(), CommandContext::default())
        .await;
    assert!(stop.is_ok());

    let response = waiter.await.unwrap().unwrap();
    assert!(!response.accepted);
    assert_eq!(response.action, "cancel");
    assert_eq!(response.responder, "runtime");
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert_eq!(
        runtime.cycle_lifecycle(&cycle_id).await,
        Some(CycleStatus::Stopped)
    );
}

pub(crate) async fn restart_closes_prompts() {
    let root = tempdir().unwrap();
    let (mut owner, runtime) = started(prompt_config(root.path())).await;
    let cycle = runtime
        .start_cycle(
            CycleDefinition {
                suite: prompt_test_suite(),
                metadata: Metadata::new(
                    Identifier::new("cycle.prompt.recovery").unwrap(),
                    "Prompt Recovery",
                    None,
                ),
                cases: Vec::new(),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let mut scoped = prompt_request();
    scoped.scope = RecordScope::default().with_cycle(cycle);
    let id = scoped.prompt_id.clone();
    runtime.request_prompt(scoped).await.unwrap();
    let standalone = runtime.request_prompt(prompt_request()).await.unwrap();
    assert_eq!(runtime.pending_prompt_projection().await.len(), 2);
    drop(runtime);
    owner.shutdown().await.unwrap();

    for _ in 0..2 {
        let (mut owner, runtime) = started(prompt_config(root.path())).await;
        assert_eq!(
            runtime.pending_prompt_projection().await,
            vec![standalone.clone()]
        );
        assert!(runtime.prompt_response(&id).await.is_none());
        let error = runtime
            .submit_prompt_response(
                id.clone(),
                "submit".into(),
                "operator".into(),
                BTreeMap::from([("confirmed".into(), Value::Bool(true))]),
                true,
            )
            .await
            .unwrap_err();
        assert_eq!(error.code(), Some("prompt_not_found"));
        assert!(runtime.prompt_response_records().await.unwrap().is_empty());
        drop(runtime);
        owner.shutdown().await.unwrap();
    }
}

fn prompt_definition() -> dokime::prompts::PromptDefinition {
    use dokime::prompts::{FormPrompt, Prompt};
    FormPrompt::<BTreeMap<String, bool>>::new("test.prompt.confirm", "Confirm Test Prompt")
        .field(PromptField {
            metadata: Metadata::new(Identifier::new("confirmed").unwrap(), "Confirmed", None),
            value_type: ValueType::Boolean,
            kind: PromptFieldKind::Boolean,
            multiple: false,
            required: true,
            default: Some(Value::Bool(true)),
            choices: Vec::new(),
            validation: Default::default(),
        })
        .definition()
}

fn prompt_request() -> PromptRequest {
    PromptRequest {
        prompt_id: PromptId::new(),
        prompt_type: Identifier::new("test.prompt.confirm").unwrap(),
        values: BTreeMap::new(),
        scope: RecordScope::default(),
    }
}

fn question_config(
    root: &std::path::Path,
    definition: dokime::prompts::PromptDefinition,
) -> RuntimeConfig {
    let domains = prompts([definition]);
    config(root)
        .with_installed_plugins([])
        .with_declared_plugin("prompt.fixture", "1.0.0", "Questions", domains)
        .unwrap()
}

fn prompt_config(root: impl Into<std::path::PathBuf>) -> RuntimeConfig {
    config(root)
        .with_declared_plugin(
            "prompt.fixture",
            "1.0.0",
            "Questions",
            prompts([prompt_definition()]),
        )
        .unwrap()
}

fn prompts(
    definitions: impl IntoIterator<Item = dokime::prompts::PromptDefinition>,
) -> harmos::sidecar::Registrar {
    use dokime::plugin::registration::{PROMPTS_NAMESPACE, domain_schema};
    let mut registrar = harmos::sidecar::Registrar::new();
    registrar
        .register(
            PROMPTS_NAMESPACE,
            domain_schema(PROMPTS_NAMESPACE),
            &definitions.into_iter().collect::<Vec<_>>(),
        )
        .unwrap();
    registrar
}

fn prompt_test_suite() -> TestSuite {
    TestSuite {
        metadata: Metadata::new(
            Identifier::new("suite.prompt.stop").unwrap(),
            "Prompt Stop Suite",
            None,
        ),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: Vec::new(),
    }
}

pub(crate) async fn requests_refuses_scopes() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = started(prompt_config(root.path())).await;
    let mut unknown = prompt_request();
    unknown.prompt_type = Identifier::new("unknown.question").unwrap();
    assert_eq!(
        runtime.request_prompt(unknown).await.unwrap_err().code(),
        Some("prompt_definition_not_found")
    );
    for values in [
        BTreeMap::from([("confirmed".to_owned(), Value::from("not a boolean"))]),
        BTreeMap::from([("unregistered".to_owned(), Value::from(true))]),
    ] {
        let mut prompt = prompt_request();
        prompt.values = values;
        assert!(runtime.request_prompt(prompt).await.is_err());
    }
    let mut prompt = prompt_request();
    prompt.scope.test_execution_id = Some(dokime::core::TestExecutionId::new());
    assert_eq!(
        runtime.request_prompt(prompt).await.unwrap_err().code(),
        Some("prompt_scope_invalid")
    );
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert!(runtime.prompt_request_records().await.unwrap().is_empty());
}

pub(crate) async fn answer_rules_commit() {
    use dokime::expressions::{BooleanExpression, Expression, ExpressionBinaryOp};
    use dokime::prompts::{FormPrompt, Prompt, TextPrompt};
    let root = tempdir().unwrap();
    let definition = FormPrompt::<BTreeMap<String, String>>::new("pair", "Confirm the entry")
        .field(TextPrompt::new("first", "First").field())
        .field(TextPrompt::new("second", "Second").field())
        .rule(
            BooleanExpression::new(Expression::binary(
                ExpressionBinaryOp::Eq,
                Expression::selector("first"),
                Expression::selector("second"),
            )),
            "Entries must match",
        )
        .definition();
    let mut invalid = definition.clone();
    invalid.rules[0].predicate = BooleanExpression::selector("undeclared").eq(true);
    let domains = prompts([invalid]);
    let invalid_root = tempdir().unwrap();
    let invalid_config = config(invalid_root.path())
        .with_installed_plugins([])
        .with_declared_plugin("invalid.rules", "1.0.0", "Invalid rules", domains)
        .unwrap();
    assert!(
        {
            let runtime_config = invalid_config;
            dokime_app::Runtime::start(runtime_config)
        }
        .await
        .is_err()
    );
    let (mut owner, runtime) = started(question_config(root.path(), definition.clone())).await;
    let id = PromptId::new();
    runtime
        .request_prompt(PromptRequest {
            prompt_id: id.clone(),
            prompt_type: definition.prompt_type.clone(),
            values: BTreeMap::new(),
            scope: RecordScope::default(),
        })
        .await
        .unwrap();
    assert_eq!(
        runtime.pending_prompt_projection().await[0].definition,
        definition
    );
    let revision = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let bad = BTreeMap::from([
        ("first".into(), Value::from("one")),
        ("second".into(), Value::from("two")),
    ]);
    let rejected = runtime
        .submit_prompt_response(id.clone(), "submit".into(), "operator".into(), bad, true)
        .await
        .unwrap_err();
    assert_eq!(rejected.code(), Some("validation_failed"));
    assert!(matches!(
        &rejected,
        dokime_app::Error::Validation { issues, .. }
            if issues.iter().any(|issue| issue.message == "Entries must match")
    ));
    assert_eq!(
        runtime
            .runtime_overview()
            .await
            .expect("runtime overview query succeeds")
            .revision,
        revision
    );
    assert_eq!(runtime.pending_prompt_projection().await.len(), 1);
    assert!(runtime.prompt_response_records().await.unwrap().is_empty());
    let good = BTreeMap::from([
        ("first".into(), Value::from("one")),
        ("second".into(), Value::from("one")),
    ]);
    runtime
        .submit_prompt_response(id, "submit".into(), "operator".into(), good.clone(), true)
        .await
        .unwrap();
    assert!(runtime.pending_prompt_projection().await.is_empty());
    let responses = runtime.prompt_response_records().await.unwrap();
    assert_eq!(responses.len(), 1);
    drop(runtime);
    owner.shutdown().await.unwrap();
    let (mut restarted, _) = started(prompt_config(root.path())).await;
    assert!(
        restarted
            .handle()
            .pending_prompt_projection()
            .await
            .is_empty()
    );
    assert_eq!(
        restarted.handle().prompt_response_records().await.unwrap(),
        responses,
        "accepted prompt evidence survives managed recovery"
    );
    restarted.shutdown().await.unwrap();
}

/* === Tests === */
