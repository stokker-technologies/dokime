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
use dokime::harmos::domain::Portable;
use tonic::Status as TonicStatus;

// Local Imports
use super::super::{
    execution,
    value::{self, Budget, canonical},
};
use dokime::domain::model::appearance::HostCardBlock;
use dokime::domain::model::appearance::HostCardBody;
use dokime::domain::model::appearance::HostCardKeyValue;
use dokime::domain::model::appearance::HostCardMetric;
use dokime::domain::model::appearance::HostCardTone;
use dokime::domain::model::identity::Identifier;
use dokime::domains::installed_plugins::RuntimePluginManagementError;
use dokime::domains::installed_plugins::RuntimePluginSettings;
use dokime::domains::settings::RuntimeSettingsCard;
use dokime::domains::settings::RuntimeSettingsCardState;
use dokime::domains::settings::RuntimeSettingsFrame;
use dokime::domains::settings::RuntimeSettingsSection;
use dokime::domains::settings::RuntimeSettingsSectionOwner;
use dokime::domains::settings::SettingsCardRefreshPolicy;
use dokime::runtime::Error;
use dokime::settings::Boolean;
use dokime::settings::Choice;
use dokime::settings::MultipleChoice;
use dokime::settings::Number;
use dokime::settings::Path;
use dokime::settings::SecretReference;
use dokime::settings::SecretSetting;
use dokime::settings::Settings;
use dokime::settings::SettingsEntry;
use dokime::settings::TextArea;

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
    if value.len() > MAX_TEXT_BYTES {
        return Err(TonicStatus::resource_exhausted(format!("{field} exceeds {MAX_TEXT_BYTES} bytes")));
    }
    Identifier::new(value).map_err(|error| TonicStatus::invalid_argument(format!("{field}: {error}")))
}

pub(super) fn bounded_text(value: String, field: &str) -> Result<String, TonicStatus> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(TonicStatus::resource_exhausted(format!("{field} exceeds {MAX_TEXT_BYTES} bytes")));
    }
    Ok(value)
}

pub(super) fn encode_frame(
    frame: RuntimeSettingsFrame, max_message_bytes: usize,
) -> Result<dokime::domain::model::settings::wire::SettingsFrame, TonicStatus> {
    let mut budget = Budget::new();
    value::collection(frame.sections.len())?;
    value::collection(frame.cards.len())?;
    let sections = frame.sections.into_iter().map(encode_section).collect::<Result<_, _>>()?;
    let cards = frame.cards.into_iter().map(|card| encode_card(card, &mut budget)).collect::<Result<_, _>>()?;
    let frame = dokime::domain::model::settings::wire::SettingsFrame { sections, cards };
    ensure_size(&frame, max_message_bytes, "settings frame")?;
    Ok(frame)
}

pub(super) fn encode_plugin_settings(
    settings: RuntimePluginSettings, max_message_bytes: usize,
) -> Result<dokime::domain::model::settings::wire::PluginRuntimeSettings, TonicStatus> {
    let mut budget = Budget::new();
    value::collection(settings.settings.contributions.len())?;
    value::collection(settings.values.len())?;
    value::collection(settings.credential_bindings.bindings.len())?;
    value::collection(settings.credential_bindings.missing_required.len())?;
    let values = canonical::encode(&dokime::domain::model::values::Value::Object(settings.values), &mut budget, 0)?;
    let discovered = canonical::encode(&dokime::domain::model::values::Value::Object(settings.discovered), &mut budget, 0)?;
    let response = dokime::domain::model::settings::wire::PluginRuntimeSettings {
        settings: Some(encode_settings(settings.settings)?),
        values: Some(values),
        discovered: Some(discovered),
        credential_bindings: Some(encode_credentials(settings.credential_bindings, &mut budget)?),
    };
    ensure_size(&response, max_message_bytes, "plugin settings response")?;
    Ok(response)
}

pub(super) fn plugin_error(error: RuntimePluginManagementError) -> TonicStatus {
    let message = error.to_string();
    let error = match error {
        RuntimePluginManagementError::NotFound(_) => Error::not_found("installed_plugin_not_found", message),
        RuntimePluginManagementError::InvalidInput(_) => Error::validation("invalid_plugin_settings", message, vec![]),
        RuntimePluginManagementError::Store(_) => Error::internal("plugin_settings_store", message),
    };
    execution::status(error)
}

/* === Private Functions === */

