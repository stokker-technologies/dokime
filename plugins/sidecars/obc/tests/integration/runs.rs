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
use std::sync::{Arc, Mutex};

// Third Party Imports

// Local Imports
use super::answering;
use dokime::domain::model;
use dokime::execution::{Caller, SidecarTemplate, Workflow};
use dokime::prompts::{PromptRequest, PromptResponse};
use dokime::transport::{EvidenceBatch, RunExecution, SidecarCriterionStatus, SidecarExecutionOutcome, StepAdmission};
use dokime::{PromptId, Timestamp};
use plugin_obc::domain::model::connections::Connections as ListedConnections;
use plugin_obc::domain::model::readiness::Readiness;
use plugin_obc::domain::operations::AskReadiness;
use plugin_obc::domain::operations::ListConnections;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Connections = Arc<Mutex<Vec<String>>>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn assert_record_contract(request: model::records::publication::RecordPublishRequest) {
    let contract = dokime::catalog::entries::<plugin_obc::suites::Suites, dokime::catalog::Suites>()
        .into_iter()
        .flat_map(|definition| definition().record_contracts)
        .find(|contract| contract.record_type == request.record_type)
        .expect("emitted record is declared");
    let record = request.into_record(
        dokime::model::identity::new_id::<dokime::RecordId>(),
        contract.schema_version,
        Timestamp::new("2026-01-01T00:00:00Z"),
        plugin_obc::SOURCE,
    );
    dokime::records::validate_record(Some(&contract), &record, dokime::ValidationMode::Strict)
        .expect("authored evidence matches its declared contract");
}

fn road(resources: &Connections) -> Caller {
    let connections = Arc::clone(resources);
    Caller::local(BTreeMap::from([
        answering::<AskReadiness, _>(|_| Readiness { stopped: false }),
        answering::<ListConnections, _>(move |_| ListedConnections {
            resources: connections.lock().unwrap().clone(),
        }),
    ]))
}

fn workflow<T: SidecarTemplate>(suite: &str) -> Workflow<T> {
    Workflow::new(suite, plugin_obc::SOURCE)
}

