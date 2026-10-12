/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod presentation;
mod storage;

/* === Re-Exports === */
pub use presentation::SettingsCardRefreshPolicy;
pub use storage::StorageSource;

/* === Imports === */
// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};
// Third Party Imports
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::RuntimeHandle;
use dokime::core::{HostCardBlock, HostCardBody, HostCardTone, Identifier, Value};
use dokime::domains::{DispatchContext, StatusResponse};
use presentation::{SettingsCardContribution, SettingsSectionContribution};

/* === Constants === */
pub const IDENTITY_SETTINGS_SECTION_ID: &str = "identity";
const LIVE_STATUS_TIMEOUT: Duration = Duration::from_secs(2);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSettingsFrame {
    pub sections: Vec<RuntimeSettingsSection>,
    pub cards: Vec<RuntimeSettingsCard>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSettingsSection {
    pub section_id: Identifier,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    pub owner: RuntimeSettingsSectionOwner,
    pub order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeSettingsCard {
    pub plugin_id: Identifier,
    pub card_id: Identifier,
    pub section_id: Identifier,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    pub body: HostCardBody,
    #[serde(default)]
    pub refresh: SettingsCardRefreshPolicy,
    pub state: RuntimeSettingsCardState,
    pub order: i32,
}

#[derive(Clone)]
struct SettingsSource {
    plugin_id: Identifier,
    sections: Vec<SettingsSectionContribution>,
    cards: Vec<SettingsCardContribution>,
    state: RuntimeSettingsCardState,
    live_status: Option<Result<StatusResponse, String>>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RuntimeSettingsSectionOwner {
    App,
    InstalledPlugin { plugin_id: Identifier },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSettingsCardState {
    Ready,
    Disabled,
    Disconnected,
    Stale,
    Error,
    PermissionDenied,
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn settings_frame(&self) -> RuntimeSettingsFrame {
        let plugins = self.installed_plugins();
        let sources = join_all(plugins.values().map(|module| {
            let plugin_id = Identifier::new(&module.source).expect("installed source is valid");
            let (sections, cards) = presentation(module);
            let (state, dispatch_context) = self.settings_card_posture(module);
            let dispatcher = (module.enabled
                && module.settings_status.is_some()
                && !cards.is_empty()
                && dispatch_context.is_some())
            .then(|| self.action_dispatcher(&plugin_id))
            .flatten();
            async move {
                let live_status = match dispatcher {
                    Some(dispatcher) => Some(
                        tokio::time::timeout(LIVE_STATUS_TIMEOUT, dispatcher.status())
                            .await
                            .map_err(|_| "request exceeded the 2 second timeout".to_owned())
                            .and_then(|status| status.map_err(|error| error.to_string())),
                    ),
                    None => None,
                };
                SettingsSource {
                    plugin_id,
                    sections,
                    cards,
                    state,
                    live_status,
                }
            }
        }))
        .await;
        settings_frame_from_sources(sources)
    }

    fn settings_card_posture(
        &self,
        module: &crate::InstalledPlugin,
    ) -> (RuntimeSettingsCardState, Option<DispatchContext>) {
        if !module.enabled {
            return (RuntimeSettingsCardState::Disabled, None);
        }

        match self.action_dispatch_context(module) {
            Ok(context) if context.credentials_configured => {
                (RuntimeSettingsCardState::Ready, Some(context))
            }
            Ok(context) => (RuntimeSettingsCardState::Disconnected, Some(context)),
            Err(_) => (RuntimeSettingsCardState::Error, None),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn presentation(
    module: &crate::InstalledPlugin,
) -> (
    Vec<SettingsSectionContribution>,
    Vec<SettingsCardContribution>,
) {
    let section_id = Identifier::new(&module.source).expect("valid installed source");
    let metadata = || {
        dokime::core::Metadata::new(
            section_id.clone(),
            module.source.clone(),
            Some(module.description.clone()),
        )
    };
    let sections = if module.settings.contributions.is_empty() && module.settings_status.is_none() {
        Vec::new()
    } else {
        vec![SettingsSectionContribution {
            metadata: metadata(),
            order: None,
        }]
    };

    // One card per plugin, identified and labelled the way the plugin is: the
    // sections the plugin declared are the card's form, and the readiness block
    // below is the one thing on it that is not scanned from the binary. Its rows
    // are whatever the probe answers, so nothing is declared twice.
    let cards = module
        .settings_status
        .is_some()
        .then(|| SettingsCardContribution {
            metadata: metadata(),
            section_id: section_id.clone(),
            body: HostCardBody {
                blocks: vec![
                    HostCardBlock::Status {
                        label: "Disconnected".into(),
                        tone: HostCardTone::Warning,
                        detail: Some(module.description.clone()),
                    },
                    HostCardBlock::KeyValueList { items: Vec::new() },
                ],
            },
            refresh: SettingsCardRefreshPolicy::Manual,
            order: None,
        })
        .into_iter()
        .collect();

    (sections, cards)
}

fn settings_frame_from_sources(sources: Vec<SettingsSource>) -> RuntimeSettingsFrame {
    let mut sections = BTreeMap::from([identity_settings_section()]);

    for source in &sources {
        for section in &source.sections {
            sections
                .entry(section.metadata.identifier.clone())
                .or_insert_with(|| RuntimeSettingsSection {
                    section_id: section.metadata.identifier.clone(),
                    label: section.metadata.label.clone(),
                    description: section.metadata.description.clone(),
                    owner: RuntimeSettingsSectionOwner::InstalledPlugin {
                        plugin_id: source.plugin_id.clone(),
                    },
                    order: section.order.unwrap_or(1_000),
                });
        }
    }

    let known_sections = sections.keys().cloned().collect::<BTreeSet<_>>();
    let mut cards = Vec::new();

    for source in &sources {
        for card in &source.cards {
            let section_id = card.section_id.clone();
            if !known_sections.contains(&section_id) {
                continue;
            }

            let mut body = card.body.clone();
            let state = apply_live_status(&mut body, source.state, source.live_status.as_ref());
            cards.push(RuntimeSettingsCard {
                plugin_id: source.plugin_id.clone(),
                card_id: card.metadata.identifier.clone(),
                section_id,
                label: card.metadata.label.clone(),
                description: card.metadata.description.clone(),
                body,
                refresh: card.refresh.clone(),
                state,
                order: card.order.unwrap_or(1_000),
            });
        }
    }

    let mut sections = sections.into_values().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.section_id.cmp(&right.section_id))
    });
    cards.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.label.cmp(&right.label))
            .then_with(|| left.card_id.cmp(&right.card_id))
    });

    RuntimeSettingsFrame { sections, cards }
}

fn apply_live_status(
    body: &mut HostCardBody,
    fallback_state: RuntimeSettingsCardState,
    live_status: Option<&Result<StatusResponse, String>>,
) -> RuntimeSettingsCardState {
    match live_status {
        None => fallback_state,
        Some(Err(error)) => {
            if let Some(HostCardBlock::Status { detail, .. }) = body
                .blocks
                .iter_mut()
                .find(|block| matches!(block, HostCardBlock::Status { .. }))
            {
                *detail = Some(format!("Live status unavailable: {error}"));
            }
            fallback_state
        }
        Some(Ok(status)) => {
            let (state, label, tone) = live_state(&status.state);
            if let Some(HostCardBlock::Status {
                label: card_label,
                tone: card_tone,
                detail,
            }) = body
                .blocks
                .iter_mut()
                .find(|block| matches!(block, HostCardBlock::Status { .. }))
            {
                *card_label = label.to_owned();
                *card_tone = tone;
                *detail = Some(format!("Live sidecar state: {}.", status.state));
            }
            // The probe owns the rows: a card declares no key-value labels of its
            // own, so what it shows is exactly what the plugin just answered.
            for block in &mut body.blocks {
                if let HostCardBlock::KeyValueList { items } = block {
                    *items = status
                        .fields
                        .iter()
                        .map(|(label, value)| dokime::core::HostCardKeyValue {
                            label: label.clone(),
                            value: display_value(value),
                            detail: None,
                        })
                        .collect();
                }
            }
            state
        }
    }
}

fn live_state(state: &str) -> (RuntimeSettingsCardState, &'static str, HostCardTone) {
    match state.trim().to_ascii_lowercase().as_str() {
        "connected" => (
            RuntimeSettingsCardState::Ready,
            "Connected",
            HostCardTone::Success,
        ),
        "disconnected" => (
            RuntimeSettingsCardState::Disconnected,
            "Disconnected",
            HostCardTone::Warning,
        ),
        state if state.starts_with("missing_") => (
            RuntimeSettingsCardState::Disconnected,
            "Disconnected",
            HostCardTone::Warning,
        ),
        "disabled" => (
            RuntimeSettingsCardState::Disabled,
            "Disabled",
            HostCardTone::Neutral,
        ),
        "stale" => (
            RuntimeSettingsCardState::Stale,
            "Stale",
            HostCardTone::Warning,
        ),
        "permission_denied" => (
            RuntimeSettingsCardState::PermissionDenied,
            "Permission denied",
            HostCardTone::Danger,
        ),
        "error" | "failed" | "unknown" => (
            RuntimeSettingsCardState::Error,
            "Error",
            HostCardTone::Danger,
        ),
        _ => (
            RuntimeSettingsCardState::Error,
            "Error",
            HostCardTone::Danger,
        ),
    }
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Null => "Not configured".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Integer(value) => value.to_string(),
        Value::Unsigned(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Text(value) => value.clone(),
        Value::Datetime(timestamp) => crate::core::datetime_text(*timestamp),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| "Unavailable".to_owned())
        }
    }
}

fn identity_settings_section() -> (Identifier, RuntimeSettingsSection) {
    let section_id = Identifier::new(IDENTITY_SETTINGS_SECTION_ID).expect("valid section id");

    (
        section_id.clone(),
        RuntimeSettingsSection {
            section_id,
            label: "Identity".to_owned(),
            description: Some("Host and installed plugin identity settings.".to_owned()),
            owner: RuntimeSettingsSectionOwner::App,
            order: 10,
        },
    )
}

/* === Tests === */
