/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Window Queries Reuse Saved Buckets And Fetch Detail On Zoom.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::service_support::{all_plots::AllPlots, prelude::*};
use dokime_runtime::runtime::query::PlotWindow;

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
async fn windowed_plot_zoom_and_late_sample() {
    // Capture A Saved History With Reduced Whole-History Previews.
    let root = tempdir().unwrap();
    let fixture = AllPlots::new(root.path(), 128, 1, 8).await;
    let mut request = fixture.requests(100, Some(64.0)).remove(0);
    request.marker_record_types.clear();
    request.panel_width = Some(4);
    let wide = fixture.runtime.query_plot_payloads(vec![request.clone()]).await.unwrap();
    assert_eq!(wide[0].payload.series[0].raw_points, 65);
    assert!(wide[0].payload.series[0].points.len() < 65);
    assert_eq!(wide[0].payload.summaries[0].count, 65);

    // Reuse Closed Buckets On An Unchanged Window.
    let before = fixture.runtime.storage_read_counters();
    let repeated = fixture.runtime.query_plot_payloads(vec![request.clone()]).await.unwrap();
    let after = fixture.runtime.storage_read_counters();
    assert_eq!(wide[0].payload.series, repeated[0].payload.series);
    assert!(after.data_scan_rows - before.data_scan_rows < 65);

    // Fetch More Detail For A Narrow Viewport At The Same Width.
    let mut zoom = request.clone();
    zoom.viewport = Some(PlotWindow { from: 100.0, to: 110.0 });
    let detail = fixture.runtime.query_plot_payloads(vec![zoom]).await.unwrap();
    assert_eq!(detail[0].payload.series[0].raw_points, 11);
    assert!(detail[0].payload.series[0].points.iter().all(|point| {
        let at = time::OffsetDateTime::parse(point.x.as_str().unwrap(), &time::format_description::well_known::Rfc3339)
            .unwrap()
            .unix_timestamp();
        (100..=110).contains(&at)
    }));
    assert_eq!(detail[0].payload.summaries[0].count, 11);
    let wide_visible = wide[0].payload.series[0]
        .points
        .iter()
        .filter(|point| {
            let at = time::OffsetDateTime::parse(point.x.as_str().unwrap(), &time::format_description::well_known::Rfc3339)
                .unwrap()
                .unix_timestamp();
            (100..=110).contains(&at)
        })
        .count();
    assert!(detail[0].payload.series[0].points.len() > wide_visible);

    // Reopen A Previously Saved Bucket For A Late Arrival.
    fixture.append(80, 1).await;
    let late = fixture.runtime.query_plot_payloads(vec![request]).await.unwrap();
    assert_eq!(late[0].payload.series[0].raw_points, 66);
    assert_eq!(late[0].payload.summaries[0].count, 66);
    fixture.close().await;
}