fn start<T: SidecarTemplate>(workflow: &Workflow<T>, resources: &Connections, request: RunExecution) -> Result<(), dokime::Refusal> {
    workflow.start_on(request, None, road(resources))
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn declining_prompt_failure() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);
    start(&workflow, &resources, runtime_request("declined", false)).unwrap();

    let outcome = drain_runtime(&workflow, "declined", Some(false), false).await;

    assert!(!outcome.passed);
    assert!(outcome.message.as_deref().is_some_and(|message| message.starts_with("execution completed")));
    assert_eq!(outcome.criteria.len(), 1);
    assert_eq!(outcome.criteria[0].status, SidecarCriterionStatus::Failed as i32);
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn stopping_continuous_evidence() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);
    start(&workflow, &resources, runtime_request("continuous", true)).unwrap();

    let outcome = drain_runtime(&workflow, "continuous", None, true).await;

    assert!(outcome.passed, "{:?} {:?}", outcome.message, outcome.criteria);
    assert!(outcome.message.as_deref().is_some_and(|message| message.starts_with("execution completed")));
    assert_eq!(outcome.criteria[0].status, SidecarCriterionStatus::Passed as i32);
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn cancelling_steady_device() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::steady::Steady>(plugin_obc::suites::steady::STEADY_SUITE);
    let mut request = steady_request("cancel-steady");
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["duration_s"] = serde_json::to_value(dokime::Value::from(10.0)).unwrap();
    request.parameters_json = parameters.to_string();
    start(&workflow, &resources, request).unwrap();
    let outcome = drain_runtime(&workflow, "cancel-steady", None, true).await;
    assert!(!outcome.passed);
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn image_failure_device() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);
    let mut request = runtime_request("missing-image", false);
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["attach_reference"] = serde_json::to_value(dokime::Value::from(true)).unwrap();
    request.parameters_json = parameters.to_string();
    // No images_directory capture: lookup fails before attachment, after Charger startup.
    start(&workflow, &resources, request).unwrap();
    let outcome = drain_runtime(&workflow, "missing-image", Some(true), false).await;
    assert!(!outcome.passed);
    assert_eq!(
        outcome.criteria[0].status,
        SidecarCriterionStatus::Passed as i32,
        "attachment failure must preserve the already measured passing verdict"
    );
    assert!(outcome.message.as_deref().unwrap_or_default().contains("images_directory"));
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn protection_refuses_baseline() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::protection::Overcurrent>(plugin_obc::suites::protection::SUITE);
    let mut request = runtime_request("initial-overload", false);
    request.workflow = plugin_obc::suites::protection::SUITE.into();
    request.template_identifier = plugin_obc::suites::protection::TEMPLATE.into();
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["load_ohm"] = serde_json::to_value(dokime::Value::from(20.0)).unwrap();
    request.parameters_json = parameters.to_string();
    start(&workflow, &resources, request).unwrap();
    let outcome = drain_runtime(&workflow, "initial-overload", None, false).await;
    assert!(!outcome.passed);
    assert_eq!(outcome.criteria[0].status, SidecarCriterionStatus::Failed as i32);
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn one_token_inputs() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);
    start(&workflow, &resources, runtime_request("shared", false)).unwrap();

    let refusal =
        start(&workflow, &resources, runtime_request("shared", true)).expect_err("one token cannot start a second, different execution");

    assert!(refusal.to_string().contains("reused with different inputs"));
    start(&workflow, &resources, runtime_request("shared", false)).expect("an identical start is idempotent");
    workflow.cancel("shared");
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn faults_contained_ready() {
    let resources = Connections::default();
    let protection = workflow::<plugin_obc::suites::protection::Overcurrent>(plugin_obc::suites::protection::SUITE);
    let recovery = workflow::<plugin_obc::suites::recovery::PowerLoss>(plugin_obc::suites::recovery::SUITE);
    for (token, suite, template, fault_record) in [
        ("overload", plugin_obc::suites::protection::SUITE, plugin_obc::suites::protection::TEMPLATE, "obc.charger.protection.faulted"),
        ("power-loss", plugin_obc::suites::recovery::SUITE, plugin_obc::suites::recovery::TEMPLATE, "obc.charger.recovery.cleared"),
    ] {
        let mut request = runtime_request(token, false);
        request.workflow = suite.into();
        request.template_identifier = template.into();
        let (records, samples) = if token == "overload" {
            start(&protection, &resources, request).unwrap();
            settled_evidence(&protection, token).await
        } else {
            start(&recovery, &resources, request).unwrap();
            settled_evidence(&recovery, token).await
        };
        assert!(records.iter().any(|record| record.contains(fault_record)));
        assert_eq!(records.iter().filter(|record| record.contains(".stopped\"")).count(), 1);
        assert!(samples.iter().any(
            |sample| matches!(sample.values.get("phase"), Some(dokime::Value::Text(phase)) if phase == "faulted" || phase == "power_loss")
        ));
        // The bench is untouched by a fault the Charger contained.
        assert!(road(&resources).call::<AskReadiness>(AskReadiness {}).await.is_ok_and(|readiness| !readiness.stopped));
    }
    protection.shutdown().await;
    recovery.shutdown().await;
}

async fn settled_evidence<T: SidecarTemplate>(
    workflow: &Workflow<T>, token: &str,
) -> (Vec<String>, Vec<dokime::telemetry::TelemetrySample>) {
    let mut cursor = 0;
    let mut records = Vec::new();
    let mut samples = Vec::new();
    loop {
        let reply = workflow.next(token, cursor).await.unwrap();
        admit_steps(workflow, token, &reply);
        cursor = reply.cursor;
        for record in &reply.records {
            assert_record_contract(serde_json::from_str(record).unwrap());
        }
        records.extend(reply.records);
        samples.extend(workflow.take_local_samples(token).unwrap());
        if let Some(outcome) = reply.outcome {
            assert!(outcome.passed, "{token}: {:?}", outcome.message);
            assert_eq!(outcome.criteria[0].status, SidecarCriterionStatus::Passed as i32);
            return (records, samples);
        }
    }
}

fn connection(resource: &str) -> model::records::evidence::Record {
    let mut record = model::records::evidence::Record::new(
        dokime::model::identity::new_id(),
        Timestamp::new("2026-01-01T00:00:00Z"),
        dokime::Identifier::new("tool.connection.acquired").unwrap(),
        1,
        "fixture-gateway",
        dokime::records::RecordSeverity::Info,
    );
    record.details.extend([
        ("resource_id".to_owned(), resource.into()),
        ("resource_kind".to_owned(), "device".into()),
        ("connection_state".to_owned(), "connected".into()),
    ]);

    record
}

