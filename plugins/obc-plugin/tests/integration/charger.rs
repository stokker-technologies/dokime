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
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime::execution::Workflow;
use dokime::transport::{RunExecutionRequest, StepAdmission};
use obc_plugin::{SOURCE, suites::steady};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
// Observer callbacks run on independent runtime threads, so the story keeps
// the real clock: a paused one would spin the drain poll while they run.

#[tokio::test]
async fn cleanup_drains_input() {
    let workflow = Workflow::<steady::Steady>::new(steady::STEADY_SUITE, SOURCE);
    workflow
        .start_on(
            RunExecutionRequest {
                mode: "automatic".into(),
                execution_token: "finalize".into(),
                test_cycle_id: "cycle".into(),
                test_execution_id: "execution-finalize".into(),
                template_identifier: steady::STEADY_TEMPLATE.into(),
                workflow: steady::STEADY_SUITE.into(),
                secrets_json: "{}".into(),
                parameters_json: serde_json::json!({
                    "ac_voltage_v": dokime::Value::from(230.0),
                    "load_ohm": dokime::Value::from(100.0),
                    "frequency_hz": dokime::Value::from(50.0),
                    "phase_deg": dokime::Value::from(15.0),
                    "current_limit_a": dokime::Value::from(5.0),
                    "duration_s": dokime::Value::from(0.002),
                    "sample_interval_s": dokime::Value::from(0.001),
                })
                .to_string(),
            },
            None,
            super::road(),
        )
        .unwrap();

    let mut cursor = 0;
    loop {
        let reply = workflow.drain("finalize", cursor).unwrap();
        for encoded in &reply.step_begins {
            let start: dokime::execution::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            workflow.admit(
                "finalize",
                &StepAdmission {
                    attempt_id: start.attempt_id.to_string(),
                    accepted: true,
                    message: String::new(),
                },
            );
        }
        cursor = reply.cursor;
        if !reply.prompt_requests.is_empty() {
            break;
        }
        // TODO(source-hook): an awaitable Workflow drain or change notification.
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    // The SDK cancels the waiting execution and joins its sampler before
    // stopping the app; a hang here is caught by the lane deadline.
    workflow.shutdown().await;
}
