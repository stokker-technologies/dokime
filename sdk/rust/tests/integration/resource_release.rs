/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Resource Lifetime Proof

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::support::{
    TestResult,
    plugin::{Placement, Process},
};
use dokime::Origin;
use dokime::PluginRun;
use dokime::Termination;
use dokime::origin;
use dokime::plugin::ExecutionsCleanupResponse;
use dokime::plugin::ExecutionsInitializeResponse;
use dokime::plugin::TestCasesMaterializeResponse;
use dokime_protocol::plugin::ExecutionsCleanupRequest;
use dokime_protocol::plugin::ExecutionsInitializeRequest;
use dokime_protocol::plugin::TestCasesMaterializeRequest;
use dokime_rust_stories::ResourceState;
use harmos_protocol::plugin::Window;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The Fixture's Resource State Row, One Value per Sample.
#[derive(dokime::Message)]
#[dokime(package = "resources.v1")]
struct ResourceStateColumns {
    before: Vec<bool>,
    present: Vec<bool>,
    dropped: Vec<u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn cleanup_releases_resources_while_an_observer_finishes_its_device_read() -> TestResult {
    // Materialize the Resource Fixture Without Allocating Its Device.
    let mut process = Process::start("dokime-rust-resources", Placement::Managed).await?;
    let description = process.describe().await?;
    let request = TestCasesMaterializeRequest {
        template: "resources.resources.case".into(),
        parameters: Default::default(),
    };
    let case = process
        .invoke("resources.resources.case.materialize", "", request)
        .await?
        .finish::<TestCasesMaterializeResponse>()
        .await?
        .test_case
        .ok_or("Missing Materialized Test Case")?;
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "resources.resources.case".into(),
            digest: description.declarations.ok_or("Missing Declarations")?.digest,
            started_at: description.started_at,
        })),
    };

    // Initialize a Device and Begin an Observer Read on the Execution's Store.
    let request = ExecutionsInitializeRequest {
        execution: "retained".into(),
        test_case: Some(case.clone()),
        origin: Some(origin.clone()),
    };
    process.invoke("resources.resources.case.initialize", "retained", request).await?.finish::<ExecutionsInitializeResponse>().await?;
    let (grants, mut samples) = process.publish("resources.resources.case.state", "retained").await?;
    let window = Window {
        start: Some(prost_types::Timestamp { seconds: 100, nanos: 0 }),
        period: Some(prost_types::Duration { seconds: 1, nanos: 0 }),
        count: 1,
        sequence: 1,
        capturing_executions: Vec::new(),
    };
    let answer = tokio::spawn(async move {
        let columns = Process::window(&grants, &mut samples, window).await.map_err(|error| error.to_string());
        (columns, grants, samples)
    });
    process.invoke("resources.observer_ready", "", ()).await?.finish::<()>().await?;

    // Complete Derived Cleanup While the Observer Still Holds Its Call Context.
    let request = ExecutionsCleanupRequest {
        execution: "retained".into(),
        termination: Termination::Completed as i32,
        test_case: Some(case),
        origin: Some(origin),
    };
    process.invoke("resources.resources.case.cleanup", "retained", request).await?.finish::<ExecutionsCleanupResponse>().await?;

    // Release the Device Read and See That the Store Was Emptied, Not Merely Unregistered.
    process.invoke("resources.release_observer", "", ()).await?.finish::<()>().await?;
    let (columns, grants, mut samples) = answer.await?;
    let columns = <ResourceStateColumns as dokime::encoding::Message>::decode(columns?.as_slice())?;
    assert_eq!((columns.before.len(), columns.present.len(), columns.dropped.len()), (1, 1, 1));
    // The Window's Only Sample Is the State the Released Device Read Observed.
    let state = ResourceState {
        before: columns.before[0],
        present: columns.present[0],
        dropped: columns.dropped[0],
    };
    assert!(state.before);
    assert!(!state.present);
    assert_eq!(state.dropped, 1);
    drop(grants);
    assert!(samples.message().await?.is_none());
    process.stop().await?;
    Ok(())
}

/* === Tests === */
