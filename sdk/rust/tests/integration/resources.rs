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
use prost::Message as _;

// Local Imports
use super::support::{TestResult, plugin::Process};
use dokime::v1;

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
    let mut process = Process::start("dokime-rust-thermal", false).await?;
    let description = process.describe().await?;
    let request = v1::MaterializeRequest {
        template: "thermal.cycle".into(),
        parameters: [(
            "limit".into(),
            v1::Value {
                kind: Some(v1::value::Kind::Number(70.0)),
            },
        )]
        .into(),
    };
    let case = process.invoke("thermal.cycle.materialize", "", request.clone()).await?.finish::<v1::TestCase>().await?;
    let origin = v1::Origin {
        kind: Some(v1::origin::Kind::PluginRun(v1::PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "thermal.cycle".into(),
            digest: description.catalog.ok_or("Missing Catalog")?.digest,
            started_at: description.started_at,
        })),
    };

    // Materialization Cannot Allocate a Bench Resource for a Later Execution.
    let step = v1::StepRequest {
        execution: "a".into(),
        step: "ramp".into(),
        test_case: Some(case.clone()),
        origin: Some(origin.clone()),
    };
    let failure = process.invoke(&case.steps[0].operation, "a", step.clone()).await?.failure().await?;
    assert_eq!(failure.classification, harmos_protocol::v1::Classification::Permanent as i32);

    // Reject a Mismatched Scope Before Entering the Step.
    let failure = process.invoke(&case.steps[0].operation, "b", step.clone()).await?.failure().await?;
    assert_eq!(failure.classification, harmos_protocol::v1::Classification::Permanent as i32);

    // Initialize Two Interleaved Executions with Independent Stores.
    let mut initialize = process
        .invoke(
            "thermal.cycle.initialize",
            "a",
            v1::InitializeRequest {
                execution: "a".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    initialize.reply(log.call, dokime::protocol::context::v1::evidence::LogsAppendResponse {}).await?;
    initialize.finish::<v1::InitializeResponse>().await?;
    let mut initialize = process
        .invoke(
            "thermal.cycle.initialize",
            "b",
            v1::InitializeRequest {
                execution: "b".into(),
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = initialize.child().await?;
    initialize.reply(log.call, dokime::protocol::context::v1::evidence::LogsAppendResponse {}).await?;
    initialize.finish::<v1::InitializeResponse>().await?;

    // Reject a Changed Definition on an Already Initialized Execution.
    let mut changed = case.clone();
    changed.parameters.insert(
        "limit".into(),
        v1::Value {
            kind: Some(v1::value::Kind::Number(90.0)),
        },
    );
    let mut changed_origin = origin.clone();
    let Some(v1::origin::Kind::PluginRun(run)) = changed_origin.kind.as_mut() else {
        return Err("Missing Run Origin".into());
    };
    run.test_case = Some(changed.clone());
    let changed = v1::StepRequest {
        test_case: Some(changed),
        origin: Some(changed_origin),
        ..step.clone()
    };
    assert_eq!(
        process.invoke(&case.steps[0].operation, "a", changed).await?.failure().await?.classification,
        harmos_protocol::v1::Classification::Permanent as i32
    );

    // Clean Up One Execution While the Other Remains Ready.
    let mut cleanup = process
        .invoke(
            "thermal.cycle.cleanup",
            "a",
            v1::CleanupRequest {
                execution: "a".into(),
                termination: v1::Termination::Cancelled as i32,
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, dokime::protocol::context::v1::evidence::LogsAppendResponse {}).await?;
    cleanup.finish::<v1::CleanupResponse>().await?;
    assert_eq!(
        process.invoke(&case.steps[0].operation, "a", step.clone()).await?.failure().await?.classification,
        harmos_protocol::v1::Classification::Permanent as i32
    );
    let mut ramp = process.invoke(&case.steps[0].operation, "b", v1::StepRequest { execution: "b".into(), ..step }).await?;
    let progress = ramp.child().await?;
    assert_eq!(v1::ReportProgress::decode(progress.payload.as_slice())?.execution, "b");
    ramp.reply(progress.call, v1::ReportProgressResponse {}).await?;
    ramp.finish::<v1::StepResponse>().await?;

    // Cleanup May Be the First Operation, Including on a Case Never Initialized.
    let mut cleanup = process
        .invoke(
            "thermal.cycle.cleanup",
            "unstarted",
            v1::CleanupRequest {
                execution: "unstarted".into(),
                termination: v1::Termination::Interrupted as i32,
                test_case: Some(case.clone()),
                origin: Some(origin.clone()),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, dokime::protocol::context::v1::evidence::LogsAppendResponse {}).await?;
    cleanup.finish::<v1::CleanupResponse>().await?;

    // Finish the Remaining Execution and Verify Managed Stop Exits Normally.
    let mut cleanup = process
        .invoke(
            "thermal.cycle.cleanup",
            "b",
            v1::CleanupRequest {
                execution: "b".into(),
                termination: v1::Termination::Completed as i32,
                test_case: Some(case),
                origin: Some(origin),
            },
        )
        .await?;
    let log = cleanup.child().await?;
    cleanup.reply(log.call, dokime::protocol::context::v1::evidence::LogsAppendResponse {}).await?;
    cleanup.finish::<v1::CleanupResponse>().await?;
    process.stop(false).await?;
    Ok(())
}

/* === Tests === */
