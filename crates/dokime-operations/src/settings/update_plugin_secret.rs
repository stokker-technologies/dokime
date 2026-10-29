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

#[harmos::message(skip_debug)]
#[harmos::operation(resources = [crate::hosting::Host<UpdatePluginSecret>], codec = manual, response = PluginRuntimeSettings, id = "dokime.settings.update_plugin_secret",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "UpdatePluginSecret"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePluginSecret {
    pub plugin_id: String,
    pub secret_id: String,
    pub secret: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for UpdatePluginSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UpdatePluginSecret")
            .field("plugin_id", &self.plugin_id)
            .field("secret_id", &self.secret_id)
            .field("secret", &"<redacted>")
            .finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::UpdatePluginSecret;

    #[test]
    fn debug_redacts_values_without_changing_the_request() {
        let request = UpdatePluginSecret {
            plugin_id: "instrument".to_owned(),
            secret_id: "connection".to_owned(),
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

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(UpdatePluginSecret);
}