fn encode_section(section: RuntimeSettingsSection) -> Result<dokime::domain::model::settings::wire::SettingsSection, TonicStatus> {
    let section_id = checked_identifier(&section.section_id, "section_id")?;
    bounded_text(section.label.clone(), "section label")?;
    let description = section.description.map(|description| bounded_text(description, "section description")).transpose()?;
    let owner = match section.owner {
        RuntimeSettingsSectionOwner::App => Some(dokime::domain::model::settings::wire::SettingsSectionOwner::App(true)),
        RuntimeSettingsSectionOwner::InstalledPlugin { plugin_id } => {
            Some(dokime::domain::model::settings::wire::SettingsSectionOwner::InstalledPluginId(checked_identifier(
                &plugin_id,
                "section owner plugin_id",
            )?))
        }
    };
    Ok(dokime::domain::model::settings::wire::SettingsSection {
        section_id,
        label: section.label,
        description,
        owner,
        order: section.order,
    })
}

fn encode_card(card: RuntimeSettingsCard, budget: &mut Budget) -> Result<dokime::domain::model::settings::wire::SettingsCard, TonicStatus> {
    let plugin_id = checked_identifier(&card.plugin_id, "card plugin_id")?;
    let card_id = checked_identifier(&card.card_id, "card_id")?;
    let section_id = checked_identifier(&card.section_id, "card section_id")?;
    bounded_text(card.label.clone(), "card label")?;
    let description = card.description.map(|description| bounded_text(description, "card description")).transpose()?;
    let body = encode_body(card.body, budget)?;
    let refresh = Some(encode_refresh(card.refresh));
    Ok(dokime::domain::model::settings::wire::SettingsCard {
        plugin_id,
        card_id,
        section_id,
        label: card.label,
        description,
        body: Some(body),
        refresh,
        state: encode_card_state(card.state),
        order: card.order,
    })
}

fn encode_body(body: HostCardBody, _budget: &mut Budget) -> Result<dokime::domain::model::settings::wire::HostCardBody, TonicStatus> {
    value::collection(body.blocks.len())?;
    Ok(dokime::domain::model::settings::wire::HostCardBody { blocks: body.blocks.into_iter().map(encode_block).collect::<Result<_, _>>()? })
}

fn encode_block(block: HostCardBlock) -> Result<dokime::domain::model::settings::wire::HostCardBlock, TonicStatus> {
    use dokime::domain::model::settings::wire::HostCardBlockKind as Kind;
    Ok(dokime::domain::model::settings::wire::HostCardBlock {
        kind: Some(match block {
            HostCardBlock::Text { text } => Kind::Text(bounded_text(text, "card text")?),
            HostCardBlock::Status { label, tone, detail } => Kind::Status(dokime::domain::model::settings::wire::HostCardStatus {
                label: bounded_text(label, "status label")?,
                tone: encode_tone(tone),
                detail: detail.map(|detail| bounded_text(detail, "status detail")).transpose()?,
            }),
            HostCardBlock::KeyValueList { items } => {
                value::collection(items.len())?;
                Kind::KeyValueList(dokime::domain::model::settings::wire::HostCardKeyValueList {
                    items: items.into_iter().map(encode_key_value).collect::<Result<_, _>>()?,
                })
            }
            HostCardBlock::Metrics { items } => {
                value::collection(items.len())?;
                Kind::Metrics(dokime::domain::model::settings::wire::HostCardMetrics {
                    items: items.into_iter().map(encode_metric).collect::<Result<_, _>>()?,
                })
            }
        }),
    })
}

fn encode_key_value(item: HostCardKeyValue) -> Result<dokime::domain::model::settings::wire::HostCardKeyValue, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::HostCardKeyValue {
        label: bounded_text(item.label, "key-value label")?,
        value: bounded_text(item.value, "key-value value")?,
        detail: item.detail.map(|detail| bounded_text(detail, "key-value detail")).transpose()?,
    })
}

fn encode_metric(item: HostCardMetric) -> Result<dokime::domain::model::settings::wire::HostCardMetric, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::HostCardMetric {
        label: bounded_text(item.label, "metric label")?,
        value: bounded_text(item.value, "metric value")?,
        unit: item.unit.map(|unit| bounded_text(unit, "metric unit")).transpose()?,
        trend: item.trend.map(|trend| bounded_text(trend, "metric trend")).transpose()?,
    })
}

