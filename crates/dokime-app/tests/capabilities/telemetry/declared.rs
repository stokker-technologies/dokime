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

// Third Party Imports
use async_trait::async_trait;
// Local Imports
use crate::support::prelude::*;
use dokime::execution::Runnable;
use dokime::harmos::operations::{OperationPolicy, RestartPolicy, RetryPolicy};
use dokime::harmos::operations::{ServingError, WireTable};
use dokime::harmos::plugin::Declared;
use dokime::{Refusal, Supervisor};
use dokime_app::InstalledPlugin;
use dokime_app::definition::SuiteSet;
use dokime_app::runtime::EmbeddedProvider;
use harmos::sidecar::StreamDeclaration;

/* === Constants === */
const SOURCE: &str = "fixture.declaring";
const SUITE: &str = "fixture.declaring.checks";
const RECORD: &str = "fixture.declaring.measured";
const PROMPT: &str = "fixture.declaring.confirm";
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::stream(id = "fixture.declaring.samples", window = 32, label = "Samples")]
struct Sample {
    value: f64,
}

#[dokime::record(
    id = "fixture.declaring.measured",
    label = "Measured",
    description = "The value a template measured, filed by both templates of the suite."
)]
#[derive(Clone, Debug)]
struct Measured {
    value: f64,
}

#[dokime::prompt(
    id = "fixture.declaring.confirm",
    label = "Confirm The Bench",
    description = "Confirm the bench is ready before the measurement starts."
)]
struct Confirm;

struct Checks;

struct First;

struct Second;

/// A plugin that contributes only its suite: every stream, record and prompt
/// it registers is one its templates declared.
struct Declaring;

struct DeclaringProvider {
    catalog: InstalledPlugin,
}

/* === Enums === */
#[dokime::catalog]
enum Streams {
    Sample(Sample),
}

#[dokime::catalog]
enum Records {
    Measured(Measured),
}

#[dokime::catalog]
enum Prompts {
    Confirm(Confirm),
}

#[dokime::catalog]
enum Templates {
    First(First),
    Second(Second),
}

#[dokime::catalog]
enum Suites {
    Checks(Checks),
}

/* === Traits === */

/* === Implementations === */
#[dokime::suite(id = "fixture.declaring.checks", label = "Checks", templates = Templates)]
impl Checks {}

#[dokime::template(
    id = "fixture.declaring.checks.first",
    label = "First",
    streams = Streams,
    records = Records,
    prompts = Prompts
)]
impl First {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Measure")]
    async fn measure(
        &mut self,
        ctx: &mut dokime::execution::StepContext<'_>,
    ) -> dokime::Result<()> {
        ctx.publish(Sample { value: 1.0 }).await;
        ctx.publish(Measured { value: 1.0 }).await;
        Ok(())
    }
}

