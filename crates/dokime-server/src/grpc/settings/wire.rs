/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::grpc::transcode::Transcode;

use dokime::harmos::domain::Portable;
use tonic::Status as TonicStatus;

use super::super::{
    execution,
    value::{Budget, canonical},
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

pub(super) fn identifier(value: String, field: &str) -> Result<Identifier, TonicStatus> {
    Identifier::new(value).map_err(|error| TonicStatus::invalid_argument(format!("{field}: {error}")))
}

pub(super) fn encode_frame(
    frame: RuntimeSettingsFrame, max_message_bytes: usize,
) -> Result<dokime::domain::model::settings::wire::SettingsFrame, TonicStatus> {
    let mut budget = Budget::new()?;
    let sections = frame.sections.into_iter().map(encode_section).collect::<Result<_, _>>()?;
    let cards = frame.cards.into_iter().map(|card| encode_card(card, &mut budget)).collect::<Result<_, _>>()?;
    let frame = dokime::domain::model::settings::wire::SettingsFrame {
        bootstrap_paths: frame.bootstrap_paths,
        sections,
        cards,
    };
    ensure_size(&frame, max_message_bytes, "settings frame")?;
    Ok(frame)
}

pub(super) fn encode_plugin_settings(
    settings: RuntimePluginSettings, max_message_bytes: usize,
) -> Result<dokime::domain::model::settings::wire::PluginRuntimeSettings, TonicStatus> {
    let mut budget = Budget::new()?;
    let values = canonical::encode(&dokime::domain::model::values::Value::Object(settings.values), &mut budget, 0)?;
    let discovered = canonical::encode(&dokime::domain::model::values::Value::Object(settings.discovered), &mut budget, 0)?;
    let response = dokime::domain::model::settings::wire::PluginRuntimeSettings {
        settings: Some(encode_settings(settings.settings)?),
        values: Some(values),
        discovered: Some(discovered),
        secret_bindings: Some(encode_secrets(settings.secret_bindings, &mut budget)?),
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

fn encode_section(section: RuntimeSettingsSection) -> Result<dokime::domain::model::settings::wire::SettingsSection, TonicStatus> {
    let section_id = checked_identifier(&section.section_id, "section_id")?;
    let description = section.description;
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
    let description = card.description;
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
    Ok(dokime::domain::model::settings::wire::HostCardBody {
        blocks: body.blocks.into_iter().map(encode_block).collect::<Result<_, _>>()?,
    })
}

fn encode_block(block: HostCardBlock) -> Result<dokime::domain::model::settings::wire::HostCardBlock, TonicStatus> {
    use dokime::domain::model::settings::wire::HostCardBlockKind as Kind;
    Ok(dokime::domain::model::settings::wire::HostCardBlock {
        kind: Some(match block {
            HostCardBlock::Text { text } => Kind::Text(text),
            HostCardBlock::Status { label, tone, detail } => {
                Kind::Status(dokime::domain::model::settings::wire::HostCardStatus { label, tone: encode_tone(tone), detail })
            }
            HostCardBlock::KeyValueList { items } => Kind::KeyValueList(dokime::domain::model::settings::wire::HostCardKeyValueList {
                items: items.into_iter().map(encode_key_value).collect::<Result<_, _>>()?,
            }),
            HostCardBlock::Metrics { items } => Kind::Metrics(dokime::domain::model::settings::wire::HostCardMetrics {
                items: items.into_iter().map(encode_metric).collect::<Result<_, _>>()?,
            }),
        }),
    })
}

fn encode_key_value(item: HostCardKeyValue) -> Result<dokime::domain::model::settings::wire::HostCardKeyValue, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::HostCardKeyValue {
        label: item.label,
        value: item.value,
        detail: item.detail,
    })
}

fn encode_metric(item: HostCardMetric) -> Result<dokime::domain::model::settings::wire::HostCardMetric, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::HostCardMetric {
        label: item.label,
        value: item.value,
        unit: item.unit,
        trend: item.trend,
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
    Ok((id, title, description))
}

fn encode_boolean(value: Boolean) -> Result<dokime::domain::model::settings::wire::BooleanSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::BooleanSetting {
        id,
        title,
        description,
        default: value.default,
    })
}

fn encode_text_area(value: TextArea) -> Result<dokime::domain::model::settings::wire::TextAreaSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::TextAreaSetting {
        id,
        title,
        description,
        default: value.default,
        required: value.required,
    })
}

fn encode_path(value: Path) -> Result<dokime::domain::model::settings::wire::PathSetting, TonicStatus> {
    let (id, title, description) = setting_texts(value.id, value.title, value.description)?;
    Ok(dokime::domain::model::settings::wire::PathSetting {
        id,
        title,
        description,
        default: value.default,
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
    let provider = value.provider;
    Ok(dokime::domain::model::settings::wire::MultipleChoiceSetting {
        id,
        title,
        description,
        default: value.default,
        choices: value.choices.into_iter().map(encode_choice).collect::<Result<_, _>>()?,
        provider,
    })
}

fn encode_choice(value: Choice) -> Result<dokime::domain::model::settings::wire::SettingChoice, TonicStatus> {
    Ok(dokime::domain::model::settings::wire::SettingChoice { value: value.value, title: value.title })
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
        SecretReference::Environment { variable } => Kind::Environment(variable),
        SecretReference::StoredSecret { secret_id } => Kind::StoredSecret(secret_id),
    };
    Ok(dokime::domain::model::settings::wire::SecretReference { kind: Some(kind) })
}

fn encode_secrets(
    secrets: dokime::domains::installed_plugins::PluginRuntimeSecretBindings, _budget: &mut Budget,
) -> Result<dokime::domain::model::settings::wire::PluginRuntimeSecretBindings, TonicStatus> {
    let bindings = secrets
        .bindings
        .into_iter()
        .map(|(key, binding)| {
            Ok(dokime::domain::model::settings::wire::PluginRuntimeSecretBindingsBindingsEntry {
                key,
                value: Some(dokime::domain::model::settings::wire::RedactedSecretBinding {
                    secret_id: binding.secret_id,
                    backend: encode_secret_backend(binding.backend),
                    configured: binding.configured,
                }),
            })
        })
        .collect::<Result<Vec<_>, TonicStatus>>()?;
    Ok(dokime::domain::model::settings::wire::PluginRuntimeSecretBindings {
        bindings,
        missing_required: secrets.missing_required.into_iter().collect(),
    })
}

fn encode_secret_backend(
    backend: dokime::domain::model::secrets::SecretBackendKind,
) -> dokime::domain::model::settings::wire::SecretBackendKind {
    match backend {
        dokime::domain::model::secrets::SecretBackendKind::OsKeyring => dokime::domain::model::settings::wire::SecretBackendKind::OsKeyring,
    }
}

fn checked_identifier(value: &Identifier, _field: &str) -> Result<String, TonicStatus> {
    Ok(value.as_str().to_owned())
}

fn ensure_size<M: Portable>(message: &M, max_message_bytes: usize, name: &str) -> Result<(), TonicStatus> {
    if message.wire_len()? > max_message_bytes {
        return Err(TonicStatus::resource_exhausted(format!("{name} exceeds the native message budget")));
    }
    Ok(())
}
