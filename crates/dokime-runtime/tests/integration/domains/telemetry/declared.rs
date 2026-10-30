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
// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::definition::SuiteSet;
use dokime_runtime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime_runtime::harmos::plugin::Declared;
use dokime_runtime::{Refusal, Supervisor};

/* === Constants === */
const SOURCE: &str = "fixture_declaring";
const SUITE: &str = "fixture.declaring.checks";
const RECORD: &str = "fixture.declaring.measured";
const PROMPT: &str = "fixture.declaring.confirm";
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(120),
    attempt_timeout: Duration::from_secs(120),
    cancellation_timeout: std::time::Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::stream(id = "fixture.declaring.samples", window = 32, label = "Samples")]
struct Sample {
    value: f64,
}

#[dokime_runtime::record(
    id = "fixture.declaring.measured",
    label = "Measured",
    description = "The value a template measured, filed by both templates of the suite."
)]
#[derive(Clone, Debug)]
struct Measured {
    value: f64,
}

#[dokime_runtime::prompt(
    id = "fixture.declaring.confirm",
    label = "Confirm The Bench",
    description = "Confirm the bench is ready before the measurement starts."
)]
struct Confirm;

struct Checks;

struct First;

struct Second;

struct Declaring;

/* === Enums === */
#[dokime_runtime::catalog]
enum Streams {
    Sample(Sample),
}

#[dokime_runtime::catalog]
enum Records {
    Measured(Measured),
}

#[dokime_runtime::catalog]
enum Prompts {
    Confirm(Confirm),
}

#[dokime_runtime::catalog]
enum Templates {
    First(First),
    Second(Second),
}

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::suite(id = "fixture.declaring.checks", label = "Checks", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(
    id = "fixture.declaring.checks.first",
    label = "First",
    streams = Streams,
    records = Records,
    prompts = Prompts
)]
impl First {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Sample { value: 1.0 }).await;
        ctx.publish(Measured { value: 1.0 }).await;
        Ok(())
    }
}

#[dokime_runtime::template(
    id = "fixture.declaring.checks.second",
    label = "Second",
    streams = Streams,
    records = Records,
    prompts = Prompts
)]
impl Second {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Sample { value: 2.0 }).await;
        ctx.publish(Measured { value: 2.0 }).await;
        Ok(())
    }
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for Declaring {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime_runtime::plugin(
    id = "fixture_declaring",
    requires(),
    contributions(dokime_runtime::Suites<Suites>),
    restart = "never",
    description = "Registers what its templates declare and repeats none of it."
)]
impl dokime_runtime::Plugin for Declaring {
    const CATALOGS: &'static [dokime_runtime::harmos::Catalog] = &[];
}

impl dokime_runtime::harmos::Lifecycle for Declaring {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(
        _config: dokime_runtime::plugin::Configuration, setup: &mut dokime_runtime::harmos::Setup<Self>,
    ) -> Result<Self, Refusal> {
        let owner = { Self };
        setup.resource::<()>(owner.resources());
        dokime_runtime::execution::prepare::<Self, Suites>(setup)?;
        Ok(owner)
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

impl Declaring {
    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
#[tokio::test]
async fn template_declarations_register() {
    // The Plugin Grants The Shared Stream Once
    let granted = <Declaring as Declared>::STREAMS.iter().map(|entry| entry.id).collect::<Vec<_>>();
    assert_eq!(granted, [Sample::ID], "two templates share one stream grant");

    let root = tempdir().unwrap();
    let native = dokime_runtime::testing::plugin::<Declaring>(config(root.path()), root.path()).unwrap();
    let mut runtime = Runtime::start(native).await.unwrap();
    let handle = runtime.handle();

    // Admission Registered The Template Record And Prompt
    let records = handle.record_contracts().await;
    assert!(records.iter().any(|contract| contract.record_type.as_str() == RECORD), "{records:?}");
    let prompts = handle.prompt_definitions().await;
    assert!(prompts.iter().any(|prompt| prompt.prompt_type.as_str() == PROMPT), "{prompts:?}");

    // Both Templates Publish What They Declared
    let suite = Checks::definition().suite;
    let cycle = start_cases(&handle, metadata("fixture.declaring.cycle", "Declaring Cycle"), materialize_cases(SUITE, &suite)).await;
    let receipt = handle.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_cycle_outcome(&handle, &cycle, ResultStatus::Passed).await;
    let rows = handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![Identifier::new(Sample::ID).unwrap().into()],
            scope: ScopeSelection {
                test_cycle_id: Some(cycle.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(rows.len(), 2, "both templates publish the shared stream");
    let ended = handle.cycles().await.into_iter().find(|item| item.test_cycle_id == cycle).unwrap();
    assert!(
        ended.test_executions.iter().all(|execution| execution.status == dokime_domain::model::execution::result::ExecutionStatus::Passed)
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn conflicting_record_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut definition = Checks::definition();
    let second = <Second as dokime_runtime::definition::DeclaredTemplate>::template_id();
    let suite = &mut definition;
    for contract in &mut suite.record_contracts {
        if contract.template_id.as_ref() == Some(&second) {
            contract.fields[0].required = !contract.fields[0].required;
        }
    }

    let refused = runtime.handle().set_suites(SOURCE, SuiteSet::new([definition])).await;

    assert!(
        refused.as_ref().is_err_and(|error| error.to_string().contains(&format!("event definition {RECORD} is already registered"))),
        "{refused:?}"
    );
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn conflicting_prompt_refused() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::start(config(root.path())).await.unwrap();
    let mut definition = Checks::definition();
    let suite = &mut definition;
    suite.suite.test_case_templates[1].prompt_definitions[0].title = "Another Title".into();

    let refused = runtime.handle().set_suites(SOURCE, SuiteSet::new([definition])).await;

    assert!(
        refused.as_ref().is_err_and(|error| error.to_string().contains(&format!("conflicting definitions for prompt {PROMPT}"))),
        "{refused:?}"
    );
    runtime.shutdown().await.unwrap();
}