fn deliver<T: SidecarTemplate>(workflow: &Workflow<T>, resources: &Connections, token: &str, resource: &str) {
    resources.lock().unwrap().push(resource.to_owned());
    workflow.deliver_captured(
        1,
        &connection(resource),
        "fixture-gateway",
        &[dokime::RecordScope {
            test_execution_id: Some(dokime::TestExecutionId::from_uuid(execution_id(token)).unwrap()),
            ..Default::default()
        }],
    );
}

async fn awaited_line<T: SidecarTemplate>(workflow: &Workflow<T>, token: &str, cursor: &mut u64, prefix: &str) -> String {
    loop {
        let reply = workflow.next(token, *cursor).await.unwrap();
        admit_steps(workflow, token, &reply);
        *cursor = reply.cursor;
        if let Some(line) = reply.diagnostics.iter().find(|entry| entry.message.starts_with(prefix)) {
            return line.message.clone();
        }
    }
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn step_waits_connections() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);

    // The first run waits for a unit that nobody has reported yet, and says
    // the bench has been told about none.
    let mut request = runtime_request("first", false);
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["await_connection"] = serde_json::to_value(dokime::Value::from(true)).unwrap();
    request.parameters_json = parameters.to_string();
    start(&workflow, &resources, request).unwrap();
    let mut cursor = 0;
    assert_eq!(awaited_line(&workflow, "first", &mut cursor, "record=bench_connections").await, "record=bench_connections count=0");

    deliver(&workflow, &resources, "first", "unit-a");

    assert_eq!(awaited_line(&workflow, "first", &mut cursor, "record=connected").await, "record=connected resource=unit-a");
    let outcome = drain_runtime(&workflow, "first", Some(true), false).await;
    assert!(outcome.passed, "{:?}", outcome.message);

    // The second run is a different execution, and the unit the listener
    // heard about during the first one is still on the bench.
    let mut request = runtime_request("second", false);
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["await_connection"] = serde_json::to_value(dokime::Value::from(true)).unwrap();
    request.parameters_json = parameters.to_string();
    start(&workflow, &resources, request).unwrap();
    let mut cursor = 0;
    assert_eq!(
        awaited_line(&workflow, "second", &mut cursor, "record=bench_connections").await,
        "record=bench_connections count=1",
        "the listener fired for a record published during another execution"
    );
    deliver(&workflow, &resources, "second", "unit-b");
    let outcome = drain_runtime(&workflow, "second", Some(true), false).await;

    assert!(outcome.passed, "{:?}", outcome.message);
    workflow.shutdown().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn unit_refuses_waited() {
    let resources = Connections::default();
    let workflow = workflow::<plugin_obc::suites::startup::Startup>(plugin_obc::suites::startup::STARTUP_SUITE);
    let mut request = runtime_request("silent", false);
    let mut parameters: serde_json::Value = serde_json::from_str(&request.parameters_json).unwrap();
    parameters["await_connection"] = serde_json::to_value(dokime::Value::from(true)).unwrap();
    parameters["connection_budget_s"] = serde_json::to_value(dokime::Value::from(0.05)).unwrap();
    request.parameters_json = parameters.to_string();
    start(&workflow, &resources, request).unwrap();

    // Nothing is delivered; the step's own budget is what ends the wait.
    let outcome = drain_runtime(&workflow, "silent", None, false).await;

    assert!(!outcome.passed);
    let message = outcome.message.unwrap_or_default();
    assert!(
        message.contains("the deadline elapsed before the stream produced an item"),
        "the deadline reaches the SDK error boundary: {message}"
    );
    workflow.shutdown().await;
}

fn execution_id(token: &str) -> String {
    dokime::TestExecutionId::from_seed(token).to_string()
}

