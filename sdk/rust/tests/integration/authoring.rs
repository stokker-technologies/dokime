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

use harmos_runtime::plugin::grpc::Placement;
use prost::Message;

// Local Imports
use super::support::{
    TestResult,
    plugin::{self, Process},
    reference,
};
use dokime::Origin;
use dokime::PluginRun;
use dokime::Statistics;
use dokime::Suites;
use dokime::Termination;
use dokime::Value;
use dokime::origin;
use dokime::plugin::CriteriaEvaluateResponse;
use dokime::plugin::ExecutionsCleanupResponse;
use dokime::plugin::ExecutionsInitializeResponse;
use dokime::plugin::StepsExecuteResponse;
use dokime::plugin::TestCasesMaterializeResponse;
use dokime::value;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::evidence::CapturedRecord;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::LogsAppendResponse;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendResponse;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::host::executions::ProgressSetRequest;
use dokime_protocol::host::executions::ProgressSetResponse;
use dokime_protocol::plugin::CriteriaEvaluateRequest;
use dokime_protocol::plugin::ExecutionsCleanupRequest;
use dokime_protocol::plugin::ExecutionsInitializeRequest;
use dokime_protocol::plugin::StepsExecuteRequest;
use dokime_protocol::plugin::TestCasesMaterializeRequest;
use harmos_protocol::host::TimeSleepRequest;
use harmos_protocol::host::TimeSleepResponse;
use harmos_protocol::model::Visibility;
use harmos_protocol::plugin::Window;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Message)]
#[dokime(package = "thermal")]
struct TemperatureColumns {
    celsius: Vec<f64>,
}

