/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tempfile::tempdir;

use super::*;
use dokime_runtime::Runtime;
use dokime_runtime::plugins::PluginBundle;

pub(crate) async fn broken_configured_loud() {
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
