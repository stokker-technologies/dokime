/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "authoring/error.rs"]
pub(crate) mod error;
pub(crate) mod catalog_lifecycle {
    use super::*;
    use dokime::catalog::{SuiteRunners, Suites};
    use dokime::definition::DeclaredSuite;
    use dokime::harmos::operations::{ServingError, WireTable};
    use std::sync::Mutex;

    static RECORDS: Mutex<Vec<&str>> = Mutex::new(Vec::new());
    struct CountedSuite;

    impl DeclaredSuite for CountedSuite {
        const OPERATIONS: &'static [dokime::harmos::catalog::Node<
            dokime::harmos::operations::WireDeclaration,
        >] = &[];

        fn definition() -> dokime::domains::TestSuiteDefinition {
            RECORDS.lock().unwrap().push("catalog");
            SpikeChecks::definition()
        }

        fn bind<P: dokime::harmos::plugin::contract::Declared>(
            _: &mut WireTable<dokime::Supervisor>,
            _: &str,
            _: &[dokime::records::RecordContract],
        ) -> Result<(), ServingError> {
            Ok(())
        }

        fn runners() -> Vec<Arc<dyn dokime::execution::Runnable>> {
            RECORDS.lock().unwrap().push("factories");

            Vec::new()
        }
    }

    #[dokime::catalog]
    enum CountedSuites {
        CountedSuite(CountedSuite),
    }

    pub(crate) fn registration_inert_once() {
        use dokime::harmos::plugin::contract::Contribution;

        let mut registrar = dokime::harmos::plugin::Registrar::new();
        dokime::Suites::<CountedSuites>::register(&mut registrar).unwrap();
        assert_eq!(registrar.finish().len(), 1);
        assert_eq!(RECORDS.lock().unwrap().as_slice(), ["catalog"]);
        RECORDS.lock().unwrap().clear();

        // Collecting the catalog's entries asks for nothing; only calling an
        // entry builds the executable half, and each call builds it once.
        let runners = dokime::catalog::entries::<CountedSuites, SuiteRunners>();
        let definitions = dokime::catalog::entries::<CountedSuites, Suites>();
        assert!(RECORDS.lock().unwrap().is_empty());
        for runner in runners {
            runner();
        }
        assert_eq!(RECORDS.lock().unwrap().as_slice(), ["factories"]);
        for definition in definitions {
            definition();
        }
        assert_eq!(RECORDS.lock().unwrap().as_slice(), ["factories", "catalog"]);
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
// Third Party Imports

// Local Imports
use dokime::core::{Value, ValueType};
use dokime::definition::{
    Criterion, DeclaredTemplate, SuiteSet, authored_criterion as criterion,
    materialize_template_cases,
};
use dokime::execution::{CriterionContext, ObserverContext, StepContext};
use dokime::execution::{Run, Signal, Workflow};
use dokime::harmos::operations::OperationDeclaration;
use dokime::harmos::plugin::Registrar;
use dokime::harmos::plugin::contract::Contribution;
use dokime::plugin::registration::{SUITES_NAMESPACE, TELEMETRY_NAMESPACE};
use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use dokime::transport::{
    EvidenceBatch, RunExecutionRequest, SidecarCriterionStatus, StepAdmission,
};
use error::Error;

/* === Constants === */
const SUITE_ID: &str = "spike.checks";
const TEMPLATE_ID: &str = "spike.checks.probe";
const STREAM_ID: &str = "spike.pulse";
const PULSE_SUITE_ID: &str = "spike.pulses";
const PULSE_TEMPLATE_ID: &str = "spike.pulses.pulse";
const PULSE_STREAM_ID: &str = <PulseRow as dokime::StreamSpec>::ID;
const PULSE_CRITERION_ID: &str = "spike.pulses.acceptance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct SpikeChecks;

struct Probe;

#[dokime::record(
    id = "spike.run.started",
    label = "Started",
    description = "One fact the probe narrates, declared beside the template that publishes it."
)]
struct Started {
    #[record(description = "How many rows the run had produced when it said so.")]
    produced: u64,
}

struct PulseChecks;

struct Pulse;

#[derive(Default)]
struct PulseResources {
    produced: AtomicU64,
}

#[dokime::stream(
    id = "spike.pulses.rows",
    window = 4096,
    label = "Pulse Rows",
    description = "One row of the pulse stream, and with it the stream's own declaration."
)]
struct PulseRow {
    #[row(role = counter)]
    index: i64,
    value: f64,
}

#[dokime::parameters]
pub struct ProbeParameters {
    #[parameter(label = "Sample Count", default = 8)]
    pub sample_count: i64,
}

