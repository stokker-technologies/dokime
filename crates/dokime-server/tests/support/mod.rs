/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![allow(dead_code)]

/* === Modules === */
pub mod artifacts;
#[path = "../../../dokime-runtime/tests/support/services/budgets.rs"]
pub mod budgets;
pub mod host;
pub mod wire;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Third Party Imports
use tempfile::tempdir;
use tonic::{Extensions, Request, Status};
// Local Imports
use dokime_domain::model;
use dokime_runtime::domains::{ActionDispatcher, ExecutionProviderTarget, TestSuiteDefinition};
use dokime_runtime::{
    CaseId, CredentialBackendKind, Identifier, Metadata, ParameterId, TemplateId, TestCycleId,
    Value, ValueType,
};
use dokime_runtime::definition::{
    CriterionDefinition, ExecutionPhaseDefinition, ParameterDefinition, ParameterExpr, TestCase,
    TestCaseTemplate, TestSuite,
};
use dokime_runtime::domains::execution::provider::{
    ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture,
};
use dokime_runtime::execution::ExecutionStatus;
use dokime_runtime::execution::{CycleStatus, CycleView};
use dokime_runtime::runtime::command::CommandContext;
use dokime_runtime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use dokime_runtime::transactions::CycleDefinition;
use dokime_runtime::transport::{
    ProviderCriterionOutcome, ProviderCycleOutcome, ProviderDependencyProbe,
    ProviderExecutionFailure, ProviderExecutionOutcome, ProviderPhaseOutcome,
    ProviderStatusOutcome, ProviderTelemetryOutcome,
};
use dokime_runtime::{InstalledPlugin, Runtime, RuntimeCredentialConfig, RuntimeHandle};
use dokime_client::contracts;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct ProgressRow {
    pub progress: f64,
}

pub struct TestEnvironment {
    pub root: tempfile::TempDir,
    _runtime_owner: Runtime,
    pub runtime: RuntimeHandle,
}

struct FakePythonExecutor;

struct FixtureProgressExecutor;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl TestEnvironment {
    pub async fn with_installed_plugins(artifacts: Vec<InstalledPlugin>) -> Self {
        Self::with_provider(Arc::new(FakePythonExecutor), artifacts, None).await
    }

    pub async fn with_installed_plugins_and_action_dispatcher(
        artifacts: Vec<InstalledPlugin>,
        suite_id: Identifier,
        dispatcher: Arc<dyn ActionDispatcher>,
    ) -> Self {
        Self::with_provider(
            Arc::new(FakePythonExecutor),
            artifacts,
            Some((suite_id, dispatcher)),
        )
        .await
    }

    async fn with_provider(
        provider: Arc<dyn ExecutionFixture>,
        artifacts: Vec<InstalledPlugin>,
        action_dispatcher: Option<(Identifier, Arc<dyn ActionDispatcher>)>,
    ) -> Self {
        let root = tempdir().unwrap();
        let mut runtime_config =
            budgets::runtime_config(root.path(), metadata("runtime.hub", "Runtime Hub"))
                .with_credential_backend(RuntimeCredentialConfig {
                    backend: CredentialBackendKind::Memory,
                    local_file: None,
                })
                .with_installed_plugins(artifacts);
        if let Some((suite_id, dispatcher)) = action_dispatcher {
            runtime_config = runtime_config.with_action_dispatcher(suite_id, dispatcher);
        }
        let runtime_owner =
            { dokime_runtime::Runtime::with_execution_fixture(runtime_config, "fixture", provider) }
                .await
                .unwrap();
        let runtime = runtime_owner.handle();

        Self {
            root,
            _runtime_owner: runtime_owner,
            runtime,
        }
    }
}

impl ExecutionFixture for FakePythonExecutor {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            if cycle.metadata.identifier.as_str() == "cycle.parameters" {
                let execution = cycle
                    .test_executions
                    .first()
                    .expect("parameter fixture execution exists");
                let target = context
                    .execution_file_target(
                        &execution.test_execution_id,
                        "artifacts/support/tooling/example.log".to_owned(),
                        true,
                        true,
                    )
                    .await
                    .expect("parameter fixture output target succeeds")
                    .expect("parameter fixture output target is available");
                std::fs::write(target.absolute_path, "line 1\n")
                    .expect("parameter fixture output is writable");
            }
            Ok(fixture_provider_outcome(cycle, false))
        })
    }
}

impl ExecutionFixture for FixtureProgressExecutor {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move { Ok(fixture_provider_outcome(cycle, true)) })
    }
}