#[derive(dokime::Record)]
#[dokime(package = "thermal")]
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
    // Start the SDK Sample as a Remote Plugin and Inspect Its Typed Declarations.
    let mut process = Process::start("dokime-rust-thermal", plugin::Placement::Remote).await?;
    let description = process.describe().await?;
    let declarations = description.declarations.as_ref().ok_or("Missing Declarations")?;
    let section = declarations.sections.iter().find(|section| section.r#type == "dokime.model.v1.Suites").ok_or("Missing Suites")?;
    let suites = Suites::decode(section.payload.as_slice())?;
    let template = &suites.suites[0].templates[0];
    assert_eq!(template.metadata.as_ref().ok_or("Missing Metadata")?.identifier, "thermal.thermal.cycle");
    let pool = prost_reflect::DescriptorPool::from_file_descriptor_set(declarations.types.clone().ok_or("Missing Types")?)?;
    assert!(pool.get_message_by_name("thermal.Door").is_some());
    assert_eq!(
        declarations
            .operations
            .iter()
            .find(|operation| operation.id == "thermal.thermal.cycle.ramp")
            .and_then(|operation| operation.policy.as_ref())
            .ok_or("Missing Retry Policy")?
            .retries
            .ok_or("Missing Retries")?
            .total_attempts,
        3
    );

    // Each Lifecycle Binding Retains Its Own Protected Identity and Plugin-Face Payloads.
    for (id, request, response) in [
        ("thermal.thermal.cycle.materialize", "TestCasesMaterializeRequest", "TestCasesMaterializeResponse"),
        ("thermal.thermal.cycle.initialize", "ExecutionsInitializeRequest", "ExecutionsInitializeResponse"),
        ("thermal.thermal.cycle.ramp", "StepsExecuteRequest", "StepsExecuteResponse"),
        ("thermal.thermal.cycle.hold", "StepsExecuteRequest", "StepsExecuteResponse"),
        ("thermal.thermal.cycle.within_limit", "CriteriaEvaluateRequest", "CriteriaEvaluateResponse"),
        ("thermal.thermal.cycle.cleanup", "ExecutionsCleanupRequest", "ExecutionsCleanupResponse"),
    ] {
        let operation = declarations.operations.iter().find(|operation| operation.id == id).ok_or("Missing Lifecycle Binding")?;
        assert_eq!(operation.visibility, Visibility::Protected as i32);
        assert_eq!(operation.request, format!("dokime.plugin.v1.{request}"));
        assert_eq!(operation.response, format!("dokime.plugin.v1.{response}"));
        assert!(pool.get_message_by_name(&operation.request).is_some());
        assert!(pool.get_message_by_name(&operation.response).is_some());
    }

    // Only the Step Authored With Retries Advertises Replay Safety.
    assert!(declarations.operations.iter().find(|operation| operation.id == "thermal.thermal.cycle.ramp").unwrap().replay_safe);
    assert!(!declarations.operations.iter().find(|operation| operation.id == "thermal.thermal.cycle.hold").unwrap().replay_safe);

    // Admit the Suite and Materialize It Through the Published Harmos Reference Host.
    let runtime = reference::start().await?;
    let attached = runtime.attach_grpc(Placement::Remote(process.address.clone()), reference::placement()).await?;
    let parameters = [("limit".into(), Value { kind: Some(value::Kind::Number(85.0)) })].into();
    let request = TestCasesMaterializeRequest {
        template: "thermal.thermal.cycle".into(),
        parameters,
    };
    let payload = runtime.operations.call_encoded(&template.materialize, request.encode_to_vec()).await?;
    let case = TestCasesMaterializeResponse::decode(payload.as_slice())?.test_case.ok_or("Missing Materialized Test Case")?;
    assert_eq!(
        case.steps.iter().map(|step| step.metadata.as_ref().unwrap().identifier.as_str()).collect::<Vec<_>>(),
        ["thermal.thermal.cycle.ramp", "thermal.thermal.cycle.hold"]
    );
    assert_eq!(case.streams, ["thermal.temperature", "thermal.counter"]);
    assert_eq!(case.criteria[0].metadata.as_ref().unwrap().identifier, "thermal.thermal.cycle.within_limit");
    attached.stop("Materialization Complete").await?;
    runtime.stop().await?;

    // Freeze Origin and Initialize an Execution with Explicit Harmos Scope.
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "thermal.thermal.cycle".into(),
            digest: declarations.digest.clone(),
            started_at: description.started_at,
        })),
    };
    let mut initialize = process
        .invoke(
            template.initialize.as_deref().ok_or("Missing Initialize")?,
            "execution_a",
            ExecutionsInitializeRequest {
                execution: "execution_a".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    assert_eq!(log.operation, "dokime.host.v1.evidence.Logs/Append");
    assert_eq!(LogsAppendRequest::decode(log.payload.as_slice())?.execution, "execution_a");
    initialize.reply(log.call, LogsAppendResponse::default()).await?;
    initialize.finish::<ExecutionsInitializeResponse>().await?;

    // Run the First Step and Observe Its Acknowledged Progress Call.
    let mut ramp = process
        .invoke(
            &case.steps[0].operation,
            "execution_a",
            StepsExecuteRequest {
                execution: "execution_a".into(),
                step: "thermal.thermal.cycle.ramp".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let get = ramp.child().await?;
    ramp.reply(
        get.call,
        ProgressGetResponse {
            revision: vec![0; 8],
            ..Default::default()
        },
    )
    .await?;
    let progress = ramp.child().await?;
    assert_eq!(ProgressSetRequest::decode(progress.payload.as_slice())?.value.unwrap().completed, 1);
    ramp.reply(progress.call, ProgressSetResponse::default()).await?;
    ramp.finish::<StepsExecuteResponse>().await?;

    // Answer the Operator Prompt and Acknowledge the Record Before the Step Continues.
    let mut hold = process
        .invoke(
            &case.steps[1].operation,
            "execution_a",
            StepsExecuteRequest {
                execution: "execution_a".into(),
                step: "thermal.thermal.cycle.hold".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let question = hold.child().await?;
    assert_eq!(question.operation, "dokime.host.v1.Prompts/Create");
    let prompt = PromptsCreateRequest::decode(question.payload.as_slice())?;
    hold.reply(
        question.call,
        PromptsCreateResponse {
            prompt: prompt.prompt,
            action: prompt.definition.unwrap().actions[0].identifier.clone(),
            accepted: true,
            ..Default::default()
        },
    )
    .await?;
    let record = hold.child().await?;
    assert_eq!(record.operation, "dokime.host.v1.evidence.Records/Append");
    let record = (record.call, RecordsAppendRequest::decode(record.payload.as_slice())?);
    assert_eq!(record.1.execution, "execution_a");
    assert!(Door::decode(record.1.payload.as_slice())?.closed);
    assert!(!record.1.request_id.is_empty());
    hold.reply(record.0, RecordsAppendResponse { ordinal: 1, receipt: None }).await?;

    // Keep the Step Waiting on the Host While Its Observer Produces Evidence.
    let sleep = hold.child().await?;
    assert_eq!(sleep.operation, "harmos.plugin.v1.Time/Sleep");
    assert_eq!(
        TimeSleepRequest::decode(sleep.payload.as_slice())?.duration,
        Some(prost_types::Duration { seconds: 2, nanos: 500_000_000 })
    );
    let (grants, mut samples) = process.publish("thermal.temperature", "execution_a").await?;
    let period = prost_types::Duration { seconds: 1, nanos: 0 };
    let at = prost_types::Timestamp { seconds: 100, nanos: 0 };
    let window = Window {
        start: Some(at),
        period: Some(period),
        count: 2,
        sequence: 1,
        capturing_executions: Vec::new(),
    };
    let columns = Process::window(&grants, &mut samples, window).await?;
    assert_eq!(TemperatureColumns::decode(columns.as_slice())?.celsius, [38.25, 49.6875]);

    // A Second Window Retains the Same Execution Resource and Continues the Sequence.
    let window = Window {
        start: Some(prost_types::Timestamp { seconds: 102, nanos: 0 }),
        period: Some(period),
        count: 1,
        sequence: 3,
        capturing_executions: Vec::new(),
    };
    let columns = Process::window(&grants, &mut samples, window).await?;
    assert_eq!(TemperatureColumns::decode(columns.as_slice())?.celsius, [58.265625]);
    drop(grants);
    assert!(samples.message().await?.is_none());

    // Finish the Wait by Protocol Event and Acknowledge the Step's Final Progress.
    hold.reply(sleep.call, TimeSleepResponse {}).await?;
    let get = hold.child().await?;
    hold.reply(
        get.call,
        ProgressGetResponse {
            revision: vec![0; 8],
            ..Default::default()
        },
    )
    .await?;
    let progress = hold.child().await?;
    assert_eq!(ProgressSetRequest::decode(progress.payload.as_slice())?.value.unwrap().completed, 2);
    hold.reply(progress.call, ProgressSetResponse::default()).await?;
    hold.finish::<StepsExecuteResponse>().await?;

    // Evaluate Captured Evidence Through Host Statistics and Bounded Record Reads.
    let mut criterion = process
        .invoke(
            &case.criteria[0].operation,
            "execution_a",
            CriteriaEvaluateRequest {
                execution: "execution_a".into(),
                criterion: "thermal.thermal.cycle.within_limit".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let stats = criterion.child().await?;
    let query = EvidenceStatisticsRequest::decode(stats.payload.as_slice())?;
    assert!(query.scope.is_none());
    assert_eq!(query.step, "thermal.thermal.cycle.hold");
    assert_eq!(query.field, "celsius");
    criterion
        .reply(
            stats.call,
            EvidenceStatisticsResponse {
                statistics: Some(Statistics {
                    count: 2,
                    max: Some(Value { kind: Some(value::Kind::Number(84.0)) }),
                    min: Some(Value { kind: Some(value::Kind::Number(84.0)) }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .await?;
    let records = criterion.child().await?;
    assert_eq!(RecordsListRequest::decode(records.payload.as_slice())?.record_type, "thermal.door");
    criterion
        .reply(
            records.call,
            RecordsListResponse {
                count: 1,
                last: Some(CapturedRecord {
                    record_type: "thermal.door".into(),
                    payload: record.1.payload,

                    timestamp: Some(at),
                    ..Default::default()
                }),
                records: Vec::new(),
                ..Default::default()
            },
        )
        .await?;
    assert!(criterion.finish::<CriteriaEvaluateResponse>().await?.passed);

    // Cleanup Completes Before the Execution Store Is Released.
    let mut cleanup = process
        .invoke(
            template.cleanup.as_deref().ok_or("Missing Cleanup")?,
            "execution_a",
            ExecutionsCleanupRequest {
                execution: "execution_a".into(),
                termination: Termination::Completed as i32,
                test_case: Some(case),
                origin: Some(origin),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, LogsAppendResponse::default()).await?;
    cleanup.finish::<ExecutionsCleanupResponse>().await?;
    process.stop().await?;
    Ok(())
}

/* === Tests === */

#[tokio::test]
async fn plain_operation_values_roundtrip_over_the_plugin_carrier() -> TestResult {
    let mut process = Process::start("dokime-rust-plugin", plugin::Placement::Remote).await?;
    let declarations = process.describe().await?.declarations.ok_or("Missing Declarations")?;
    let operation = declarations.operations.iter().find(|operation| operation.id == "example.ping").ok_or("Missing Echo")?;
    assert_eq!(
        (&operation.request, &operation.response),
        (&"google.protobuf.StringValue".to_owned(), &"google.protobuf.StringValue".to_owned())
    );
    let invocation = process.invoke("example.ping", "", String::from("plain value")).await?;
    assert_eq!(invocation.finish::<String>().await?, "plain value");
    process.stop().await?;
    Ok(())
}