#[dokime::parameters]
pub struct PulseParameters {
    #[parameter(
        label = "Rows",
        default = 3,
        min = 1,
        description = "A bound the case is refused by rather than one a step has to handle."
    )]
    pub rows: u64,
}

/* === Enums === */

/* === Traits === */

#[dokime::catalog]
enum RowsStreams {
    PulseRow(PulseRow),
}

/* === Implementations === */
#[dokime::catalog]
enum SpikeChecksTemplates {
    Probe(Probe),
}

#[dokime::suite(
    id = SUITE_ID,
    label = "Spike Checks",
    product = "Spike",
    tags = ["example", "plugin"],
    templates = SpikeChecksTemplates)]
impl SpikeChecks {
    fn telemetry() -> Vec<TelemetryContract> {
        vec![
            TelemetryContract::declared(STREAM_ID, "Spike Pulse")
                .for_template(Probe::template_id())
                .column(TelemetryColumn::declared(
                    "value",
                    "Value",
                    ValueType::Number,
                    TelemetryRole::Value,
                ))
                .strict(),
        ]
    }
}

#[dokime::catalog]
enum ProbeRecords {
    Started(Started),
}

#[dokime::catalog]
enum ProbeSubscribes {
    Started(Started),
}

#[dokime::template(
    id = TEMPLATE_ID,
    label = "Probe",
    tags = ["example", "plugin"],
    records = ProbeRecords,
    subscribes = ProbeSubscribes,
    attributes(path = "Spike/Probe"),
    parameters = ProbeParameters)]
impl Probe {
    async fn initialize(
        _: &ProbeParameters,
        _: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<Self, Error> {
        Ok(Self)
    }

    fn criteria() -> Vec<Criterion> {
        vec![criterion(
            "spike.checks.probe.acceptance",
            "Probe Completed",
            Some(Value::from(true)),
        )]
    }

    #[dokime::step(
        label = "Start",
        description = "Arm the probe.\nThe **first** line is a summary; the rest is Markdown."
    )]
    fn start(&mut self, _run: &mut StepContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    #[dokime::step(label = "Complete", description = "Disarm and report.")]
    fn complete(&mut self, _run: &mut StepContext<'_>) -> Result<(), Error> {
        Ok(())
    }
}

#[dokime::catalog]
enum PulseChecksTemplates {
    Pulse(Pulse),
}

#[dokime::suite(
    id = PULSE_SUITE_ID,
    label = "Spike Pulses",
    templates = PulseChecksTemplates)]
impl PulseChecks {}

#[dokime::template(
    id = PULSE_TEMPLATE_ID,
    label = "Pulse",
    parameters = PulseParameters, description = "Steps, a criterion, and a stream, written as functions.")]
impl Pulse {
    async fn initialize(
        _: &PulseParameters,
        _ctx: &mut dokime::execution::InitializationContext<'_>,
    ) -> Result<(Self, PulseResources), Error> {
        let template = Self;
        let shared = PulseResources::default();
        Ok((template, shared))
    }

    #[dokime::step(label = "Measure")]
    async fn measure(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let wanted = ctx.parameters().rows;
        while ctx.resources().produced.load(Ordering::Acquire) < wanted {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }

        Ok(())
    }

    #[dokime::criterion(id = PULSE_CRITERION_ID, label = "Rows Measured")]
    async fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        tokio::task::yield_now().await;
        let samples = ctx.samples::<PulseRow>();
        for (index, sample) in samples.iter().enumerate() {
            if !sample.value.is_finite() {
                return Err(Error::Unmeasured(index));
            }
        }

        Ok(samples.len() as u64 >= ctx.parameters().rows)
    }

    #[dokime::observer(interval_ms = 20, streams = RowsStreams)]
    async fn rows(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        tokio::task::yield_now().await;
        let index = ctx.resources().produced.fetch_add(1, Ordering::AcqRel);
        let value = ctx.tick().index() as f64 * 0.25;
        ctx.publish(PulseRow {
            index: index as i64,
            value,
        })
        .await;

        Ok(())
    }
}

#[dokime::catalog]
enum SpikeSuites {
    SpikeChecks(SpikeChecks),
    PulseChecks(PulseChecks),
}

/* === Public Functions === */

/* === Private Functions === */
fn pulse_start() -> RunExecutionRequest {
    RunExecutionRequest {
        mode: "automatic".into(),
        execution_token: "authoring".to_owned(),
        test_cycle_id: "cycle-1".to_owned(),
        test_execution_id: "execution-1".to_owned(),
        template_identifier: PULSE_TEMPLATE_ID.to_owned(),
        workflow: PULSE_SUITE_ID.to_owned(),

        secrets_json: "{}".to_owned(),
        parameters_json: serde_json::to_string(&std::collections::BTreeMap::from([(
            "rows",
            dokime::Value::Integer(3),
        )]))
        .unwrap(),
    }
}

