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
use super::support::{TestResult, plugin::Process, resources::ResourceState};
use dokime::Origin;
use dokime::PluginRun;
use dokime::Termination;
use dokime::origin;
use dokime::plugin::ExecutionsCleanupRequest;
use dokime::plugin::ExecutionsCleanupResponse;
use dokime::plugin::ExecutionsInitializeRequest;
use dokime::plugin::ExecutionsInitializeResponse;
use dokime::plugin::TestCasesMaterializeRequest;
use dokime::plugin::TestCasesMaterializeResponse;
use harmos_protocol::plugin::InstanceSamplesRequest;
use harmos_protocol::plugin::Tick;
use harmos_protocol::plugin::instance_samples_request::Frame as InstanceSamplesRequestFrame;

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
async fn cleanup_releases_resources_while_an_observer_finishes_its_device_read() -> TestResult {
    // Materialize the Resource Fixture Without Allocating Its Device.
    let mut process = Process::start("dokime-rust-resources", false).await?;
    let description = process.describe().await?;
    let request = TestCasesMaterializeRequest {
        template: "resources.case".into(),
        parameters: Default::default(),
    };
    let case = process
        .invoke("resources.case.materialize", "", request)
        .await?
        .finish::<TestCasesMaterializeResponse>()
        .await?
        .test_case
        .ok_or("Missing Materialized Test Case")?;
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            test_case: Some(case.clone()),
            plugin: description.id,
            template: "resources.case".into(),
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
    process.invoke("resources.case.initialize", "retained", request).await?.finish::<ExecutionsInitializeResponse>().await?;
    let (ticks, mut samples) = process.publish("resources.state", "retained").await?;
    let at = prost_types::Timestamp { seconds: 100, nanos: 0 };
    ticks
        .send(InstanceSamplesRequest {
            frame: Some(InstanceSamplesRequestFrame::Tick(Tick { at: Some(at) })),
        })
        .await?;
    process.invoke("resources.observer_ready", "", ()).await?.finish::<()>().await?;

    // Complete Derived Cleanup While the Observer Still Holds Its Call Context.
    let request = ExecutionsCleanupRequest {
        execution: "retained".into(),
        termination: Termination::Completed as i32,
        test_case: Some(case),
        origin: Some(origin),
    };
    process.invoke("resources.case.cleanup", "retained", request).await?.finish::<ExecutionsCleanupResponse>().await?;

    // Release the Device Read and See That the Store Was Emptied, Not Merely Unregistered.
    process.invoke("resources.release_observer", "", ()).await?.finish::<()>().await?;
    let sample = samples.message().await?.ok_or("Missing Resource State")?.samples.remove(0);
    let state = <ResourceState as dokime::encoding::Message>::decode(sample.row.as_slice())?;
    assert!(state.before);
    assert!(!state.present);
    assert_eq!(state.dropped, 1);
    assert_eq!(sample.at, Some(at));
    drop(ticks);
    assert!(samples.message().await?.is_none());
    process.stop(false).await?;
    Ok(())
}

/* === Tests === */
