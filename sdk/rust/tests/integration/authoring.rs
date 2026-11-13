/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serve and Execute a Thermal Suite

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use harmos_runtime::plugin::grpc::Placement;
use prost::Message as _;

// Local Imports
use super::support::{TestResult, plugin::Process, reference};
use dokime::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "thermal.v1")]
struct Temperature {
    celsius: f64,
}

#[derive(dokime::Record)]
#[harmos(package = "thermal.v1")]
struct Door {
    closed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn author_serve_materialize_and_execute_a_thermal_suite() -> TestResult {
    // Start the SDK Sample as a Remote Plugin and Inspect Its Typed Catalog.
    let mut process = Process::start("dokime-rust-thermal", true).await?;
    let description = process.describe().await?;
    let catalog = description.catalog.as_ref().ok_or("Missing Catalog")?;
    let section = catalog.sections.iter().find(|section| section.r#type == "dokime.v1.Suites").ok_or("Missing Suites")?;
    let suites = v1::Suites::decode(section.payload.as_slice())?;
    let template = &suites.suites[0].templates[0];
    assert_eq!(template.metadata.as_ref().ok_or("Missing Metadata")?.identifier, "thermal.cycle");
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(catalog.types.clone().ok_or("Missing Types")?)?;
    assert!(pool.get_message_by_name("thermal.v1.Door").is_some());
    assert_eq!(
        catalog
            .operations
            .iter()
            .find(|operation| operation.id == "thermal.cycle.step.0")
            .and_then(|operation| operation.policy.as_ref())
            .ok_or("Missing Retry Policy")?
            .retries
            .ok_or("Missing Retries")?
            .total_attempts,
        3
    );

    // Admit the Suite and Materialize It Through the Published Harmos Reference Host.
    let runtime = reference::start().await?;
    let attached = runtime.attach_grpc(Placement::Remote(process.address.clone()), reference::placement()).await?;
    let parameters = [(
        "limit".into(),
        v1::Value {
            kind: Some(v1::value::Kind::Number(85.0)),
        },
    )]
    .into();
    let request = v1::MaterializeRequest {
        template: "thermal.cycle".into(),
        parameters,
    };
    let payload = runtime.operations.call_encoded(&template.materialize, request.encode_to_vec()).await?;
    let case = v1::TestCase::decode(payload.as_slice())?;
    assert_eq!(case.steps.iter().map(|step| step.metadata.as_ref().unwrap().identifier.as_str()).collect::<Vec<_>>(), ["ramp", "hold"]);
    assert_eq!(case.streams, ["thermal.temperature"]);
    assert_eq!(case.criteria[0].metadata.as_ref().unwrap().identifier, "within_limit");
    attached.stop("Materialization Complete").await?;
    runtime.shutdown().await?;

    // Freeze Origin and Initialize an Execution with Explicit Harmos Scope.
    let origin = v1::Origin {
        kind: Some(v1::origin::Kind::PluginRun(v1::PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "thermal.cycle".into(),
            digest: catalog.digest.clone(),
            started_at: description.started_at,
        })),
    };
    let mut initialize = process
        .invoke(
            template.initialize.as_deref().ok_or("Missing Initialize")?,
            "execution_a",
            v1::InitializeRequest {
                execution: "execution_a".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    assert_eq!(log.operation, "dokime.execution.log");
    assert_eq!(v1::WriteLog::decode(log.payload.as_slice())?.execution, "execution_a");
    initialize.reply(log.call, v1::WriteLogResponse {}).await?;
    initialize.finish::<v1::InitializeResponse>().await?;

    // Run the First Step and Observe Its Acknowledged Progress Call.
    let mut ramp = process
        .invoke(
            &case.steps[0].operation,
            "execution_a",
            v1::StepRequest {
                execution: "execution_a".into(),
                step: "ramp".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let progress = ramp.child().await?;
    assert_eq!(v1::ReportProgress::decode(progress.payload.as_slice())?.completed, 1);
    ramp.reply(progress.call, v1::ReportProgressResponse {}).await?;
    ramp.finish::<v1::StepResponse>().await?;

    // Answer the Operator Prompt and Acknowledge the Record Before the Step Continues.
    let mut hold = process
        .invoke(
            &case.steps[1].operation,
            "execution_a",
            v1::StepRequest {
                execution: "execution_a".into(),
                step: "hold".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let question = hold.child().await?;
    assert_eq!(question.operation, "dokime.execution.prompt");
    assert_eq!(v1::OperatorQuestion::decode(question.payload.as_slice())?.prompt_type, "thermal.door");
    hold.reply(
        question.call,
        v1::OperatorAnswer {
            prompt: "door".into(),
            action: "yes".into(),
            accepted: true,
            ..Default::default()
        },
    )
    .await?;
    let record = hold.child().await?;
    assert_eq!(record.operation, "dokime.execution.record");
    let record = (record.call, v1::PublishRecord::decode(record.payload.as_slice())?);
    assert_eq!(record.1.execution, "execution_a");
    assert!(Door::decode(record.1.payload.as_slice())?.closed);
    hold.reply(record.0, v1::RecordReceipt { ordinal: 1 }).await?;

    // Keep the Step Waiting on the Host While Its Observer Produces Evidence.
    let sleep = hold.child().await?;
    assert_eq!(sleep.operation, "harmos.context.v1.Time/Sleep");
    assert_eq!(
        harmos_protocol::context::TimeSleepRequest::decode(sleep.payload.as_slice())?.duration,
        Some(prost_types::Duration { seconds: 2, nanos: 500_000_000 })
    );
    let (ticks, mut samples) = process.publish("thermal.temperature", "execution_a").await?;
    let at = prost_types::Timestamp { seconds: 100, nanos: 0 };
    ticks
        .send(harmos_protocol::plugin::InstanceSamplesRequest {
            frame: Some(harmos_protocol::plugin::instance_samples_request::Frame::Tick(h::Tick { at: Some(at) })),
        })
        .await?;
    let sample = samples.message().await?.ok_or("Missing Sample")?.samples.remove(0);
    assert_eq!(sample.at, Some(at));
    assert_eq!(sample.sequence, 1);
    assert_eq!(Temperature::decode(sample.row.as_slice())?.celsius, 84.0);

    // A Second Host Tick Retains the Same Execution Resource and Ordered Sequence.
    let next = prost_types::Timestamp { seconds: 101, nanos: 0 };
    ticks
        .send(harmos_protocol::plugin::InstanceSamplesRequest {
            frame: Some(harmos_protocol::plugin::instance_samples_request::Frame::Tick(h::Tick { at: Some(next) })),
        })
        .await?;
    let sample = samples.message().await?.ok_or("Missing Second Sample")?.samples.remove(0);
    assert_eq!(sample.at, Some(next));
    assert_eq!(sample.sequence, 2);
    assert_eq!(Temperature::decode(sample.row.as_slice())?.celsius, 84.0);
    drop(ticks);
    assert!(samples.message().await?.is_none());

    // Finish the Wait by Protocol Event and Acknowledge the Step's Final Progress.
    hold.reply(sleep.call, harmos_protocol::context::TimeSleepResponse {}).await?;
    let progress = hold.child().await?;
    assert_eq!(v1::ReportProgress::decode(progress.payload.as_slice())?.completed, 2);
    hold.reply(progress.call, v1::ReportProgressResponse {}).await?;
    hold.finish::<v1::StepResponse>().await?;

    // Evaluate Captured Evidence Through Host Statistics and Bounded Record Reads.
    let mut criterion = process
        .invoke(
            &case.criteria[0].operation,
            "execution_a",
            v1::CriterionRequest {
                execution: "execution_a".into(),
                criterion: "within_limit".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let stats = criterion.child().await?;
    let query = v1::StatisticsRequest::decode(stats.payload.as_slice())?;
    assert_eq!(query.execution, "execution_a");
    assert_eq!(query.step, "hold");
    assert_eq!(query.field, "celsius");
    criterion
        .reply(
            stats.call,
            v1::Statistics {
                count: 2,
                max: 84.0,
                min: 84.0,
                ..Default::default()
            },
        )
        .await?;
    let records = criterion.child().await?;
    assert_eq!(v1::RecordsRequest::decode(records.payload.as_slice())?.limit, 0);
    criterion
        .reply(
            records.call,
            v1::Records {
                count: 1,
                last: Some(v1::Record {
                    kind: "thermal.door".into(),
                    payload: record.1.payload,
                    ordinal: 1,
                    at: Some(at),
                }),
                records: Vec::new(),
            },
        )
        .await?;
    assert!(criterion.finish::<v1::CriterionResponse>().await?.passed);

    // Cleanup Completes Before the Execution Store Is Released.
    let mut cleanup = process
        .invoke(
            template.cleanup.as_deref().ok_or("Missing Cleanup")?,
            "execution_a",
            v1::CleanupRequest {
                execution: "execution_a".into(),
                termination: v1::Termination::Completed as i32,
                test_case: Some(case),
                origin: Some(origin),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, v1::WriteLogResponse {}).await?;
    cleanup.finish::<v1::CleanupResponse>().await?;
    process.stop(true).await?;
    Ok(())
}

/* === Tests === */
