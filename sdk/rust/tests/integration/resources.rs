/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Isolated Execution Resources

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use super::support::{
    TestResult,
    plugin::{Placement, Process},
};
use dokime::Origin;
use dokime::PluginRun;
use dokime::Termination;
use dokime::Value;
use dokime::origin;
use dokime::plugin::ExecutionsCleanupResponse;
use dokime::plugin::ExecutionsInitializeResponse;
use dokime::plugin::StepsExecuteResponse;
use dokime::plugin::TestCasesMaterializeResponse;
use dokime::value;
use dokime_protocol::host::evidence::LogsAppendResponse;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::host::executions::ProgressSetRequest;
use dokime_protocol::host::executions::ProgressSetResponse;
use dokime_protocol::plugin::ExecutionsCleanupRequest;
use dokime_protocol::plugin::ExecutionsInitializeRequest;
use dokime_protocol::plugin::StepsExecuteRequest;
use dokime_protocol::plugin::TestCasesMaterializeRequest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn keep_execution_resources_isolated_and_release_them_after_cleanup() -> TestResult {
    // Start a Managed Plugin and Preview Its Case Without an Execution.
    let mut process = Process::start("dokime-rust-thermal", Placement::Managed).await?;
    let description = process.describe().await?;
    let request = TestCasesMaterializeRequest {
        template: "thermal.thermal.cycle".into(),
        parameters: [("limit".into(), Value { kind: Some(value::Kind::Number(70.0)) })].into(),
    };
    let case = process
        .invoke("thermal.thermal.cycle.materialize", "", request.clone())
        .await?
        .finish::<TestCasesMaterializeResponse>()
        .await?
        .test_case
        .ok_or("Missing Materialized Test Case")?;
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "thermal.thermal.cycle".into(),
            digest: description.declarations.ok_or("Missing Declarations")?.digest,
            started_at: description.started_at,
        })),
    };

    // Materialization Cannot Allocate a Bench Resource for a Later Execution.
    let step = StepsExecuteRequest {
        execution: "a".into(),
        step: "thermal.thermal.cycle.ramp".into(),
        test_case: Some(case.clone()),
        origin: Some(origin.clone()),
    };
    let failure = process.invoke(&case.steps[0].operation, "a", step.clone()).await?.failure().await?;
    assert_eq!(failure.classification, harmos_protocol::model::Classification::Permanent as i32);

    // Reject a Mismatched Scope Before Entering the Step.
    let failure = process.invoke(&case.steps[0].operation, "b", step.clone()).await?.failure().await?;
    assert_eq!(failure.classification, harmos_protocol::model::Classification::Permanent as i32);

    // Initialize Two Interleaved Executions with Independent Stores.
    let mut initialize = process
        .invoke(
            "thermal.thermal.cycle.initialize",
            "a",
            ExecutionsInitializeRequest {
                execution: "a".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    initialize.reply(log.call, LogsAppendResponse::default()).await?;
    initialize.finish::<ExecutionsInitializeResponse>().await?;
    let mut initialize = process
        .invoke(
            "thermal.thermal.cycle.initialize",
            "b",
            ExecutionsInitializeRequest {
                execution: "b".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    initialize.reply(log.call, LogsAppendResponse::default()).await?;
    initialize.finish::<ExecutionsInitializeResponse>().await?;

    // Reject a Changed Definition on an Already Initialized Execution.
    let mut changed = case.clone();
    changed.parameters.insert("limit".into(), Value { kind: Some(value::Kind::Number(90.0)) });
    let mut changed_origin = origin.clone();
    let Some(origin::Kind::PluginRun(run)) = changed_origin.kind.as_mut() else {
        return Err("Missing Run Origin".into());
    };
    run.test_case = Some(changed.clone());
    let changed = StepsExecuteRequest {
        test_case: Some(changed),
        origin: Some(changed_origin),
        ..step.clone()
    };
    assert_eq!(
        process.invoke(&case.steps[0].operation, "a", changed).await?.failure().await?.classification,
        harmos_protocol::model::Classification::Permanent as i32
    );

    // Clean Up One Execution While the Other Remains Ready.
    let mut cleanup = process
        .invoke(
            "thermal.thermal.cycle.cleanup",
            "a",
            ExecutionsCleanupRequest {
                execution: "a".into(),
                termination: Termination::Cancelled as i32,
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, LogsAppendResponse::default()).await?;
    cleanup.finish::<ExecutionsCleanupResponse>().await?;
    assert_eq!(
        process.invoke(&case.steps[0].operation, "a", step.clone()).await?.failure().await?.classification,
        harmos_protocol::model::Classification::Permanent as i32
    );
    let mut ramp = process.invoke(&case.steps[0].operation, "b", StepsExecuteRequest { execution: "b".into(), ..step }).await?;
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
    assert!(ProgressSetRequest::decode(progress.payload.as_slice())?.name.is_none());
    ramp.reply(progress.call, ProgressSetResponse::default()).await?;
    ramp.finish::<StepsExecuteResponse>().await?;

    // Cleanup May Be the First Operation, Including on a Case Never Initialized.
    let mut cleanup = process
        .invoke(
            "thermal.thermal.cycle.cleanup",
            "unstarted",
            ExecutionsCleanupRequest {
                execution: "unstarted".into(),
                termination: Termination::Interrupted as i32,
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, LogsAppendResponse::default()).await?;
    cleanup.finish::<ExecutionsCleanupResponse>().await?;

    // Finish the Remaining Execution and Verify Managed Stop Exits Normally.
    let mut cleanup = process
        .invoke(
            "thermal.thermal.cycle.cleanup",
            "b",
            ExecutionsCleanupRequest {
                execution: "b".into(),
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