fn encode_refresh(refresh: SettingsCardRefreshPolicy) -> dokime::domain::model::settings::wire::SettingsCardRefreshPolicy {
    use dokime::domain::model::settings::wire::SettingsCardRefreshPolicyPolicy as Policy;
    let policy = match refresh {
        SettingsCardRefreshPolicy::Manual => Policy::Manual(dokime::domain::model::settings::wire::ManualRefresh {}),
        SettingsCardRefreshPolicy::Cached { max_age_ms } => {
            Policy::Cached(dokime::domain::model::settings::wire::CachedRefresh { max_age_ms })
        }
        SettingsCardRefreshPolicy::OnDemand => Policy::OnDemand(dokime::domain::model::settings::wire::OnDemandRefresh {}),
    };
    dokime::domain::model::settings::wire::SettingsCardRefreshPolicy { policy: Some(policy) }
}

fn encode_tone(tone: HostCardTone) -> dokime::domain::model::settings::wire::HostCardTone {
    match tone {
        HostCardTone::Neutral => dokime::domain::model::settings::wire::HostCardTone::Neutral,
        HostCardTone::Info => dokime::domain::model::settings::wire::HostCardTone::Info,
        HostCardTone::Success => dokime::domain::model::settings::wire::HostCardTone::Success,
        HostCardTone::Warning => dokime::domain::model::settings::wire::HostCardTone::Warning,
        HostCardTone::Danger => dokime::domain::model::settings::wire::HostCardTone::Danger,
    }
}

fn encode_card_state(state: RuntimeSettingsCardState) -> dokime::domain::model::settings::wire::SettingsCardState {
    match state {
        RuntimeSettingsCardState::Ready => dokime::domain::model::settings::wire::SettingsCardState::Ready,
        RuntimeSettingsCardState::Disabled => dokime::domain::model::settings::wire::SettingsCardState::Disabled,
        RuntimeSettingsCardState::Disconnected => dokime::domain::model::settings::wire::SettingsCardState::Disconnected,
        RuntimeSettingsCardState::Stale => dokime::domain::model::settings::wire::SettingsCardState::Stale,
        RuntimeSettingsCardState::Error => dokime::domain::model::settings::wire::SettingsCardState::Error,
        RuntimeSettingsCardState::PermissionDenied => dokime::domain::model::settings::wire::SettingsCardState::PermissionDenied,
    }
}

fn encode_settings(settings: Settings) -> Result<dokime::domain::model::settings::wire::SettingsCatalog, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::SettingsCatalog {
        contributions: settings.contributions.into_iter().map(encode_contribution).collect::<Result<_, _>>()?,
    })
}

fn encode_contribution(contribution: SettingsEntry) -> Result<dokime::domain::model::settings::wire::SettingContribution, TonicStatus> {
    use dokime::domain::model::settings::wire::SettingContributionKind as Kind;
    let kind = match contribution {
        SettingsEntry::Boolean(value) => Kind::Boolean(encode_boolean(value)?),
        SettingsEntry::TextArea(value) => Kind::TextArea(encode_text_area(value)?),
        SettingsEntry::Path(value) => Kind::Path(encode_path(value)?),
        SettingsEntry::Number(value) => Kind::Number(encode_number(value)?),
        SettingsEntry::MultipleChoice(value) => Kind::MultipleChoice(encode_multiple_choice(value)?),
        SettingsEntry::SecretReference(value) => Kind::SecretReference(encode_secret(value)?),
    };
    Ok(dokime::domain::model::settings::wire::SettingContribution { kind: Some(kind) })
}

fn setting_texts(id: String, title: String, description: Option<String>) -> Result<(String, String, Option<String>), TonicStatus> {
    Ok((
        bounded_text(id, "setting id")?,
        bounded_text(title, "setting title")?,
        description.map(|description| bounded_text(description, "setting description")).transpose()?,
    ))
}

fn encode_boolean(value: Boolean) -> Result<dokime::domain::model::settings::wire::BooleanSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::BooleanSetting { id, title, description, default: value.default })
}

fn encode_text_area(value: TextArea) -> Result<dokime::domain::model::settings::wire::TextAreaSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::TextAreaSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "text-area default")?,
        required: value.required,
    })
}

fn encode_path(value: Path) -> Result<dokime::domain::model::settings::wire::PathSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::PathSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "path default")?,
        required: value.required,
    })
}

