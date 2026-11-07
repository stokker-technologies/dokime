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
use dokime::execution::Workflow;
use dokime::transport::{RunExecution, StepAdmission};
use plugin_obc::{SOURCE, suites::steady};

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn cleanup_drains_input() {
    let workflow = Workflow::<steady::Steady>::new(steady::STEADY_SUITE, SOURCE);
    workflow
        .start_on(
            RunExecution {
                publication_rows: 1024,
                publication_bytes: 1024 * 1024,
                publication_delay_micros: 0,
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
        let reply = workflow.next("finalize", cursor).await.unwrap();
        for encoded in &reply.step_begins {
            let start: dokime::domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
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
    }

    // The SDK cancels the waiting execution and joins its sampler before
    // stopping the app; a hang here is caught by the lane deadline.
    workflow.shutdown().await;
}

#[test]
fn operation_resources_isolate_devices_and_release_owned_logs() {
    use plugin_obc::domain::model::charger::{Fault, Power, Supply};
    use plugin_obc::operations::charger::{Chargers, SampleChargerBatch};
    let devices = Chargers::default();
    let supply = Supply {
        voltage_rms_v: 230.0,
        load_ohm: 100.0,
        frequency_hz: 50.0,
        phase_deg: 15.0,
        current_limit_a: 5.0,
    };
    let first = devices.initialize(supply, true).unwrap();
    let second = devices.initialize(Supply { voltage_rms_v: 120.0, ..supply }, false).unwrap();
    assert_ne!(first.device_id, second.device_id);
    devices.start(first.device_id).unwrap();
    devices.start(second.device_id).unwrap();
    devices.inject_fault(first.device_id, Fault::PowerLoss).unwrap();
    assert!(devices.probe_restart(first.device_id).unwrap().blocked);
    let operation = SampleChargerBatch {
        device_id: second.device_id,
        first_index: 3,
        count: 4,
        interval_s: 0.01,
    };
    let samples = operation.sample(&devices).unwrap();
    assert_eq!(samples.rows.len(), 4);
    assert!(samples.rows.iter().all(|sample| sample.reading.current_rms_a > 0.0 && sample.charger.power == Power::Running));
    assert_eq!(devices.read(first.device_id).unwrap().fault, Some(Fault::PowerLoss));
    let log = std::path::PathBuf::from(first.log.unwrap());
    devices.sample(first.device_id, 0.1).unwrap();
    assert!(std::fs::metadata(&log).unwrap().len() > 0);
    devices.release(first.device_id).unwrap();
    assert!(!log.exists());
    assert!(devices.read(first.device_id).is_err());
    assert_eq!(devices.read(second.device_id).unwrap().power, Power::Running);
    devices.release(second.device_id).unwrap();
}
