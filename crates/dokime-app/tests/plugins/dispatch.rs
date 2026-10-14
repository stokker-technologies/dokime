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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// external bundle boots its declared routes and reports live status.

pub(crate) async fn external_bundle_live() {
    let root = tempfile::tempdir().unwrap();
    let bytes = fixture_bundle_bytes("dokime-route-plugin");
    let store = crate::support::plugins::plugin_store(root.path());
    let staged = store.stage(&bytes).unwrap();
    store
        .consent(staged.installation.install_id, &[], &Default::default())
        .unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let row = store.list().unwrap().remove(0);
    assert!(
        matches!(
            row.installation.last_boot,
            Some(dokime_app::plugins::PluginLastBoot::Loaded { .. })
        ),
        "{row:?}"
    );

    // Boot succeeding here already proves the sidecar's four declared routes
    // (a plain `invoke`, the `status` probe, and the `choices` options
    // provider, plus the controlled record publication route) were all admitted: `Bindings::inspect` refuses undeclared,
    // reserved, or doubly-claimed routes before the plugin is ever marked
    // loaded, so an `Ok` boot is the binding assertion.
    //
    // The status route is the only one left with a public, host-driven call
    // path after actions were removed (`domains::actions` and its
    // `query_artifact_options` are gone, so the `choices` route has no
    // integration-level caller to exercise from here). Read it live through
    // the settings frame the way an operator's settings page would.
    let frame = handle.settings_frame().await;
    let card = frame
        .cards
        .iter()
        .find(|card| card.plugin_id == identifier("route-fixture"))
        .expect("route fixture settings card is present");
    let serialized = serde_json::to_value(card).unwrap();
    assert!(
        serialized.to_string().contains("calls-0"),
        "expected the live status probe's call count in {serialized}"
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

/* === Tests === */
