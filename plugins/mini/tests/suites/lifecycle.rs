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
use std::time::{Duration, Instant};

// Third Party Imports

// Local Imports
use dokime::core::Value;
use dokime::execution::Workflow;
use dokime::records::RecordPublishRequest;
use dokime::transport::{
    EvidenceBatch, RunExecutionRequest, SidecarCriterionStatus, SidecarExecutionOutcome,
    StepAdmission,
};
use mini_sidecar::{Pulse, PulseRow, SOURCE, pulse_value, suites};

/* === Constants === */
const POLL: Duration = Duration::from_millis(5);
const DEADLINE: Duration = Duration::from_secs(20);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn start_request(token: &str, sample_count: u64) -> RunExecutionRequest {
    let template = <Pulse as dokime::definition::DeclaredTemplate>::declaration();
    let cases = dokime::definition::materialize_template_cases(
        "fixture.registration",
        &template,
        dokime::definition::CaseMaterializationSelection {
            case_overrides: std::collections::BTreeMap::from([(
                "sample_count".to_owned(),
                vec![Value::from(i64::try_from(sample_count).unwrap())],
            )]),
            ..Default::default()
        },
        Default::default(),
    )
    .expect("the host resolves declared inputs before starting the sidecar");
    RunExecutionRequest {
        mode: "automatic".into(),
        execution_token: token.to_owned(),
        test_cycle_id: dokime::core::TestCycleId::from_seed("mini.fixture.cycle").to_string(),
        test_execution_id: dokime::core::TestExecutionId::from_seed(token).to_string(),
        template_identifier: template.metadata.identifier.to_string(),
        workflow: suites::checks::ID.to_owned(),

        secrets_json: "{}".to_owned(),
        parameters_json: serde_json::to_string(&cases.test_cases[0].parameters).unwrap(),
    }
}

fn workflow() -> Workflow<Pulse> {
    Workflow::new(suites::checks::ID, SOURCE)
}

fn drain(workflow: &Workflow<Pulse>, token: &str, cursor: u64) -> EvidenceBatch {
    let reply = workflow
        .drain(token, cursor)
        .expect("the execution remains drainable");
    for encoded in &reply.step_begins {
        let start: dokime::execution::StepAttemptStart = serde_json::from_str(encoded).unwrap();
        workflow.admit(
            token,
            &StepAdmission {
                attempt_id: start.attempt_id.to_string(),
                accepted: true,
                message: String::new(),
            },
        );
    }
    reply
}

async fn drain_until_terminal(
    workflow: &Workflow<Pulse>,
    token: &str,
    mut cursor: u64,
) -> Vec<EvidenceBatch> {
    let deadline = Instant::now() + DEADLINE;
    let mut replies = Vec::new();
    loop {
        let reply = drain(workflow, token, cursor);
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
        tokio::time::sleep(POLL).await;
    }
}

fn outcome(replies: &[EvidenceBatch]) -> &SidecarExecutionOutcome {
    replies
        .iter()
        .find_map(|reply| reply.outcome.as_ref())
        .expect("the drain stream terminates once")
}

fn records(replies: &[EvidenceBatch]) -> Vec<RecordPublishRequest> {
    replies
        .iter()
        .flat_map(|reply| reply.records.iter())
        .map(|record| serde_json::from_str(record).expect("every event decodes"))
        .collect()
}

/* === Tests === */

pub(crate) async fn execution_streams_outcome() {
    let workflow = workflow();
    let request = start_request("streaming", 3);
    workflow
        .start(request.clone())
        .expect("the execution starts");
    workflow
        .start(request)
        .expect("an identical start is idempotent");

    let replies = drain_until_terminal(&workflow, "streaming", 0).await;
    let samples = workflow.take_local_samples("streaming").unwrap();
    let records = records(&replies);
    let outcome = outcome(&replies);

    // The declared stream sampled until the step had what it waited for.
    assert!(samples.len() >= 3);
    assert!(samples.iter().all(|sample| {
        sample
            .scope
            .as_ref()
            .is_some_and(|scope| scope.test_execution_id.is_some())
    }));
    assert!(
        samples
            .iter()
            .all(|sample| { sample.stream.as_str() == <PulseRow as dokime::StreamSpec>::ID })
    );
    // The values are the sidecar's own deterministic function of the index,
    // compared with a tolerance because they round-trip through JSON.
    for (index, sample) in samples.iter().enumerate() {
        assert_eq!(
            sample.values.get("sample_index"),
            Some(&Value::from(index as i64))
        );
        let Some(Value::Number(value)) = sample.values.get("value") else {
            panic!("every sample carries a numeric value");
        };
        assert!((value - pulse_value(index as u64)).abs() < 1e-9);
        // The canonical column the stream's contract requires is filled in
        // by the engine when the row itself leaves it out.
        assert!(sample.values.contains_key("timestamp"));
    }
    assert_eq!(
        records
            .iter()
            .map(|record| record.record_type.as_str())
            .collect::<Vec<_>>(),
        vec![
            <suites::checks::pulse::RunStarted as dokime::records::DeclaredRecord>::RECORD_TYPE,
            <suites::checks::pulse::RunCompleted as dokime::records::DeclaredRecord>::RECORD_TYPE,
        ]
    );
    assert!(outcome.passed);
    let criterion = &outcome.criteria[0];
    assert_eq!(criterion.identifier, suites::checks::pulse::ACCEPTANCE);
    assert_eq!(
        SidecarCriterionStatus::try_from(criterion.status),
        Ok(SidecarCriterionStatus::Passed)
    );
    workflow.shutdown().await;
}

