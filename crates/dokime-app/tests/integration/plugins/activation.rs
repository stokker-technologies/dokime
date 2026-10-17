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
use tempfile::tempdir;
// Local Imports
use super::*;
use dokime_app::Runtime;
use dokime_app::plugins::PluginBundle;

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
#[tokio::test]
async fn broken_configured_loud() {
    let root = tempdir().unwrap();
    let invalid = root.path().join("invalid-sidecar");
    std::fs::write(&invalid, b"not a sidecar").unwrap();

    let result = {
        let runtime_config = config(root.path()).with_plugins([PluginBundle::new(invalid)]);
        Runtime::start(runtime_config)
    }
    .await;

    assert!(result.is_err());
}
