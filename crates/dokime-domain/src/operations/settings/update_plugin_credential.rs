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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message(skip_debug)]
#[harmos_domain::operation(response = PluginRuntimeSettings, id = "dokime.settings.update_plugin_credential", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "UpdatePluginCredential"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePluginCredential {
    pub plugin_id: String,
    pub credential_id: String,
    pub secret: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for UpdatePluginCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UpdatePluginCredential")
            .field("plugin_id", &self.plugin_id)
            .field("credential_id", &self.credential_id)
            .field("secret", &"<redacted>")
            .finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::UpdatePluginCredential;

    #[test]
    fn debug_redacts_values_without_changing_the_request() {
        let request = UpdatePluginCredential {
            plugin_id: "instrument".to_owned(),
            credential_id: "connection".to_owned(),
            secret: "private-value-42".to_owned(),
        };
        for rendered in [format!("{request:?}"), format!("{request:#?}")] {
            assert!(!rendered.contains(&request.secret));
            assert!(rendered.contains("<redacted>"));
            assert!(rendered.contains("instrument"));
        }
        assert_eq!(request.secret, "private-value-42");
    }
}
