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
use std::collections::BTreeMap;
// Third Party Imports
use prost::Message;
use tonic::Status as TonicStatus;
// Local Imports
use super::super::{
    execution,
    value::{self, Budget},
};
use dokime_app::Error;
use dokime_app::{
    capabilities::{
        installed_plugins::{RuntimePluginManagementError, RuntimePluginSettings},
        settings::{
            RuntimeSettingsCard, RuntimeSettingsCardState, RuntimeSettingsFrame,
            RuntimeSettingsSection, RuntimeSettingsSectionOwner, SettingsCardRefreshPolicy,
        },
    },
    core::{
        HostCardBlock, HostCardBody, HostCardKeyValue, HostCardMetric, HostCardTone, Identifier,
    },
    settings::{
        Boolean, Choice, MultipleChoice, Number, Path, SecretReference, SecretSetting, Settings,
        SettingsEntry, TextArea,
    },
};
use dokime_protocol::v1;

/* === Constants === */
pub(super) const MAX_TEXT_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn identifier(value: String, field: &str) -> Result<Identifier, TonicStatus> {
    bounded_text(value.clone(), field)?;
    Identifier::new(value)
        .map_err(|error| TonicStatus::invalid_argument(format!("{field}: {error}")))
}

pub(super) fn bounded_text(value: String, field: &str) -> Result<String, TonicStatus> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(TonicStatus::resource_exhausted(format!(
            "{field} exceeds {MAX_TEXT_BYTES} bytes"
        )));
    }
    Ok(value)
}

pub(super) fn encode_frame(
    frame: RuntimeSettingsFrame,
    max_message_bytes: usize,
) -> Result<v1::SettingsFrame, TonicStatus> {
    let mut budget = Budget::new();
    value::collection(frame.sections.len())?;
    value::collection(frame.cards.len())?;
    let sections = frame
        .sections
        .into_iter()
        .map(encode_section)
        .collect::<Result<_, _>>()?;
    let cards = frame
        .cards
        .into_iter()
        .map(|card| encode_card(card, &mut budget))
        .collect::<Result<_, _>>()?;
    let frame = v1::SettingsFrame { sections, cards };
    ensure_size(&frame, max_message_bytes, "settings frame")?;
    Ok(frame)
}

pub(super) fn encode_plugin_settings(
    settings: RuntimePluginSettings,
    max_message_bytes: usize,
) -> Result<v1::PluginRuntimeSettings, TonicStatus> {
    let mut budget = Budget::new();
    value::collection(settings.settings.contributions.len())?;
    value::collection(settings.values.len())?;
    value::collection(settings.credential_bindings.bindings.len())?;
    value::collection(settings.credential_bindings.missing_required.len())?;
    let values = settings
        .values
        .into_iter()
        .map(|(key, value)| {
            bounded_text(key.clone(), "setting value key")?;
            Ok((key, value::encode(&value, &mut budget, 0)?))
        })
        .collect::<Result<BTreeMap<_, _>, TonicStatus>>()?;
    let discovered = settings
        .discovered
        .into_iter()
        .map(|(key, value)| {
            bounded_text(key.clone(), "discovered setting key")?;
            Ok((key, value::encode(&value, &mut budget, 0)?))
        })
        .collect::<Result<BTreeMap<_, _>, TonicStatus>>()?;
    let response = v1::PluginRuntimeSettings {
        settings: Some(encode_settings(settings.settings)?),
        values: Some(v1::ValueObject { fields: values }),
        discovered: Some(v1::ValueObject { fields: discovered }),
        credential_bindings: Some(encode_credentials(
            settings.credential_bindings,
            &mut budget,
        )?),
    };
    ensure_size(&response, max_message_bytes, "plugin settings response")?;
    Ok(response)
}

pub(super) fn plugin_error(error: RuntimePluginManagementError) -> TonicStatus {
    let message = error.to_string();
    let error = match error {
        RuntimePluginManagementError::NotFound(_) => {
            Error::not_found("installed_plugin_not_found", message)
        }
        RuntimePluginManagementError::InvalidInput(_) => {
            Error::validation("invalid_plugin_settings", message, vec![])
        }
        RuntimePluginManagementError::Store(_) => Error::internal("plugin_settings_store", message),
    };
    execution::status(error)
}