pub(crate) async fn repeated_cursor_output() {
    let workflow = workflow();
    workflow
        .start(start_request("replay", 3))
        .expect("the execution starts");
    // Replay is asserted after the run is over, so the buffer this reads
    // twice is a buffer nothing is still writing to.
    let finished = drain_until_terminal(&workflow, "replay", 0).await;
    let acknowledged = finished.last().expect("the run produced output").cursor;
    let before_terminal = finished.iter().rev().nth(1).map_or(0, |reply| reply.cursor);

    let first = drain(&workflow, "replay", before_terminal);
    let replayed = drain(&workflow, "replay", before_terminal);
    let empty = drain(&workflow, "replay", acknowledged);

    // A host that never saw its reply asks again from the same cursor and
    // gets the same answer — the terminal outcome included.
    assert_eq!(replayed.cursor, first.cursor);
    assert_eq!(replayed.phase, first.phase);
    assert_eq!(replayed.records, first.records);
    assert_eq!(replayed.outcome, first.outcome);
    assert!(first.outcome.is_some());
    assert_eq!(first.phase, "complete");
    // Once the host has acknowledged everything, the poll says so by
    // leaving the cursor where it was and the phase empty.
    assert_eq!(empty.cursor, acknowledged);
    assert!(empty.phase.is_empty());
    assert!(empty.records.is_empty());
    assert!(empty.outcome.is_none());
    workflow.shutdown().await;
}

pub(crate) async fn cancellation_measures_criterion() {
    let workflow = workflow();
    workflow
        .start(start_request("cancelled", 64))
        .expect("the execution is admitted");
    // Admission precedes initialization. Advance through phase boundaries
    // until execution starts, retaining the evidence for the final assertions.
    let mut cursor = 0;
    let mut replies = Vec::new();
    tokio::time::timeout(DEADLINE, async {
        loop {
            let reply = drain(&workflow, "cancelled", cursor);
            cursor = reply.cursor;
            assert!(
                reply.outcome.is_none(),
                "the execution ended before cancellation"
            );
            let started = records(std::slice::from_ref(&reply))
                .iter()
                .any(|record| record.record_type.as_str() == "mini.run.started");
            replies.push(reply);
            if started {
                break;
            }
            tokio::time::sleep(POLL).await;
        }
    })
    .await
    .expect("the execution starts before cancellation");
    // Cancelling is idempotent: the second request is a successful no-op.
    workflow.cancel("cancelled");
    workflow.cancel("cancelled");

    replies.extend(drain_until_terminal(&workflow, "cancelled", cursor).await);
    let outcome = outcome(&replies);

    assert!(!outcome.passed);
    assert!(
        outcome
            .message
            .as_deref()
            .is_some_and(|message| message.contains("cancelled"))
    );
    // A cancelled run measured less, so its criterion decided over less.
    // That is a smaller verdict, not an absent one.
    assert_eq!(
        SidecarCriterionStatus::try_from(outcome.criteria[0].status),
        Ok(SidecarCriterionStatus::Failed)
    );
    workflow.shutdown().await;
}

pub(crate) async fn unknown_workflow_starts() {
    let workflow = workflow();
    let mut request = start_request("wrong", 1);
    request.workflow = "someone.elses.suite".to_owned();

    let refusal = workflow
        .start(request)
        .expect_err("an unknown workflow is refused");

    assert!(refusal.to_string().contains("someone.elses.suite"));
    workflow.shutdown().await;
}
