/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod charger;
mod lifecycle;
mod operations;
mod plugin;
mod runs;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime::definition::{SuiteSet, materialize_template_cases};
use dokime::execution::{Answering, Caller};
use dokime::harmos::operations::OperationDeclaration;
use dokime::{ValidationMode, ValueType};
use plugin_obc::domain::model::readiness::Readiness;
use plugin_obc::operations::AskReadiness;
use plugin_obc::{ObcChecks, Pulse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::stream(
    id = "obc.system",
    window = 4096,
    label = "Testbench Health",
    description = "The host-side stream the pulse leases, declared by the plugin's system observer.\nThe observer's module is private to the crate, so the row the run leases is mirrored here under the same stream id and the same fields; the harness matches a measured stream by that id."
)]
struct SystemRow {
    #[row(unit = "%")]
    process_cpu_percent: f64,

    #[row(unit = "B")]
    process_memory_bytes: i64,

    #[row(unit = "%")]
    system_cpu_percent: f64,

    #[row(unit = "B")]
    system_memory_used_bytes: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn answering<O, F>(answer: F) -> (String, Answering)
where
    O: OperationDeclaration + dokime::harmos::Message,
    O::Response: dokime::harmos::Message,
    F: Fn(O) -> O::Response + Send + Sync + 'static,
{
    let answer = Arc::new(answer);
    (
        O::ID.to_owned(),
        Arc::new(move |payload: Vec<u8>| {
            let answer = Arc::clone(&answer);
            Box::pin(async move {
                let request = <O as dokime::harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
                Ok(dokime::harmos::Message::encode(&answer(request)))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }),
    )
}

// The options a host materializes one registration's cases with.
fn materialization() -> dokime::definition::CaseMaterializationOptions {
    dokime::definition::CaseMaterializationOptions {
        max_cases: 1_000,
        force: false,
        registration_id: dokime::model::identity::new_id(),
        first_label_number: 1,
    }
}

fn charger_answers() -> BTreeMap<String, Answering> {
    use plugin_obc::operations::charger::*;
    use plugin_obc::operations::wait::Wait;
    let devices = Arc::new(Chargers::default());
    let mut answers = BTreeMap::new();
    let owner = devices.clone();
    let (id, answer) = answering_result::<InitializeCharger, _>(move |request| owner.initialize(request.supply, request.logging));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<StartCharger, _>(move |request| owner.start(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ReadCharger, _>(move |request| owner.read(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SampleCharger, _>(move |request| owner.sample(request.device_id, request.elapsed_s));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SetChargerLoad, _>(move |request| owner.set_load(request.device_id, request.load_ohm));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<InjectChargerFault, _>(move |request| owner.inject_fault(request.device_id, request.fault));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ClearChargerFault, _>(move |request| owner.clear_fault(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ProbeChargerRestart, _>(move |request| owner.probe_restart(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<StopCharger, _>(move |request| owner.stop(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<ReleaseCharger, _>(move |request| owner.release(request.device_id));
    answers.insert(id, answer);
    let owner = devices.clone();
    let (id, answer) = answering_result::<SampleChargerBatch, _>(move |request| request.sample(&owner));
    answers.insert(id, answer);
    let (id, answer) = answering_result::<EncodeChargerSummary, _>(|request| request.encode().map_err(plugin_obc::Error::from));
    answers.insert(id, answer);
    answers.insert(
        Wait::ID.into(),
        Arc::new(|payload: Vec<u8>| {
            Box::pin(async move {
                let request = <Wait as dokime::harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
                tokio::time::sleep(std::time::Duration::from_secs_f64(request.duration_s)).await;
                Ok(dokime::harmos::Message::encode(&()))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }),
    );
    answers
}

fn answering_result<O, F>(answer: F) -> (String, Answering)
where
    O: OperationDeclaration + dokime::harmos::Message,
    O::Response: dokime::harmos::Message,
    F: Fn(O) -> Result<O::Response, plugin_obc::Error> + Send + Sync + 'static,
{
    let answer = Arc::new(answer);
    (
        O::ID.to_owned(),
        Arc::new(move |payload: Vec<u8>| {
            let answer = answer.clone();
            Box::pin(async move {
                let request = <O as dokime::harmos::Message>::decode(payload.as_slice()).map_err(|error| error.to_string())?;
                let response = answer(request).map_err(|error| error.to_string())?;
                Ok(dokime::harmos::Message::encode(&response))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>
        }),
    )
}

fn road() -> Caller {
    let mut answers = charger_answers();
    answers.extend([answering::<AskReadiness, _>(|_| Readiness { stopped: false })]);
    Caller::local(answers)
}

fn measured<C: dokime::telemetry::TelemetryStream>(rows: impl IntoIterator<Item = C::Row>) -> dokime::plugin::testing::Harness {
    dokime::plugin::testing::Harness::template("obc.checks.pulse").measuring::<C>(rows)
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn suite_export_bound() {
    // The plugin lists its suites on its own attribute; each of them answers
    // with its own registration.
    let suites = [ObcChecks::definition()];

    // The suite workflow is transport-neutral; the host binds it to the loaded
    // sidecar artifact when admitting an execution.
    assert_eq!(suites.len(), 1);
    assert_eq!(suites[0].suite_id, "obc.checks");
    assert_eq!(suites[0].workflow.as_deref(), Some("obc.checks"));
}

#[dokime_tracing::trace_test]
#[test]
fn contracts_bind_fields() {
    let registration = ObcChecks::definition();
    // The suite declares no telemetry of its own: this contract came off the
    // `#[dokime::observer]` that publishes on the stream, through the template that
    // declares it.
    let contract = &registration.telemetry_contracts[0];

    assert_eq!(contract.stream.as_str(), "obc.pulse");
    assert_eq!(contract.template_id, Some(Pulse::template_id()));
    assert_eq!(contract.validation_mode, Some(ValidationMode::Strict));
    assert!(!contract.allow_extra_fields);
    // `TelemetryContract::declared` contributed the leading `timestamp` column
    // and `PulseRow` contributed the rest, resolved from the field types rather
    // than from a second list an author would have to keep in agreement.
    assert_eq!(
        contract.columns.iter().map(|column| column.metadata.identifier.as_str()).collect::<Vec<_>>(),
        vec!["timestamp", "sample_index", "value"]
    );
    assert_eq!(
        contract.columns.iter().map(|column| column.value_type.clone()).collect::<Vec<_>>(),
        vec![ValueType::Datetime, ValueType::Integer, ValueType::Number]
    );
    assert_eq!(
        registration.record_contracts.iter().map(|contract| contract.record_type.as_str()).collect::<Vec<_>>(),
        vec!["obc.run.started", "obc.run.completed"]
    );
    assert!(registration.record_contracts.iter().all(|contract| {
        contract.template_id == Some(Pulse::template_id())
            && contract.validation_mode == Some(ValidationMode::Strict)
            && !contract.allow_extra_fields
    }));
}

#[dokime_tracing::trace_test]
#[test]
fn default_case_phases() {
    let template = ObcChecks::suite().test_case_templates.remove(0);
    let generated = materialize_template_cases("fixture.registration", &template, Default::default(), materialization()).unwrap();

    assert_eq!(generated.case_count, 1);
    assert_eq!(template.test_case_template_id, Pulse::template_id());
    assert_eq!(
        template.parameters.iter().map(|parameter| parameter.metadata.identifier.as_str()).collect::<Vec<_>>(),
        vec!["sample_count", "tolerance_au", "capture_system", "require_confirmation"]
    );
    // The sidecar names one of these phase ids in every drain reply.
    assert_eq!(
        template.phases.iter().map(|phase| phase.metadata.identifier.as_str()).collect::<Vec<_>>(),
        vec!["start", "stream", "complete"]
    );
    assert!(template.document.is_some(), "the template owns the document its live plots render in");
    // The criterion came off the method that decides it, not off a separate
    // `criteria()` list.
    assert_eq!(
        template.criteria.iter().map(|criterion| criterion.metadata.identifier.as_str()).collect::<Vec<_>>(),
        vec!["obc.checks.pulse.acceptance"]
    );
}

#[dokime_tracing::trace_test]
#[test]
fn suites_survive_trip() {
    let suites = SuiteSet::new(vec![ObcChecks::definition()]);
    let encoded = serde_json::to_string(&suites).expect("the suites serialize");
    let decoded: SuiteSet = serde_json::from_str(&encoded).expect("the suites deserialize");

    assert_eq!(decoded, suites);
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn pulse_leases_unmeasured() {
    let row = SystemRow {
        process_cpu_percent: 1.0,
        process_memory_bytes: 1_000_000,
        system_cpu_percent: 5.0,
        system_memory_used_bytes: 2_000_000,
    };
    let report = measured::<SystemRow>([row])
        .parameter("capture_system", true)
        .parameter("sample_count", 2)
        .run::<plugin_obc::suites::Suites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    assert!(!report.samples::<plugin_obc::PulseRow>().is_empty());

    let refused = dokime::plugin::testing::Harness::template("obc.checks.pulse")
        .parameter("capture_system", true)
        .parameter("sample_count", 2)
        .run::<plugin_obc::suites::Suites>()
        .await
        .unwrap();
    assert!(!refused.passed(), "an unmeasured global stream is refused");
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn pulse_confirmation() {
    use dokime::plugin::testing::Harness;
    for (action, passed) in [(None, true), (Some("yes"), true), (Some("no"), false), (Some("cancel"), false)] {
        let mut harness = Harness::template("obc.checks.pulse").parameter("sample_count", 2);
        if let Some(action) = action {
            harness = harness.parameter("require_confirmation", true).answer(
                "obc.checks.pulse.confirm",
                action,
                std::iter::empty::<(&str, dokime::Value)>(),
            );
        }
        let report = harness.run::<plugin_obc::suites::Suites>().await.unwrap();
        assert_eq!(report.passed(), passed, "action {action:?}: {}", report.diagnostics());
        let samples = report.samples::<plugin_obc::PulseRow>();
        if passed {
            assert!(samples.len() >= 2);
        } else {
            assert!(samples.is_empty(), "capture stays disarmed until confirmation succeeds");
        }
    }
}

#[dokime_tracing::trace_test]
#[tokio::test(start_paused = true)]
async fn cpu_load_example_uses_public_hold_and_control_subscription() {
    use plugin_host::host::cpu::load::{LoadControl, LoadHold, LoadSummary, LoadTarget};
    use std::sync::atomic::{AtomicUsize, Ordering};
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let row = LoadControl {
        mode: "percent".into(),
        target_percent: Some(35.0),
        measured_percent: Some(35.0),
        target_flops: None,
        measured_flops: None,
        duty_percent: 20.0,
        generator_flops: 1_000.0,
    };
    let report = dokime::plugin::testing::Harness::template("obc.checks.cpu_load")
        .parameter("target_percent", 35.0)
        .parameter("duration_s", 1.0)
        .measuring::<LoadControl>([row])
        .answering::<plugin_obc::operations::wait::Wait, _>(|_| ())
        .answering::<LoadHold, _>(move |request| {
            assert_eq!(request.target, LoadTarget::Percent(35.0));
            assert_eq!(request.threads, None);
            observed.fetch_add(1, Ordering::SeqCst);
            LoadSummary {
                target: request.target,
                mean: 35.0,
                error_rms: 0.0,
                duration_s: 1.0,
                samples: 4,
            }
        })
        .run::<plugin_obc::suites::Suites>()
        .await
        .unwrap();
    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