/* === Private Functions === */
fn encode_section(section: RuntimeSettingsSection) -> Result<v1::SettingsSection, TonicStatus> {
    let section_id = checked_identifier(&section.section_id, "section_id")?;
    bounded_text(section.label.clone(), "section label")?;
    let description = section
        .description
        .map(|description| bounded_text(description, "section description"))
        .transpose()?;
    let owner = match section.owner {
        RuntimeSettingsSectionOwner::App => Some(v1::settings_section::Owner::App(true)),
        RuntimeSettingsSectionOwner::InstalledPlugin { plugin_id } => {
            Some(v1::settings_section::Owner::InstalledPluginId(
                checked_identifier(&plugin_id, "section owner plugin_id")?,
            ))
        }
    };
    Ok(v1::SettingsSection {
        section_id,
        label: section.label,
        description,
        owner,
        order: section.order,
    })
}

fn encode_card(
    card: RuntimeSettingsCard,
    budget: &mut Budget,
) -> Result<v1::SettingsCard, TonicStatus> {
    let plugin_id = checked_identifier(&card.plugin_id, "card plugin_id")?;
    let card_id = checked_identifier(&card.card_id, "card_id")?;
    let section_id = checked_identifier(&card.section_id, "card section_id")?;
    bounded_text(card.label.clone(), "card label")?;
    let description = card
        .description
        .map(|description| bounded_text(description, "card description"))
        .transpose()?;
    let body = encode_body(card.body, budget)?;
    let refresh = Some(encode_refresh(card.refresh));
    Ok(v1::SettingsCard {
        plugin_id,
        card_id,
        section_id,
        label: card.label,
        description,
        body: Some(body),
        refresh,
        state: encode_card_state(card.state) as i32,
        order: card.order,
    })
}

fn encode_body(body: HostCardBody, _budget: &mut Budget) -> Result<v1::HostCardBody, TonicStatus> {
    value::collection(body.blocks.len())?;
    Ok(v1::HostCardBody {
        blocks: body
            .blocks
            .into_iter()
            .map(encode_block)
            .collect::<Result<_, _>>()?,
    })
}

fn encode_block(block: HostCardBlock) -> Result<v1::HostCardBlock, TonicStatus> {
    use v1::host_card_block::Kind;
    Ok(v1::HostCardBlock {
        kind: Some(match block {
            HostCardBlock::Text { text } => Kind::Text(bounded_text(text, "card text")?),
            HostCardBlock::Status {
                label,
                tone,
                detail,
            } => Kind::Status(v1::HostCardStatus {
                label: bounded_text(label, "status label")?,
                tone: encode_tone(tone) as i32,
                detail: detail
                    .map(|detail| bounded_text(detail, "status detail"))
                    .transpose()?,
            }),
            HostCardBlock::KeyValueList { items } => {
                value::collection(items.len())?;
                Kind::KeyValueList(v1::HostCardKeyValueList {
                    items: items
                        .into_iter()
                        .map(encode_key_value)
                        .collect::<Result<_, _>>()?,
                })
            }
            HostCardBlock::Metrics { items } => {
                value::collection(items.len())?;
                Kind::Metrics(v1::HostCardMetrics {
                    items: items
                        .into_iter()
                        .map(encode_metric)
                        .collect::<Result<_, _>>()?,
                })
            }
        }),
    })
}

fn encode_key_value(item: HostCardKeyValue) -> Result<v1::HostCardKeyValue, TonicStatus> {
    Ok(v1::HostCardKeyValue {
        label: bounded_text(item.label, "key-value label")?,
        value: bounded_text(item.value, "key-value value")?,
        detail: item
            .detail
            .map(|detail| bounded_text(detail, "key-value detail"))
            .transpose()?,
    })
}

