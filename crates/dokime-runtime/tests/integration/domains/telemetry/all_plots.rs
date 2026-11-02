/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded All Frames Across A Four-Hour Capture.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::{all_plots::AllPlots, prelude::*};
use dokime_runtime::runtime::query::RuntimePlotQueryMode;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn all_plot_previews() {
    let root = tempdir().unwrap();
    let fixture = AllPlots::new(root.path(), 144, 100, 32).await;
    let at = fixture.runtime.storage().cursor_saved();
    fixture.runtime.query_plot_payloads_at(Vec::new(), RuntimePlotQueryMode::Snapshot, &at).await.unwrap();
    let before = fixture.runtime.storage_read_counters();
    for request in fixture.requests(17, None) {
        let result = fixture.runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).await.unwrap();
        let payload = &result[0].payload;
        assert!(payload.series.iter().all(|series| series.raw_points == 144 && series.points.len() <= 17));
        assert!(payload.markers.len() <= 17);
        if result[0].plot_id.starts_with("fixture.host.") {
            assert!(!payload.markers.is_empty());
        }
        assert!(payload.analysis.is_none());
    }
    let after = fixture.runtime.storage_read_counters();
    // The Shared Counter Also Charges Asynchronous Record Projection. Even Two
    // Passes Over The 32 Markers Cost Less Than One 144-Row Telemetry Scan.
    assert!(after.data_scan_rows - before.data_scan_rows <= 64, "All must use the bounded checkpoints");
    // A Later Checkpoint Must Not Rewrite An Explicit Historical Cut.
    fixture.append(144, 100).await;
    let request = fixture.requests(17, None).pop().unwrap();
    let current = fixture.runtime.query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert_eq!(current[0].payload.series[0].raw_points, 145);
    assert!(current[0].revision > at.sequence);
    let historical = fixture.runtime.query_plot_payloads_at(vec![request], RuntimePlotQueryMode::Snapshot, &at).await.unwrap();
    assert_eq!(historical[0].payload.series[0].raw_points, 144);
    assert_eq!(historical[0].revision, at.sequence);
    let requests = fixture.requests(17, Some(300.0));
    let timed = fixture.runtime.query_plot_payloads(requests, RuntimePlotQueryMode::Snapshot).await.unwrap();
    assert!(timed.iter().all(|envelope| envelope.payload.series.iter().all(|series| series.raw_points == 4)));
    assert!(timed[..8].iter().all(|envelope| !envelope.payload.markers.is_empty() && envelope.payload.markers.len() <= 17));
    assert_eq!(timed[8].payload.series, timed[10].payload.series);
    assert_eq!(timed[9].payload.series, timed[11].payload.series);
    fixture.close().await;
}