async fn drain_until_terminal(execution: &Workflow<Pulse>) -> Vec<EvidenceBatch> {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut replies = Vec::new();
    let mut cursor = 0;
    loop {
        let reply = execution
            .drain("authoring", cursor)
            .expect("the execution remains drainable");
        for encoded in &reply.step_begins {
            let start: dokime::execution::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            execution.admit(
                "authoring",
                &StepAdmission {
                    attempt_id: start.attempt_id.to_string(),
                    accepted: true,
                    message: String::new(),
                },
            );
        }
        cursor = reply.cursor;
        let terminal = reply.outcome.is_some();
        replies.push(reply);
        if terminal {
            return replies;
        }
        assert!(
            Instant::now() < deadline,
            "the execution never reached a terminal outcome"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

pub(crate) async fn attributed_functions_runs() {
    let execution = Workflow::<Pulse>::new(PULSE_SUITE_ID, "authoring-spike");
    execution
        .start(pulse_start())
        .expect("the execution starts");

    let replies = drain_until_terminal(&execution).await;
    let samples = execution.take_local_samples("authoring").unwrap();
    let outcome = replies
        .iter()
        .find_map(|reply| reply.outcome.as_ref())
        .expect("the drain stream terminates once");

    // The declared stream sampled, the step waited on what it produced, and the
    // criterion decided over it — three attributed functions, one run.
    assert!(samples.len() >= 3);
    assert_eq!(samples[0].stream.as_str(), PULSE_STREAM_ID);
    assert_eq!(samples[1].values.get("index"), Some(&Value::Integer(1)));
    assert_eq!(replies.last().expect("a terminal reply").phase, "measure");
    assert!(outcome.passed);
    assert_eq!(outcome.criteria[0].identifier, PULSE_CRITERION_ID);
    assert_eq!(
        SidecarCriterionStatus::try_from(outcome.criteria[0].status),
        Ok(SidecarCriterionStatus::Passed)
    );
    execution.shutdown().await;
}

/* === Tests === */

pub(crate) fn registration_phase_envelope() {
    let mut registrar = Registrar::new();
    dokime::Suites::<SpikeSuites>::register(&mut registrar)
        .expect("the spike registers its suites");
    let registrations = registrar.finish();
    assert_eq!(registrations.len(), 1);
    let suites = registrations
        .iter()
        .find(|registration| registration.namespace == SUITES_NAMESPACE)
        .expect("suite domain");
    assert_eq!(
        suites.schema,
        dokime::plugin::registration::domain_schema(SUITES_NAMESPACE)
    );
    assert!(
        !registrations
            .iter()
            .any(|registration| registration.namespace == TELEMETRY_NAMESPACE),
        "template telemetry stays with its suite; an empty standalone domain is absent"
    );
    let decoded = serde_json::from_slice::<SuiteSet>(&suites.payload).expect("valid suites");
    assert_eq!(
        decoded.suites[0].suite_id,
        SpikeChecks::definition().suite_id
    );
    assert_eq!(
        decoded.suites[0].suite.test_case_templates[0].test_case_template_id,
        Probe::template_id()
    );
}

pub(crate) fn template_declares_operations() {
    assert_eq!(
        <Run<Pulse> as OperationDeclaration>::ID,
        format!("{PULSE_TEMPLATE_ID}.run")
    );
    assert_eq!(
        <Signal<Pulse> as OperationDeclaration>::ID,
        format!("{PULSE_TEMPLATE_ID}.signal")
    );
    assert_eq!(
        <Run<Pulse> as OperationDeclaration>::POLICY.timeout,
        dokime::transport::RUN_BOUND
    );
}

pub(crate) fn suite_declaration_contracts() {
    let definition = SpikeChecks::definition();

    assert_eq!(definition.suite_id, SUITE_ID);
    assert_eq!(definition.product.as_deref(), Some("Spike"));
    assert_eq!(definition.workflow.as_deref(), Some(SUITE_ID));
    assert_eq!(
        definition.suite.test_case_templates[0].subscriptions,
        vec![dokime::definition::Subscription::Record(
            <Started as dokime::records::DeclaredRecord>::contract().record_type
        )]
    );
    let encoded = serde_json::to_value(&definition).unwrap();
    assert_eq!(encoded["workflow"], SUITE_ID);
    assert!(encoded.get("execution").is_none());
    // One authored tag list reaches both the suite and its registration.
    assert_eq!(definition.tags, ["example", "plugin"]);
    assert_eq!(definition.suite.tags, definition.tags);
    assert_eq!(
        definition.telemetry_contracts[0].template_id,
        Some(Probe::template_id())
    );
    assert_eq!(
        definition.record_contracts[0].template_id,
        Some(Probe::template_id())
    );
}

pub(crate) fn one_step_phases() {
    let declaration = Probe::declaration();

    assert_eq!(
        declaration
            .phases
            .iter()
            .map(|phase| phase.metadata.identifier.as_str())
            .collect::<Vec<_>>(),
        ["start", "complete"]
    );
}

pub(crate) fn step_reads_rustdoc() {
    let declaration = Probe::declaration();

    assert_eq!(
        declaration.phases[0].metadata.description.as_deref(),
        Some("Arm the probe.\nThe **first** line is a summary; the rest is Markdown.")
    );
    assert_eq!(
        declaration.phases[1].metadata.description.as_deref(),
        Some("Disarm and report.")
    );
}

pub(crate) fn template_declaration_declares() {
    let declaration = Probe::declaration();

    assert_eq!(declaration.test_case_template_id, Probe::template_id());
    assert_eq!(declaration.metadata.identifier.as_str(), TEMPLATE_ID);
    assert_eq!(declaration.tags, ["example", "plugin"]);
    assert_eq!(
        declaration
            .parameters
            .iter()
            .map(|parameter| parameter.metadata.identifier.as_str())
            .collect::<Vec<_>>(),
        ["sample_count"]
    );
    let parameters = <ProbeParameters as dokime::parameters::FromParameters>::from_parameters(
        &std::collections::BTreeMap::from([("sample_count".to_owned(), Value::from(8_i64))]),
    )
    .unwrap();
    assert_eq!(parameters.sample_count, 8);
    assert_eq!(declaration.criteria.len(), 1);
    assert_eq!(
        declaration.attributes.get("path"),
        Some(&Value::from("Spike/Probe"))
    );
    // A template that declares no document contributes none.
    assert!(declaration.document.is_none());
    assert_eq!(
        materialize_template_cases(
            "fixture.registration",
            &declaration,
            Default::default(),
            Default::default()
        )
        .expect("the declaration generates cases")
        .case_count,
        1
    );
}

pub(crate) fn obsolete_template_refused() {
    let current = serde_json::to_value(Probe::declaration()).unwrap();
    assert!(
        serde_json::from_value::<dokime::definition::TestCaseTemplate>(current.clone()).is_ok()
    );
    for (field, value) in [
        ("prompts", serde_json::json!(true)),
        ("samples", serde_json::json!(["host.system"])),
    ] {
        let mut obsolete = current.clone();
        obsolete[field] = value;
        assert!(serde_json::from_value::<dokime::definition::TestCaseTemplate>(obsolete).is_err());
    }
}

pub(crate) fn second_suite_refused() {
    let mut registrar = Registrar::new();
    dokime::Suites::<SpikeSuites>::register(&mut registrar)
        .expect("the first suite set is accepted");

    let refusal = dokime::Suites::<SpikeSuites>::register(&mut registrar)
        .expect_err("one artifact exports its suites once");

    assert_eq!(
        refusal.to_string(),
        "registration namespace `dokime.suites` was declared more than once"
    );
}

pub(crate) fn criterion_method_decides() {
    let declaration = Pulse::declaration();

    // No `criteria()` list anywhere in the block: the method that decides it is
    // what declared it.
    assert_eq!(
        declaration
            .criteria
            .iter()
            .map(|criterion| criterion.metadata.identifier.as_str())
            .collect::<Vec<_>>(),
        [PULSE_CRITERION_ID]
    );
    assert_eq!(declaration.criteria[0].metadata.label, "Rows Measured");
}

pub(crate) fn stream_method_publishes() {
    let definition = PulseChecks::definition();
    // The suite declares no telemetry of its own; this came off the template.
    let contract = &definition.telemetry_contracts[0];

    assert_eq!(contract.stream.as_str(), PULSE_STREAM_ID);
    assert_eq!(contract.template_id, Some(Pulse::template_id()));
    assert!(!contract.allow_extra_fields);
    // The columns were resolved from the row type's own fields: the canonical
    // timestamp, then the fields in the order they are written.
    assert_eq!(
        contract
            .columns
            .iter()
            .map(|column| (
                column.metadata.identifier.as_str(),
                column.value_type.clone(),
                column.role.clone(),
            ))
            .collect::<Vec<_>>(),
        [
            ("timestamp", ValueType::Datetime, TelemetryRole::Value),
            ("index", ValueType::Integer, TelemetryRole::Counter),
            ("value", ValueType::Number, TelemetryRole::Value),
        ]
    );
}