fn encode_metric(item: HostCardMetric) -> Result<v1::HostCardMetric, TonicStatus> {
    Ok(v1::HostCardMetric {
        label: bounded_text(item.label, "metric label")?,
        value: bounded_text(item.value, "metric value")?,
        unit: item
            .unit
            .map(|unit| bounded_text(unit, "metric unit"))
            .transpose()?,
        trend: item
            .trend
            .map(|trend| bounded_text(trend, "metric trend"))
            .transpose()?,
    })
}

fn encode_refresh(refresh: SettingsCardRefreshPolicy) -> v1::SettingsCardRefreshPolicy {
    use v1::settings_card_refresh_policy::Policy;
    let policy = match refresh {
        SettingsCardRefreshPolicy::Manual => Policy::Manual(v1::ManualRefresh {}),
        SettingsCardRefreshPolicy::Cached { max_age_ms } => {
            Policy::Cached(v1::CachedRefresh { max_age_ms })
        }
        SettingsCardRefreshPolicy::OnDemand => Policy::OnDemand(v1::OnDemandRefresh {}),
    };
    v1::SettingsCardRefreshPolicy {
        policy: Some(policy),
    }
}

fn encode_tone(tone: HostCardTone) -> v1::HostCardTone {
    match tone {
        HostCardTone::Neutral => v1::HostCardTone::Neutral,
        HostCardTone::Info => v1::HostCardTone::Info,
        HostCardTone::Success => v1::HostCardTone::Success,
        HostCardTone::Warning => v1::HostCardTone::Warning,
        HostCardTone::Danger => v1::HostCardTone::Danger,
    }
}

fn encode_card_state(state: RuntimeSettingsCardState) -> v1::SettingsCardState {
    match state {
        RuntimeSettingsCardState::Ready => v1::SettingsCardState::Ready,
        RuntimeSettingsCardState::Disabled => v1::SettingsCardState::Disabled,
        RuntimeSettingsCardState::Disconnected => v1::SettingsCardState::Disconnected,
        RuntimeSettingsCardState::Stale => v1::SettingsCardState::Stale,
        RuntimeSettingsCardState::Error => v1::SettingsCardState::Error,
        RuntimeSettingsCardState::PermissionDenied => v1::SettingsCardState::PermissionDenied,
    }
}

fn encode_settings(settings: Settings) -> Result<v1::SettingsCatalog, TonicStatus> {
    Ok(v1::SettingsCatalog {
        contributions: settings
            .contributions
            .into_iter()
            .map(encode_contribution)
            .collect::<Result<_, _>>()?,
    })
}

fn encode_contribution(
    contribution: SettingsEntry,
) -> Result<v1::SettingContribution, TonicStatus> {
    use v1::setting_contribution::Kind;
    let kind = match contribution {
        SettingsEntry::Boolean(value) => Kind::Boolean(encode_boolean(value)?),
        SettingsEntry::TextArea(value) => Kind::TextArea(encode_text_area(value)?),
        SettingsEntry::Path(value) => Kind::Path(encode_path(value)?),
        SettingsEntry::Number(value) => Kind::Number(encode_number(value)?),
        SettingsEntry::MultipleChoice(value) => {
            Kind::MultipleChoice(encode_multiple_choice(value)?)
        }
        SettingsEntry::SecretReference(value) => Kind::SecretReference(encode_secret(value)?),
    };
    Ok(v1::SettingContribution { kind: Some(kind) })
}

fn setting_texts(
    id: String,
    title: String,
    description: Option<String>,
) -> Result<(String, String, Option<String>), TonicStatus> {
    Ok((
        bounded_text(id, "setting id")?,
        bounded_text(title, "setting title")?,
        description
            .map(|description| bounded_text(description, "setting description"))
            .transpose()?,
    ))
}

fn encode_boolean(value: Boolean) -> Result<v1::BooleanSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(v1::BooleanSetting {
        id,
        title,
        description,
        default: value.default,
    })
}

fn encode_text_area(value: TextArea) -> Result<v1::TextAreaSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(v1::TextAreaSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "text-area default")?,
        required: value.required,
    })
}

fn encode_path(value: Path) -> Result<v1::PathSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(v1::PathSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "path default")?,
        required: value.required,
    })
}

