/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use dokime_runtime::runtime::query::{
    DownsampleTechnique, PlotAnalysis, PlotDomain, PlotPayloadCursor, PlotPayloadRequest,
    PlotSeriesSummary, PlotWindow, RuntimePlotQueryMode, WifiBandScan, WifiBandWindow,
    WifiChannelCount,
};

pub(crate) async fn external_analysis_reload() {
    let root = tempdir().unwrap();
    let store = crate::service_support::plugins::plugin_store(root.path());
    let bundle = crate::service_support::plugins::fixture_bundle_bytes("obc-plugin");
    let staged = store.stage(&bundle).unwrap();
    store
        .consent(staged.installation.install_id, &[], &Default::default())
        .unwrap();
    let (mut owner, runtime) = booted(root.path()).await;
    register_plot_suite(&runtime).await;
    runtime
        .publish_telemetry_batch(samples("host.wifi.channel_occupancy", 100, None))
        .await
        .unwrap();
    let mut request = wifi_request(None, Some(1), None);
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "obc-plugin.ripple".into(),
        version: 1,
        parameters: Default::default(),
    });
    for reload in [false, true] {
        if reload {
            owner.reinitialize_plugin("obc-plugin").await.unwrap();
        }
        let definitions = runtime.analysis_definitions().await.unwrap();
        assert_eq!(
            definitions
                .iter()
                .filter(|definition| definition.id == "obc-plugin.ripple")
                .count(),
            1
        );
        let envelopes = runtime
            .query_plot_payloads(vec![request.clone()], RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let computed = envelopes[0].payload.computed.as_ref().unwrap();
        assert_eq!(computed.selection.id, "obc-plugin.ripple");
        assert_eq!(computed.series[0].input_samples, 100);
        assert_eq!(computed.series[0].output.outputs[0].values, [99.0]);
        assert_eq!(computed.series[0].output.outputs[0].unit, "count");
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}