fn runtime_request(token: &str, continuous: bool) -> RunExecution {
    RunExecution {
        mode: "automatic".into(),
        execution_token: token.to_owned(),
        test_cycle_id: "cycle".to_owned(),
        test_execution_id: execution_id(token),
        template_identifier: plugin_obc::suites::startup::STARTUP_TEMPLATE.to_owned(),
        workflow: plugin_obc::suites::startup::STARTUP_SUITE.to_owned(),

        secrets_json: "{}".to_owned(),
        parameters_json: serde_json::json!({
            "ac_voltage_v": dokime::Value::from(230.0), "load_ohm": dokime::Value::from(100.0), "frequency_hz": dokime::Value::from(50.0), "phase_deg": dokime::Value::from(15.0), "current_limit_a": dokime::Value::from(5.0), "duration_s": dokime::Value::from(0.002),
            "sample_interval_s": dokime::Value::from(0.001),
            "emit_artifact": dokime::Value::from(false),
            "attach_reference": dokime::Value::from(false),
            "realtime": dokime::Value::from(false),
            "continuous": dokime::Value::from(continuous),
            "await_connection": dokime::Value::from(false),
            "connection_budget_s": dokime::Value::from(30.0),
            // The two list-declaring templates this helper also starts: a
            // resolved case carries every parameter its template declares,
            // and a template that declares none simply never reads these.
            "load_steps": dokime::Value::Array(::std::vec![
                dokime::Value::from(100.0),
                dokime::Value::from(50.0),
                dokime::Value::from(10.0),
            ]),
            "outage_durations_s": dokime::Value::Array(::std::vec![
                dokime::Value::from(0.05),
                dokime::Value::from(0.1),
            ]),
            "recovery_limits_s": dokime::Value::Array(::std::vec![
                dokime::Value::from(0.02),
                dokime::Value::from(0.05),
            ]),
        })
        .to_string(),
    }
}

fn steady_request(token: &str) -> RunExecution {
    RunExecution {
        mode: "automatic".into(),
        execution_token: token.to_owned(),
        test_cycle_id: "cycle".to_owned(),
        test_execution_id: execution_id(token),
        template_identifier: plugin_obc::suites::steady::STEADY_TEMPLATE.to_owned(),
        workflow: plugin_obc::suites::steady::STEADY_SUITE.to_owned(),

        secrets_json: "{}".to_owned(),
        parameters_json: serde_json::json!({
            "ac_voltage_v": dokime::Value::from(230.0), "load_ohm": dokime::Value::from(100.0), "frequency_hz": dokime::Value::from(50.0), "phase_deg": dokime::Value::from(15.0), "current_limit_a": dokime::Value::from(5.0), "duration_s": dokime::Value::from(0.002),
            "sample_interval_s": dokime::Value::from(0.001),
        })
        .to_string(),
    }
}

fn admit_steps<T: SidecarTemplate>(workflow: &Workflow<T>, token: &str, reply: &EvidenceBatch) {
    for encoded in &reply.step_begins {
        let start: dokime::domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
        workflow.admit(
            token,
            &StepAdmission {
                attempt_id: start.attempt_id.to_string(),
                accepted: true,
                message: String::new(),
            },
        );
    }
}

async fn drain_runtime<T: SidecarTemplate>(
    workflow: &Workflow<T>, token: &str, accept_prompt: Option<bool>, stop_after_sample: bool,
) -> SidecarExecutionOutcome {
    let mut cursor = 0;
    let mut stopped = false;
    let mut cleanup_logs = 0;
    loop {
        let reply = workflow.next(token, cursor).await.unwrap();
        admit_steps(workflow, token, &reply);
        cursor = reply.cursor;
        for record in &reply.records {
            assert_record_contract(serde_json::from_str(record).unwrap());
        }
        cleanup_logs += reply.diagnostics.iter().filter(|line| line.message.as_str() == "Charger resources released").count();
        let samples = workflow.take_local_samples(token).unwrap();
        // A measured sample, not a row read from the charger's log.
        let measured = samples.iter().any(|sample| sample.stream.as_str() != "obc.charger.startup.log");
        if stop_after_sample && !stopped && measured {
            workflow.cancel(token);
            stopped = true;
        }
        if let Some(accepted) = accept_prompt {
            for encoded in reply.prompt_requests {
                let prompt: PromptRequest = serde_json::from_str(&encoded).unwrap();
                answer(workflow, token, prompt.prompt_id, accepted);
            }
        }
        if let Some(outcome) = reply.outcome {
            assert_eq!(cleanup_logs, 1, "every initialized outcome must release its Charger exactly once");
            return outcome;
        }
    }
}

fn answer<T: SidecarTemplate>(workflow: &Workflow<T>, token: &str, prompt_id: PromptId, accepted: bool) {
    let response = PromptResponse {
        prompt_id: prompt_id.clone(),
        action: "submit".to_owned(),
        responder: "test".to_owned(),
        submitted_at: Timestamp::new("2026-01-01T00:00:00Z"),
        values: BTreeMap::from([("operator_note".to_owned(), "test checkpoint".into()), ("accepted".to_owned(), accepted.into())]),
        accepted: true,
    };
    workflow.answer(token, &prompt_id.to_string(), &serde_json::to_string(&response).unwrap()).unwrap();
}

/* === Tests === */
