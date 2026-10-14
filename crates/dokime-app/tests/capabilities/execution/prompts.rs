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

/// A step that asks the operator and waits for the answer.
struct AskingChecks;

struct Hold;

#[dokime::prompt(id = "fixture.asking.hold", label = "Hold the charger")]
struct HoldQuestion;

#[dokime::catalog]
enum AskingTemplates {
    Hold(Hold),
}

#[dokime::catalog]
enum HoldPrompts {
    HoldQuestion(HoldQuestion),
}

#[dokime::suite(id = "fixture.asking.checks", label = "Asking", templates = AskingTemplates)]
impl AskingChecks {}

#[dokime::template(id = "fixture.asking.checks.hold", label = "Hold", prompts = HoldPrompts)]
impl Hold {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Ask")]
    async fn ask(&mut self, ctx: &mut dokime::execution::StepContext<'_>) -> dokime::Result<()> {
        ctx.ask(HoldQuestion).await?;
        Ok(())
    }
}

/// Serves the asking suite in process.
struct Asking;

#[async_trait::async_trait]
impl dokime_app::runtime::EmbeddedProvider for Asking {
    fn catalog(&self) -> dokime_app::InstalledPlugin {
        dokime_app::InstalledPlugin {
            source: "fixture.asking".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_app::definition::SuiteSet::new([AskingChecks::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn dokime::execution::Runnable>>> {
        Ok(vec![Arc::new(dokime::execution::Workflow::<Hold>::new(
            "fixture.asking.checks",
            "fixture.asking",
        )) as Arc<dyn dokime::execution::Runnable>])
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        Ok(())
    }
}

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
        let prompt_id = prompt_id.clone();
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

    // The stop settles the cycle's question without answering it, so the
    // waiting requester is cancelled rather than handed a response.
    let waited = waiter.await.unwrap().unwrap_err();
    assert_eq!(waited.code(), Some("prompt_wait_cancelled"));
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert!(runtime.prompt_settled_at(&prompt_id).await.is_some());
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

    // The restart settles the cycle's question as a runtime expiry, once.
    for _ in 0..2 {
        let (mut owner, runtime) = started(prompt_config(root.path())).await;
        assert_eq!(
            runtime.pending_prompt_projection().await,
            vec![standalone.clone()]
        );
        assert!(runtime.prompt_response(&id).await.is_none());
        assert!(runtime.prompt_settled_at(&id).await.is_some());
        assert_late_answer_refused(&runtime, &id).await;
        drop(runtime);
        owner.shutdown().await.unwrap();
    }
    let (mut owner, runtime) = started(prompt_config(root.path())).await;
    let settlements = settlement_records(&runtime, &id).await;
    assert_eq!(
        settlements,
        vec![(
            "prompt.response.expired".to_owned(),
            Value::from("runtime_restarted")
        )]
    );
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn cycle_stop_settles_prompts() {
    // === Ask An Operator Question From A Running Execution === //

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (_owner, runtime) = inspection_runtime(root.path(), executor.clone()).await;
    let suite = inspection_suite(plt400_suite());
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let id = ask_inspection(&runtime, &running, 0).await;
    assert_eq!(cycle_pending(&runtime, &test_cycle_id).await, 1);

    // === Stop The Cycle === //

    runtime
        .stop_cycle(test_cycle_id.clone(), CommandContext::default())
        .await
        .unwrap();

    // === The Stop Itself Settled The Question === //

    assert!(
        runtime.prompt_settled_at(&id).await.is_some(),
        "the stop settles the open prompt in its own transaction"
    );
    assert!(runtime.prompt_response(&id).await.is_none());
    assert_eq!(cycle_pending(&runtime, &test_cycle_id).await, 0);
    assert_late_answer_refused(&runtime, &id).await;
    executor.release();
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopped).await;
    assert_eq!(
        settlement_records(&runtime, &id).await,
        vec![(
            "prompt.response.cancelled".to_owned(),
            Value::from("execution_stopped")
        )]
    );
}

pub(crate) async fn execution_stop_settles_prompts() {
    // === Ask From The First Of Two Running Executions === //

    let root = tempdir().unwrap();
    let executor = Arc::new(MemberStopExecutor::default());
    let (_owner, runtime) = inspection_runtime(root.path(), executor.clone()).await;
    let mut suite = inspection_suite(plt400_suite());
    suite.test_case_templates[0].parameters[0].expansion =
        dokime::definition::ParameterExpansion::Cartesian;
    suite.test_case_templates[0].parameters[0].expression = ParameterExpr::Array {
        values: vec![
            ParameterExpr::Literal {
                value: Value::from("passed"),
            },
            ParameterExpr::Literal {
                value: Value::from("alternate"),
            },
        ],
    };
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let index = running
        .test_executions
        .iter()
        .position(|execution| execution.status == ExecutionStatus::Running)
        .expect("first execution is running");
    let id = ask_inspection(&runtime, &running, index).await;

    // === Stop That Execution Alone === //

    runtime
        .stop_execution(
            running.test_executions[index].test_execution_id.clone(),
            CommandContext::default(),
        )
        .await
        .unwrap();

    // === Its Question Settles With The Stop === //

    assert!(
        runtime.prompt_settled_at(&id).await.is_some(),
        "the execution stop settles its open prompt"
    );
    assert_eq!(cycle_pending(&runtime, &test_cycle_id).await, 0);
    assert_late_answer_refused(&runtime, &id).await;
    wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Stopped).await;
}

pub(crate) async fn interruption_settles_prompts() {
    // === Leave A Question Open When The Host Goes Down === //

    let root = tempdir().unwrap();
    let executor = Arc::new(BlockingPublishingExecutor::new());
    let (mut owner, runtime) = inspection_runtime(root.path(), executor.clone()).await;
    let suite = inspection_suite(plt400_suite());
    register_python_suite(&runtime, suite.clone()).await;
    let test_cycle_id = start_cycle(&runtime, suite, false).await;
    runtime
        .run_cycle_cases(test_cycle_id.clone(), CommandContext::default())
        .await;
    executor.wait_until_entered().await;
    let running = wait_for_cycle_lifecycle(&runtime, &test_cycle_id, CycleStatus::Running).await;
    let id = ask_inspection(&runtime, &running, 0).await;
    drop(runtime);
    owner.shutdown().await.unwrap();

    // === After Restart The Question Is Settled, Not Waiting === //

    let (mut owner, runtime) = inspection_runtime(root.path(), executor.clone()).await;
    assert!(
        runtime.prompt_settled_at(&id).await.is_some(),
        "an interrupted execution leaves no open prompt"
    );
    assert_eq!(cycle_pending(&runtime, &test_cycle_id).await, 0);
    assert!(runtime.pending_prompt_projection().await.is_empty());
    assert_late_answer_refused(&runtime, &id).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/// A runtime with the inspection question installed and `executor` running
/// every cycle.
async fn inspection_runtime(
    root: &std::path::Path,
    executor: Arc<dyn ExecutionFixture>,
) -> (dokime_app::Runtime, RuntimeHandle) {
    let config = config(root)
        .with_declared_plugin(
            "prompt.fixture",
            "1.0.0",
            "Questions",
            prompts([inspection_definition()]),
        )
        .unwrap();
    started_with_fixture(config, executor).await
}

/// Declares the operator question every execution of the suite may ask. It
/// offers no cancel action, so only the runtime can settle it.
fn inspection_suite(mut suite: TestSuite) -> TestSuite {
    suite.test_case_templates[0]
        .prompt_definitions
        .push(inspection_definition());
    suite
}

fn inspection_definition() -> dokime::prompts::PromptDefinition {
    use dokime::prompts::{FormPrompt, Prompt};
    let mut definition =
        FormPrompt::<BTreeMap<String, bool>>::new("test.prompt.inspect", "Inspect Charging State")
            .field(PromptField {
                metadata: Metadata::new(Identifier::new("charging").unwrap(), "Charging", None),
                value_type: ValueType::Boolean,
                kind: PromptFieldKind::Boolean,
                multiple: false,
                required: true,
                default: None,
                choices: Vec::new(),
                validation: Default::default(),
            })
            .definition();
    definition.actions.retain(|action| action.id != "cancel");
    definition
}

async fn ask_inspection(
    runtime: &RuntimeHandle,
    cycle: &dokime_app::execution::CycleView,
    index: usize,
) -> PromptId {
    let execution = &cycle.test_executions[index];
    let request = PromptRequest {
        prompt_id: PromptId::new(),
        prompt_type: Identifier::new("test.prompt.inspect").unwrap(),
        values: BTreeMap::new(),
        scope: RecordScope::default()
            .with_cycle(cycle.test_cycle_id.clone())
            .with_test_execution(execution.test_execution_id.clone()),
    };
    let id = request.prompt_id.clone();
    runtime.request_prompt(request).await.unwrap();
    id
}

async fn cycle_pending(runtime: &RuntimeHandle, cycle: &TestCycleId) -> usize {
    runtime
        .pending_prompt_projection()
        .await
        .iter()
        .filter(|prompt| prompt.scope.test_cycle_id.as_ref() == Some(cycle))
        .count()
}

async fn assert_late_answer_refused(runtime: &RuntimeHandle, id: &PromptId) {
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
    assert!(
        matches!(error, dokime_app::Error::Conflict { .. }),
        "a late answer is a failed precondition, got {error:?}"
    );
}

async fn settlement_records(runtime: &RuntimeHandle, id: &PromptId) -> Vec<(String, Value)> {
    runtime
        .prompt_response_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|record| {
            record.details.get("prompt_id") == Some(&Value::from(id.as_str()))
                && record.details.get("responder") == Some(&Value::from("runtime"))
        })
        .map(|record| {
            let reason = record.details.get("reason").cloned().unwrap_or(Value::Null);
            (record.record_type.as_str().to_owned(), reason)
        })
        .collect()
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

pub(crate) async fn stopped_run_question() {
    for stop_cycle in [false, true] {
        check_stopped_question(stop_cycle).await;
    }
}

async fn check_stopped_question(stop_cycle: bool) {
    use dokime_app::execution::ExecutionStatus;

    // === Run A Step That Waits On An Operator === //

    let root = tempdir().unwrap();
    let config = RuntimeConfig::new(
        root.path(),
        Metadata::new(
            Identifier::new("fixture.asking.host").unwrap(),
            "Asking Host",
            None,
        ),
    )
    .register(Arc::new(Asking));
    let mut owner = dokime_app::Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let suite = AskingChecks::definition().suite;
    let cycle = runtime
        .start_cycle(
            CycleDefinition {
                cases: crate::support::suites::materialize_cases("fixture.asking.checks", &suite),
                suite,
                metadata: Metadata::new(
                    Identifier::new("fixture.asking.cycle").unwrap(),
                    "Asking Cycle",
                    None,
                ),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = runtime
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    let id = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if let Some(prompt) = runtime.pending_prompt_projection().await.into_iter().next() {
                break prompt.prompt_id;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the step asks its question");

    // === Stop While The Question Is Open === //

    if stop_cycle {
        runtime
            .stop_cycle(cycle.clone(), CommandContext::default())
            .await
            .unwrap();
    } else {
        let execution = runtime
            .cycles()
            .await
            .into_iter()
            .find(|item| item.test_cycle_id == cycle)
            .unwrap()
            .test_executions[0]
            .test_execution_id
            .clone();
        runtime
            .stop_execution(execution, CommandContext::default())
            .await
            .unwrap();
    }

    // === The Run Ends Stopped, Not Failed === //

    let ended = wait_for_cycle_lifecycle(&runtime, &cycle, CycleStatus::Stopped).await;
    assert_eq!(
        ended.test_executions[0].status,
        ExecutionStatus::Cancelled,
        "a question the stop settled is a cancellation, not a step failure"
    );
    assert_eq!(
        settlement_records(&runtime, &id).await,
        vec![(
            "prompt.response.cancelled".to_owned(),
            Value::from("execution_stopped")
        )],
        "the stop settles the question once; winding down adds nothing"
    );
    assert_late_answer_refused(&runtime, &id).await;
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
