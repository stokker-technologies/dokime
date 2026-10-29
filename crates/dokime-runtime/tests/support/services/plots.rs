/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Saved-Telemetry Plot Paths, Shared By Their Integration Guards And Full-Scale System Stories.

mod history;

pub use history::telemetry_history;

use super::prelude::*;
use dokime_runtime::runtime::query::{DownsampleTechnique, PlotPayloadRequest};

pub async fn register_plot_suite(runtime: &RuntimeHandle) {
    let definition = TestSuiteDefinition {
        suite_id: "plot.query.suite".to_owned(),
        tags: Vec::new(),
        workflow: None,
        suite: plot_query_suite(),
        record_contracts: Vec::new(),
        telemetry_contracts: vec![wifi_occupancy_contract()],
    };
    runtime.set_suites("plot-query", dokime_runtime::definition::SuiteSet::new([definition])).await.unwrap();
}

pub fn plot_query_suite() -> TestSuite {
    TestSuite {
        metadata: Metadata::new(Identifier::new("plot.query.suite").unwrap(), "Plot Query Suite", None),
        tags: Vec::new(),
        parameter_groups: Vec::new(),
        test_case_templates: Vec::new(),
    }
}

pub fn wifi_request(scope: Option<RecordScope>, max_points: Option<usize>) -> PlotPayloadRequest {
    PlotPayloadRequest {
        plot_id: Some("wifi.network_count".to_owned()),
        plot_type: None,
        x_scale: Default::default(),
        y_scale: Default::default(),
        stream: "host.wifi.scan".to_owned(),
        y_keys: vec!["network_count".to_owned()],
        marker_record_types: Vec::new(),
        filters: Vec::new(),
        x_key: Some("sample_index".to_owned()),
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        series_by_expression: None,
        z_stream: None,
        z_field: None,
        unit: Some("count".to_owned()),
        x_format: None,
        y_format: None,
        max_points,
        window_s: None,
        scope,
        technique: DownsampleTechnique::Raw,
        ..Default::default()
    }
}
