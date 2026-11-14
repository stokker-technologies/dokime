/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Product Contributions to Library Status

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    sync::{Arc, Weak},
};

// Third Party Imports
use harmos_runtime::services::status::{Contribution, Entry, StatusSettings};
use sha2::Digest;

// Local Imports
use dokime_operations_next::client::{ClientSettings, Host};
use harmos_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Facts(pub Weak<dyn Host>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Contribution for Facts {
    fn health(&self) -> Result<Vec<v1::HealthComponent>, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.0.upgrade().ok_or_else(|| std::io::Error::other("Read Host Health: Host Is Closed"))?;
        let state = if host.shutting_down() { v1::ComponentState::Unavailable } else { v1::ComponentState::Healthy };
        let mut components = vec![v1::HealthComponent {
            name: "dokime.executions".into(),
            state: state as i32,
            reason: String::new(),
        }];
        for plugin in host.plugins()? {
            let state = match dokime_protocol::client::PluginState::try_from(plugin.state) {
                Ok(dokime_protocol::client::PluginState::Ready) => v1::ComponentState::Healthy,
                _ => v1::ComponentState::Degraded,
            };
            components.push(v1::HealthComponent {
                name: format!("dokime.plugins.{}", plugin.entry),
                state: state as i32,
                reason: String::new(),
            });
        }
        Ok(components)
    }
    fn diagnostics(&self) -> Result<Vec<Entry>, Box<dyn std::error::Error + Send + Sync>> {
        let host = self.0.upgrade().ok_or_else(|| std::io::Error::other("Collect Host Diagnostics: Host Is Closed"))?;
        let identity = host.identity();
        let paths = host.bootstrap_paths();
        let entries = vec![
            Entry {
                path: "dokime/protocol.json".into(),
                text: serde_json::to_string(&serde_json::json!({
                    "package": "dokime.v1", "descriptor_sha256": sha2::Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec(),
                    "target": identity.target, "epoch": identity.epoch,
                }))?,
            },
            Entry {
                path: "dokime/paths.json".into(),
                text: serde_json::to_string(
                    &serde_json::json!({"application": paths.application, "storage": paths.storage, "plugins": paths.plugins}),
                )?,
            },
        ];
        Ok(entries)
    }
}

/* === Public Functions === */

pub(super) fn settings(settings: &ClientSettings, host_id: String) -> StatusSettings {
    StatusSettings {
        read_timeout: settings.read_timeout,
        application_name: "Dokime Execution Host".into(),
        application_version: env!("CARGO_PKG_VERSION").into(),
        host_id,
        history_items: NonZeroUsize::new(settings.page_items).unwrap(),
        history_bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
        read_leases: NonZeroUsize::new(settings.page_items).unwrap(),
        archive_bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
        range_bytes: NonZeroUsize::new(settings.message_bytes / 2).unwrap(),
        redaction_bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
        contributions: NonZeroUsize::new(settings.stream_buffer).unwrap(),
    }
}

pub(super) fn failure(action: &str, source: impl std::error::Error + Send + Sync + 'static) -> crate::Error {
    harmos_runtime::Error::OperationFailed {
        operation: action.into(),
        source: Arc::new(source),
    }
    .into()
}

/* === Private Functions === */

/* === Tests === */