/* === Public Functions === */
/// Creates an empty cycle, then appends the planned cases under the same
/// request metadata, and returns the cycle ID.
pub(crate) async fn planned_cycle(
    client: &wire::WireClient,
    create: Request<contracts::execution::CreateCycle>,
    plan: model::planning::wire::SelectionPlan,
) -> Result<String, Status> {
    let metadata = create.metadata().clone();
    let cycle_id = client
        .unary::<contracts::execution::CreateCycle>(create)
        .await?
        .into_inner()
        .cycle_id;
    let append = contracts::execution::AppendPlannedCases {
        cycle_id: cycle_id.clone(),
        plan: Some(plan),
        context: None,
    };
    client
        .unary::<contracts::execution::AppendPlannedCases>(Request::from_parts(
            metadata,
            Extensions::default(),
            append,
        ))
        .await?;
    Ok(cycle_id)
}

/// Creates an empty cycle on the runtime, then appends the materialized cases.
pub async fn cycle_with_cases(
    runtime: &RuntimeHandle,
    metadata: Metadata,
    cases: Vec<TestCase>,
) -> TestCycleId {
    let cycle_id = runtime
        .create_cycle(
            CycleDefinition {
                metadata,
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    runtime
        .append_cycle_cases(cycle_id.clone(), cases, CommandContext::default())
        .await
        .unwrap();
    cycle_id
}

pub fn materialize_cases(suite_id: &str, suite: &TestSuite) -> Vec<TestCase> {
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: dokime_runtime::definition::DEFAULT_MATERIALIZED_CASE_CAP,
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    suite
        .test_case_templates
        .iter()
        .flat_map(|template| {
            dokime_runtime::definition::materialize_template_cases(
                suite_id,
                template,
                Default::default(),
                options.clone(),
            )
            .unwrap()
            .test_cases
            .into_iter()
            .map(|mut case| {
                case.suite = Some(suite.metadata.clone());
                case
            })
        })
        .collect()
}

pub fn fixture_registration(suite: TestSuite) -> TestSuiteDefinition {
    let suite_id = suite.metadata.identifier.to_string();
    TestSuiteDefinition {
        suite_id,
        tags: vec!["fixture".to_owned()],
        workflow: Some("execute_cycle".to_owned()),
        suite,
        record_contracts: Vec::new(),
        telemetry_contracts: Vec::new(),
    }
}

pub fn fixture_progress_registration(suite: TestSuite) -> TestSuiteDefinition {
    let mut registration = fixture_registration(suite);
    registration.telemetry_contracts = vec![fixture_progress_contract()];

    registration
}

pub async fn register_fixture_progress_execution(runtime: &RuntimeHandle, suite: TestSuite) {
    register_fixture_registration(runtime, fixture_progress_registration(suite)).await;
}

pub fn fixture_suite(results: &[&str]) -> TestSuite {
    TestSuite {
        metadata: metadata("fixture.suite", "Fixture Suite"),
        tags: vec!["fixture".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: results
            .iter()
            .enumerate()
            .map(|(index, result)| case_template(index, result))
            .collect(),
    }
}

pub async fn register_fixture_execution(runtime: &RuntimeHandle, suite: TestSuite) {
    register_fixture_registration(runtime, fixture_registration(suite)).await;
}

pub fn fixture_diagnostics_suite(results: &[&str]) -> TestSuite {
    TestSuite {
        metadata: metadata("fixture.diagnostics", "Fixture Diagnostics"),
        tags: vec!["fixture".to_owned()],
        parameter_groups: Vec::new(),
        test_case_templates: results
            .iter()
            .enumerate()
            .map(|(index, result)| {
                named_case_template("fixture.diagnostic", "Fixture Diagnostic", index, result)
            })
            .collect(),
    }
}

pub async fn register_fixture_execution_pair(
    runtime: &RuntimeHandle,
    first: TestSuite,
    second: TestSuite,
) {
    let mut diagnostics = fixture_progress_registration(second);
    diagnostics.workflow = Some("run_diagnostics".to_owned());
    runtime
        .set_suites(
            "provider:fixture",
            dokime_runtime::definition::SuiteSet::new([
                fixture_progress_registration(first),
                diagnostics,
            ]),
        )
        .await
        .unwrap();
}

pub fn materialized_case(identifier: &str, label: &str, result: &str) -> TestCase {
    TestCase {
        suite_id: "fixture.registration".to_owned(),
        test_case_id: dokime_runtime::model::identity::new_id::<CaseId>(),
        test_case_template_id: dokime_runtime::model::identity::new_id::<TemplateId>(),
        suite: None,
        metadata: metadata(identifier, label),
        parameters: BTreeMap::from([("fixture.result".to_owned(), Value::from(result))]),
        generation_index: 0,
        label_number: 0,
    }
}

pub async fn current_cycle_ids(runtime: &RuntimeHandle) -> BTreeSet<TestCycleId> {
    runtime
        .cycles()
        .await
        .into_iter()
        .map(|cycle| cycle.test_cycle_id)
        .collect()
}

pub async fn newest_test_cycle_id(
    runtime: &RuntimeHandle,
    before: &BTreeSet<TestCycleId>,
) -> TestCycleId {
    runtime
        .cycles()
        .await
        .into_iter()
        .map(|cycle| cycle.test_cycle_id)
        .find(|test_cycle_id| !before.contains(test_cycle_id))
        .unwrap()
}

pub async fn test_case_id_for(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
    test_case_id: &CaseId,
) -> CaseId {
    runtime
        .cycles()
        .await
        .into_iter()
        .find(|cycle| &cycle.test_cycle_id == test_cycle_id)
        .unwrap()
        .cases()
        .find(|case| &case.test_case_id == test_case_id)
        .unwrap()
        .test_case_id
        .clone()
}

/// Awaits the state revision in which the cycle reaches `lifecycle`.
pub async fn wait_for_cycle_lifecycle(
    runtime: &RuntimeHandle,
    test_cycle_id: &TestCycleId,
    lifecycle: CycleStatus,
) -> CycleView {
    let snapshot = runtime
        .watch_state()
        .wait_until(|snapshot| {
            snapshot
                .state()
                .cycle_view(test_cycle_id)
                .is_none_or(|cycle| cycle.lifecycle == lifecycle)
        })
        .await
        .expect("the runtime state watch stays open");
    snapshot
        .state()
        .cycle_view(test_cycle_id)
        .expect("the cycle stays live until it reaches the lifecycle")
}

/// Re-runs `probe` after every runtime state change until it yields a value.
pub async fn on_state_change<T>(
    runtime: &RuntimeHandle,
    mut probe: impl AsyncFnMut() -> Option<T>,
) -> T {
    let mut changes = runtime.watch_state();
    loop {
        if let Some(value) = probe().await {
            return value;
        }
        changes
            .next_change()
            .await
            .expect("the runtime state watch stays open");
    }
}

pub fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(Identifier::new(identifier).unwrap(), label, None)
}

pub fn fixture_provider_outcome(cycle: &CycleView, emit_progress: bool) -> ProviderCycleOutcome {
    let mut failures = Vec::new();
    let mut executions = Vec::new();

    for (index, execution) in cycle
        .test_executions
        .iter()
        .filter(|execution| {
            matches!(
                execution.status,
                ExecutionStatus::Pending | ExecutionStatus::Running
            )
        })
        .enumerate()
    {
        let case = cycle
            .cases()
            .find(|case| case.test_case_id == execution.test_case_id)
            .expect("execution case exists");
        let status = case
            .parameters
            .get("fixture.result")
            .and_then(|value| match value {
                Value::Text(value) => Some(value.as_str()),
                _ => None,
            })
            .unwrap_or("passed");

        if status != "passed" {
            failures.push(ProviderExecutionFailure {
                stage: "template".to_owned(),
                template_identifier: Some(case.metadata.identifier.as_str().to_owned()),
                module: None,
                message: "fixture failure".to_owned(),
                detail: None,
            });
        }

        executions.push(ProviderExecutionOutcome {
            case_id: case.test_case_id.as_str().to_owned(),
            test_execution_id: execution.test_execution_id.as_str().to_owned(),
            template_identifier: case.metadata.identifier.as_str().to_owned(),
            label: case.metadata.label.clone(),
            module: "server_test.fixture".to_owned(),
            status: status.to_owned(),
            phases: vec![ProviderPhaseOutcome {
                identifier: "execution".to_owned(),
                label: "Execution".to_owned(),
                status: status.to_owned(),
            }],
            records: Vec::new(),
            telemetry: if emit_progress {
                vec![fixture_progress_telemetry(index as f64)]
            } else {
                Vec::new()
            },
            criteria: vec![ProviderCriterionOutcome {
                identifier: "server.fixture.execution".to_owned(),
                label: "Server Fixture Execution".to_owned(),
                status: status.to_owned(),
                expected: None,
                required: Some(true),
                message: (status != "passed").then(|| "fixture failure".to_owned()),
            }],
            failures: if status == "passed" {
                Vec::new()
            } else {
                vec![ProviderExecutionFailure {
                    stage: "template".to_owned(),
                    template_identifier: Some(case.metadata.identifier.as_str().to_owned()),
                    module: None,
                    message: "fixture failure".to_owned(),
                    detail: None,
                }]
            },
        });

        if status != "passed" && !cycle.continue_on_failure {
            break;
        }
    }

    ProviderCycleOutcome {
        status: if failures.is_empty() {
            "passed".to_owned()
        } else {
            "failed".to_owned()
        },
        dependency_imports: vec![ProviderDependencyProbe {
            module: "server_test.fixture".to_owned(),
            status: "imported".to_owned(),
            version: None,
            error: None,
        }],
        provider: ProviderStatusOutcome {
            status: "active".to_owned(),
            domain_registration_count: 1,
            test_suite_count: 1,
            test_suite_identifiers: cycle_suite_identifiers(cycle),
            error: None,
        },
        executions,
        failures,
    }
}

/* === Private Functions === */
async fn register_fixture_registration(runtime: &RuntimeHandle, registration: TestSuiteDefinition) {
    runtime
        .set_suites(
            "provider:fixture",
            dokime_runtime::definition::SuiteSet::new([registration]),
        )
        .await
        .unwrap();
}

fn cycle_suite_identifiers(cycle: &CycleView) -> Vec<String> {
    cycle
        .cases()
        .filter_map(|case| case.suite.as_ref())
        .chain(
            cycle
                .test_executions
                .iter()
                .filter_map(|execution| execution.suite.as_ref()),
        )
        .map(|suite| suite.identifier.as_str().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn case_template(index: usize, result: &str) -> TestCaseTemplate {
    named_case_template("fixture.template", "Fixture Template", index, result)
}

fn named_case_template(
    identifier: &str,
    label: &str,
    index: usize,
    result: &str,
) -> TestCaseTemplate {
    TestCaseTemplate {
        test_case_template_id: dokime_runtime::model::identity::new_id::<TemplateId>(),
        metadata: metadata(
            &format!("{identifier}.{index}"),
            &format!("{label} {index}"),
        ),
        tags: vec!["fixture".to_owned()],
        parameter_groups: Vec::new(),
        parameters: vec![ParameterDefinition {
            expansion: Default::default(),
            min: None,
            max: None,
            parameter_id: dokime_runtime::model::identity::new_id::<ParameterId>(),
            metadata: metadata("fixture.result", "Fixture Result"),
            group_id: None,
            value_type: ValueType::Text,
            unit: None,
            required: false,
            expression: ParameterExpr::Literal {
                value: Value::from(result),
            },
            runtime_writable: false,
            selection: None,
            list: None,
            extensions: Default::default(),
        }],
        files: Vec::new(),
        phases: vec![ExecutionPhaseDefinition {
            repeatable: false,
            step_id: dokime_runtime::model::identity::new_id::<dokime_runtime::StepId>(),
            metadata: metadata("fixture.phase", "Fixture Phase"),
        }],
        criteria: vec![CriterionDefinition {
            criterion_id: dokime_runtime::model::identity::new_id::<dokime_runtime::CriterionId>(),
            metadata: metadata("fixture.criterion", "Fixture Criterion"),
            expected: Some(Value::from(result)),
        }],
        continue_on_failure: false,
        prompt_definitions: Vec::new(),
        subscriptions: Vec::new(),
        document: None,
        attributes: BTreeMap::new(),
    }
}

fn fixture_progress_contract() -> TelemetryContract {
    TelemetryContract {
        stream: Identifier::new("fixture.progress").unwrap(),
        schema_version: 1,
        storage_schema: Some(
            harmos::schema::graph::Schema::from_declaration(
                <ProgressRow as harmos::schema::Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        ),
        metadata: metadata("fixture.progress", "Fixture Progress"),
        template_id: None,
        owner: None,
        columns: vec![TelemetryColumn::new(
            Metadata::new(Identifier::new("progress").unwrap(), "Progress", None),
            ValueType::Number,
            TelemetryRole::Value,
        )],
        default_x: Some("timestamp".to_owned()),
        interval: None,
        plot_window: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

fn fixture_progress_telemetry(progress: f64) -> ProviderTelemetryOutcome {
    ProviderTelemetryOutcome {
        stream: "fixture.progress".to_owned(),
        label: "Fixture Progress".to_owned(),
        samples: vec![dokime_runtime::transport::ProviderTelemetryRow {
            values: BTreeMap::from([("progress".to_owned(), Value::Number(progress))]),
            payload: harmos::Message::encode_to_vec(&ProgressRow { progress }),
        }],
    }
}

/* === Tests === */