fn encode_number(value: Number) -> Result<dokime::domain::model::settings::wire::NumberSetting, TonicStatus> {
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
    Ok(dokime::domain::model::settings::wire::NumberSetting {
        id,
        title,
        description,
        default: value.default,
        min: value.min,
        max: value.max,
        step: value.step,
    })
}

fn encode_multiple_choice(value: MultipleChoice) -> Result<dokime::domain::model::settings::wire::MultipleChoiceSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    value::collection(value.choices.len())?;
    let provider = value.provider.map(|provider| bounded_text(provider, "choice provider")).transpose()?;
    Ok(dokime::domain::model::settings::wire::MultipleChoiceSetting {
        id,
        title,
        description,
        default: bounded_text(value.default, "choice default")?,
        choices: value.choices.into_iter().map(encode_choice).collect::<Result<_, _>>()?,
        provider,
    })
}

fn encode_choice(value: Choice) -> Result<dokime::domain::model::settings::wire::SettingChoice, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::SettingChoice {
        value: bounded_text(value.value, "choice value")?,
        title: bounded_text(value.title, "choice title")?,
    })
}

fn encode_secret(value: SecretSetting) -> Result<dokime::domain::model::settings::wire::SecretSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::SecretSetting {
        id,
        title,
        description,
        default: value.default.map(encode_secret_reference).transpose()?,
        required: value.required,
    })
}

fn encode_secret_reference(reference: SecretReference) -> Result<dokime::domain::model::settings::wire::SecretReference, TonicStatus> {
    use dokime::domain::model::settings::wire::SecretReferenceKind as Kind;
    let kind = match reference {
        SecretReference::Environment { variable } => Kind::Environment(bounded_text(variable, "environment variable")?),
        SecretReference::StoredCredential { credential_id } => Kind::StoredCredential(bounded_text(credential_id, "credential id")?),
    };
    Ok(dokime::domain::model::settings::wire::SecretReference { kind: Some(kind) })
}

fn encode_credentials(
    credentials: dokime::domains::installed_plugins::PluginRuntimeCredentialBindings, _budget: &mut Budget,
) -> Result<dokime::domain::model::settings::wire::PluginRuntimeCredentialBindings, TonicStatus> {
    value::collection(credentials.bindings.len())?;
    value::collection(credentials.missing_required.len())?;
    let bindings = credentials
        .bindings
        .into_iter()
        .map(|(key, binding)| {
            let key = bounded_text(key, "credential binding key")?;
            Ok(dokime::domain::model::settings::wire::PluginRuntimeCredentialBindingsBindingsEntry {
                key,
                value: Some(dokime::domain::model::settings::wire::RedactedCredentialBinding {
                    credential_id: bounded_text(binding.credential_id, "credential id")?,
                    backend: encode_credential_backend(binding.backend),
                    configured: binding.configured,
                }),
            })
        })
        .collect::<Result<Vec<_>, TonicStatus>>()?;
    Ok(dokime::domain::model::settings::wire::PluginRuntimeCredentialBindings {
        bindings,
        missing_required: credentials
            .missing_required
            .into_iter()
            .map(|id| bounded_text(id, "missing credential id"))
            .collect::<Result<_, _>>()?,
    })
}

fn encode_credential_backend(
    backend: dokime::domain::model::credentials::CredentialBackendKind,
) -> dokime::domain::model::settings::wire::CredentialBackendKind {
    match backend {
        dokime::domain::model::credentials::CredentialBackendKind::OsKeyring => {
            dokime::domain::model::settings::wire::CredentialBackendKind::OsKeyring
        }
        dokime::domain::model::credentials::CredentialBackendKind::Memory => {
            dokime::domain::model::settings::wire::CredentialBackendKind::Memory
        }
        dokime::domain::model::credentials::CredentialBackendKind::LocalFile => {
            dokime::domain::model::settings::wire::CredentialBackendKind::LocalFile
        }
    }
}

fn checked_identifier(value: &Identifier, field: &str) -> Result<String, TonicStatus> {
    if value.as_str().len() > MAX_TEXT_BYTES {
        return Err(TonicStatus::resource_exhausted(format!("{field} exceeds {MAX_TEXT_BYTES} bytes")));
    }
    Ok(value.as_str().to_owned())
}

fn ensure_size<M: Portable>(message: &M, max_message_bytes: usize, name: &str) -> Result<(), TonicStatus> {
    if message.encoded_len() > max_message_bytes {
        return Err(TonicStatus::resource_exhausted(format!("{name} exceeds the native message budget")));
    }
    Ok(())
}

/* === Tests === */
