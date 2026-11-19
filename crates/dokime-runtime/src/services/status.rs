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
    error, io,
    num::NonZeroUsize,
    sync::{Arc, RwLock, Weak},
};

// Third Party Imports
use harmos_runtime::services::status::{Contribution, Entry, StatusSettings};
use sha2::Digest;

// Local Imports
use dokime_operations::host::{ClientSettings, Host};

use harmos_protocol::host::ComponentState;
use harmos_protocol::host::{HealthComponent, PluginEntry, PluginState};
use harmos_runtime::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Facts {
    pub host: Weak<dyn Host>,
    pub plugins: Arc<RwLock<Vec<PluginEntry>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Contribution for Facts {
    fn health(&self) -> Result<Vec<HealthComponent>, Box<dyn error::Error + Send + Sync>> {
        let host = self.host.upgrade().ok_or_else(|| io::Error::other("Read Host Health: Host Is Closed"))?;
        let state = if host.shutting_down() { ComponentState::Unavailable } else { ComponentState::Healthy };
        let mut components = vec![HealthComponent {
            name: "dokime.executions".into(),
            state: state as i32,
            reason: String::new(),
        }];
        for (name, reason) in host.plugin_admission_failures()? {
            components.push(HealthComponent {
                name: format!("dokime.plugins.{name}"),
                state: ComponentState::Degraded as i32,
                reason,
            });
        }
        let plugins = self.plugins.read().map_err(|source| io::Error::other(format!("Read Plugin Health: {source}")))?;
        for plugin in plugins.iter() {
            let state = if plugin.state == PluginState::Ready as i32 { ComponentState::Healthy } else { ComponentState::Degraded };
            components.push(HealthComponent {
                name: format!("dokime.plugins.{}", plugin.name),
                state: state as i32,
                reason: String::new(),
            });
        }
        Ok(components)
    }
    fn diagnostics(&self) -> Result<Vec<Entry>, Box<dyn error::Error + Send + Sync>> {
        let host = self.host.upgrade().ok_or_else(|| io::Error::other("Collect Host Diagnostics: Host Is Closed"))?;
        let identity = host.identity();
        let paths = host.bootstrap_paths();
        let entries = vec![
            Entry {
                path: "dokime/protocol.json".into(),
                text: serde_json::to_string(&serde_json::json!({
                    "package": "dokime.model.v1", "descriptor_sha256": sha2::Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec(),
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

pub(super) fn failure(action: &str, source: impl error::Error + Send + Sync + 'static) -> crate::Error {
    Error::OperationFailed {
        operation: action.into(),
        source: Arc::new(source),
    }
    .into()
}

/* === Private Functions === */

/* === Tests === */