#[dokime::template(
    id = "fixture.declaring.checks.second",
    label = "Second",
    streams = Streams,
    records = Records,
    prompts = Prompts
)]
impl Second {
    async fn initialize(
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> dokime::Result<Self> {
        Ok(Self)
    }

    #[dokime::step(label = "Measure")]
    async fn measure(
        &mut self,
        ctx: &mut dokime::execution::StepContext<'_>,
    ) -> dokime::Result<()> {
        ctx.publish(Sample { value: 2.0 }).await;
        ctx.publish(Measured { value: 2.0 }).await;
        Ok(())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for Declaring {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::plugin(
    id = "fixture-declaring",
    provides = (),
    requires(),
    contributions(dokime::Suites<Suites>),
    restart = "never",
    description = "Registers what its templates declare and repeats none of it."
)]
impl dokime::Plugin for Declaring {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(_: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        Ok(Self)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        dokime::execution::serve::<Self, Suites>(table)
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

impl DeclaringProvider {
    fn new() -> Self {
        Self {
            catalog: InstalledPlugin {
                source: SOURCE.into(),
                version: "0.1.0".into(),
                enabled: true,
                suites: SuiteSet::new([Checks::definition()]),
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl EmbeddedProvider for DeclaringProvider {
    fn catalog(&self) -> InstalledPlugin {
        self.catalog.clone()
    }

    // The grant the plugin attribute derived, exactly as its artifact carries it.
    fn streams(&self) -> Vec<StreamDeclaration> {
        <Declaring as Declared>::STREAMS
            .iter()
            .map(|entry| StreamDeclaration {
                metadata: entry.metadata(),
                row_metadata: entry.row_metadata(),
                id: entry.id.into(),
                schema: entry.schema,
                window: entry.window,
                row: harmos::schema::graph::Schema::from_declaration(
                    entry.row,
                    &harmos::config::SchemaSettings::default(),
                )
                .unwrap(),
            })
            .collect()
    }

    async fn start(&self) -> dokime_app::Result<Vec<Arc<dyn Runnable>>> {
        Ok(vec![
            Arc::new(dokime::execution::Workflow::<First>::new(SUITE, SOURCE)) as Arc<dyn Runnable>,
            Arc::new(dokime::execution::Workflow::<Second>::new(SUITE, SOURCE)),
        ])
    }

    async fn stop(&self) -> dokime_app::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
pub(crate) async fn template_declarations_register() {
    // The Plugin Grants The Shared Stream Once
    let granted = <Declaring as Declared>::STREAMS
        .iter()
        .map(|entry| entry.id)
        .collect::<Vec<_>>();
    assert_eq!(
        granted,
        [Sample::ID],
        "two templates share one stream grant"
    );

    let root = tempdir().unwrap();
    let mut runtime =
        Runtime::start(config(root.path()).register(Arc::new(DeclaringProvider::new())))
            .await
            .unwrap();
    let handle = runtime.handle();

    // Admission Registered The Template Record And Prompt
    let records = handle.record_contracts().await;
    assert!(
        records
            .iter()
            .any(|contract| contract.record_type.as_str() == RECORD),
        "{records:?}"
    );
    let prompts = handle.prompt_definitions().await;
    assert!(
        prompts
            .iter()
            .any(|prompt| prompt.prompt_type.as_str() == PROMPT),
        "{prompts:?}"
    );

    // Both Templates Publish What They Declared
    let suite = Checks::definition().suite;
    let cycle = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: metadata("fixture.declaring.cycle", "Declaring Cycle"),
                cases: materialize_cases(SUITE, &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_cycle_lifecycle(&handle, &cycle, CycleStatus::Completed).await;
    let rows = handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![Identifier::new(Sample::ID).unwrap()],
            scope: ScopeSelection {
                test_cycle_id: Some(cycle.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await;
    assert_eq!(rows.len(), 2, "both templates publish the shared stream");
    let ended = handle
        .cycles()
        .await
        .into_iter()
        .find(|item| item.test_cycle_id == cycle)
        .unwrap();
    assert!(
        ended
            .test_executions
            .iter()
            .all(|execution| execution.status == dokime_app::execution::ExecutionStatus::Passed)
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn conflicting_record_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut provider = DeclaringProvider::new();
    let second = <Second as dokime::definition::DeclaredTemplate>::template_id();
    let suite = &mut provider.catalog.suites.suites[0];
    for contract in &mut suite.record_contracts {
        if contract.template_id.as_ref() == Some(&second) {
            contract.fields[0].required = !contract.fields[0].required;
        }
    }

    let refused = runtime.register(Arc::new(provider)).await;

    assert!(
        refused.as_ref().is_err_and(|error| error
            .to_string()
            .contains(&format!("event definition {RECORD} is already registered"))),
        "{refused:?}"
    );
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn conflicting_prompt_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut provider = DeclaringProvider::new();
    let suite = &mut provider.catalog.suites.suites[0];
    suite.suite.test_case_templates[1].prompt_definitions[0].title = "Another Title".into();

    let refused = runtime.register(Arc::new(provider)).await;

    assert!(
        refused.as_ref().is_err_and(|error| error
            .to_string()
            .contains(&format!("conflicting definitions for prompt {PROMPT}"))),
        "{refused:?}"
    );
    runtime.shutdown().await.unwrap();
}
