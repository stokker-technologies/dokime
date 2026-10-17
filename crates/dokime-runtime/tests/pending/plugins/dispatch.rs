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
// an installed bundle boots its declared operations and reports live status.

pub(crate) async fn external_bundle_live() {
    let root = tempfile::tempdir().unwrap();
    let bytes = fixture_bundle_bytes("obc-plugin");
    let store = crate::service_support::plugins::plugin_store(root.path());
    let staged = store.stage(&bytes).unwrap();
    store
        .consent(staged.installation.install_id, &[], &Default::default())
        .unwrap();
    let (mut runtime, handle) = booted(root.path()).await;
    let row = store.list().unwrap().remove(0);
    assert!(
        matches!(
            row.installation.last_boot,
            Some(dokime_runtime::plugins::PluginLastBoot::Loaded { .. })
        ),
        "{row:?}"
    );

    // Boot succeeding already proves the bundle's bound operations (the settings
    // status probe and the ripple analysis) were admitted: `Bindings::inspect`
    // refuses undeclared, reserved, or doubly claimed operations before the
    // plugin is marked loaded. Read the probe live through the settings frame
    // the way an operator's settings page does.
    let frame = handle.settings_frame().await;
    let card = frame
        .cards
        .iter()
        .find(|card| card.plugin_id == identifier("obc-plugin"))
        .expect("the OBC settings card is present");
    assert_eq!(
        card.state,
        dokime_runtime::domains::settings::RuntimeSettingsCardState::Ready,
        "{card:?}"
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
}

/* === Tests === */