fn encode_number(value: Number) -> Result<v1::NumberSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    for (name, number) in [
        ("number default", value.default),
        ("number min", value.min.unwrap_or(0.0)),
        ("number max", value.max.unwrap_or(0.0)),
        ("number step", value.step.unwrap_or(0.0)),
    ] {
        if !number.is_finite() {
            return Err(TonicStatus::internal(format!("{name} is not finite")));
        }
    }
    Ok(v1::NumberSetting {
        id,
        title,
        description,
        default: value.default,
        min: value.min,
        max: value.max,
        step: value.step,
    })
}

fn encode_multiple_choice(value: MultipleChoice) -> Result<v1::MultipleChoiceSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    value::collection(value.choices.len())?;
    let provider = value
        .provider
        .map(|provider| bounded_text(provider, "choice provider"))
        .transpose()?;
    Ok(v1::MultipleChoiceSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "choice default")?,
        choices: value
            .choices
            .into_iter()
            .map(encode_choice)
            .collect::<Result<_, _>>()?,
        provider,
    })
}

fn encode_choice(value: Choice) -> Result<v1::SettingChoice, TonicStatus> {
    Ok(v1::SettingChoice {
        value: bounded_text(value.value, "choice value")?,
        title: bounded_text(value.title, "choice title")?,
    })
}

fn encode_secret(value: SecretSetting) -> Result<v1::SecretSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(v1::SecretSetting {
        id,
        title,
        description,
        default: value.default.map(encode_secret_reference).transpose()?,
        required: value.required,
    })
}

fn encode_secret_reference(reference: SecretReference) -> Result<v1::SecretReference, TonicStatus> {
    use v1::secret_reference::Kind;
    let kind = match reference {
        SecretReference::Environment { variable } => {
            Kind::Environment(bounded_text(variable, "environment variable")?)
        }
        SecretReference::StoredCredential { credential_id } => {
            Kind::StoredCredential(bounded_text(credential_id, "credential id")?)
        }
    };
    Ok(v1::SecretReference { kind: Some(kind) })
}

fn encode_credentials(
    credentials: dokime_app::capabilities::installed_plugins::PluginRuntimeCredentialBindings,
    _budget: &mut Budget,
) -> Result<v1::PluginRuntimeCredentialBindings, TonicStatus> {
    value::collection(credentials.bindings.len())?;
    value::collection(credentials.missing_required.len())?;
    let bindings = credentials
        .bindings
        .into_iter()
        .map(|(key, binding)| {
            let key = bounded_text(key, "credential binding key")?;
            Ok((
                key,
                v1::RedactedCredentialBinding {
                    credential_id: bounded_text(binding.credential_id, "credential id")?,
                    backend: encode_credential_backend(binding.backend) as i32,
                    configured: binding.configured,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, TonicStatus>>()?;
    Ok(v1::PluginRuntimeCredentialBindings {
        bindings,
        missing_required: credentials
            .missing_required
            .into_iter()
            .map(|id| bounded_text(id, "missing credential id"))
            .collect::<Result<_, _>>()?,
    })
}

fn encode_credential_backend(
    backend: dokime_app::core::CredentialBackendKind,
) -> v1::CredentialBackendKind {
    match backend {
        dokime_app::core::CredentialBackendKind::OsKeyring => v1::CredentialBackendKind::OsKeyring,
        dokime_app::core::CredentialBackendKind::Memory => v1::CredentialBackendKind::Memory,
        dokime_app::core::CredentialBackendKind::LocalFile => v1::CredentialBackendKind::LocalFile,
    }
}

fn checked_identifier(value: &Identifier, field: &str) -> Result<String, TonicStatus> {
    bounded_text(value.as_str().to_owned(), field)
}

fn ensure_size<M: Message>(
    message: &M,
    max_message_bytes: usize,
    name: &str,
) -> Result<(), TonicStatus> {
    if message.encoded_len() > max_message_bytes {
        return Err(TonicStatus::resource_exhausted(format!(
            "{name} exceeds the native message budget"
        )));
    }
    Ok(())
}

/* === Tests === */
