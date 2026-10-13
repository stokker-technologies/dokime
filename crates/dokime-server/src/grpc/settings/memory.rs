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
use std::mem::size_of;
// Third Party Imports
use tonic::Status;
// Local Imports
use crate::grpc::value::{self, Budget, canonical};
use dokime_app::{
    capabilities::{
        installed_plugins::RuntimePluginSettings,
        settings::{RuntimeSettingsFrame, RuntimeSettingsSectionOwner},
    },
    core::HostCardBlock,
    settings::{SecretReference, SettingsEntry},
};
use dokime_contract::operations::settings as wire;
/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
pub(super) fn frame(frame: &RuntimeSettingsFrame) -> Result<usize, Status> {
    let mut bytes = size_of::<wire::SettingsFrame>()
        .saturating_add(slots::<wire::SettingsSection>(frame.sections.len())?)
        .saturating_add(slots::<wire::SettingsCard>(frame.cards.len())?);
    for section in &frame.sections {
        bytes = bytes
            .saturating_add(text(section.section_id.as_str())?)
            .saturating_add(owned(&section.label)?)
            .saturating_add(optional(&section.description)?);
        if let RuntimeSettingsSectionOwner::InstalledPlugin { plugin_id } = &section.owner {
            bytes = bytes.saturating_add(text(plugin_id.as_str())?);
        }
    }
    for card in &frame.cards {
        bytes = bytes
            .saturating_add(text(card.plugin_id.as_str())?)
            .saturating_add(text(card.card_id.as_str())?)
            .saturating_add(text(card.section_id.as_str())?)
            .saturating_add(owned(&card.label)?)
            .saturating_add(optional(&card.description)?)
            .saturating_add(slots::<wire::HostCardBlock>(card.body.blocks.len())?);
        for block in &card.body.blocks {
            bytes = bytes.saturating_add(match block {
                HostCardBlock::Text { text } => owned(text)?,
                HostCardBlock::Status { label, detail, .. } => {
                    owned(label)?.saturating_add(optional(detail)?)
                }
                HostCardBlock::KeyValueList { items } => {
                    let mut heap = slots::<wire::HostCardKeyValue>(items.len())?;
                    for item in items {
                        heap = heap
                            .saturating_add(owned(&item.label)?)
                            .saturating_add(owned(&item.value)?)
                            .saturating_add(optional(&item.detail)?);
                    }
                    heap
                }
                HostCardBlock::Metrics { items } => {
                    let mut heap = slots::<wire::HostCardMetric>(items.len())?;
                    for item in items {
                        heap = heap
                            .saturating_add(owned(&item.label)?)
                            .saturating_add(owned(&item.value)?)
                            .saturating_add(optional(&item.unit)?)
                            .saturating_add(optional(&item.trend)?);
                    }
                    heap
                }
            });
        }
    }
    bounded(bytes)
}
pub(super) fn plugin(settings: &RuntimePluginSettings) -> Result<usize, Status> {
    let mut budget = Budget::new();
    let mut bytes = size_of::<wire::PluginRuntimeSettings>()
        .saturating_add(slots::<wire::SettingContribution>(
            settings.settings.contributions.len(),
        )?)
        .saturating_add(canonical::object_memory(&settings.values, &mut budget)?)
        .saturating_add(canonical::object_memory(&settings.discovered, &mut budget)?);
    for entry in &settings.settings.contributions {
        let (id, title, description) = match entry {
            SettingsEntry::Boolean(v) => (&v.id, &v.title, &v.description),
            SettingsEntry::TextArea(v) => (&v.id, &v.title, &v.description),
            SettingsEntry::Path(v) => (&v.id, &v.title, &v.description),
            SettingsEntry::Number(v) => (&v.id, &v.title, &v.description),
            SettingsEntry::MultipleChoice(v) => (&v.id, &v.title, &v.description),
            SettingsEntry::SecretReference(v) => (&v.id, &v.title, &v.description),
        };
        bytes = bytes
            .saturating_add(owned(id)?)
            .saturating_add(owned(title)?)
            .saturating_add(optional(description)?);
        bytes = bytes.saturating_add(match entry {
            SettingsEntry::Boolean(_) => 0,
            SettingsEntry::TextArea(v) => owned(&v.default)?,
            SettingsEntry::Path(v) => owned(&v.default)?,
            SettingsEntry::Number(v) => {
                if [Some(v.default), v.min, v.max, v.step]
                    .into_iter()
                    .flatten()
                    .any(|n| !n.is_finite())
                {
                    return Err(Status::internal("number setting is not finite"));
                }
                0
            }
            SettingsEntry::MultipleChoice(v) => {
                let mut heap = owned(&v.default)?
                    .saturating_add(optional(&v.provider)?)
                    .saturating_add(slots::<wire::SettingChoice>(v.choices.len())?);
                for choice in &v.choices {
                    heap = heap
                        .saturating_add(owned(&choice.value)?)
                        .saturating_add(owned(&choice.title)?);
                }
                heap
            }
            SettingsEntry::SecretReference(v) => match &v.default {
                Some(SecretReference::Environment { variable }) => owned(variable)?,
                Some(SecretReference::StoredCredential { credential_id }) => owned(credential_id)?,
                None => 0,
            },
        });
    }
    bytes = bytes
        .saturating_add(slots::<wire::PluginRuntimeCredentialBindingsBindingsEntry>(
            settings.credential_bindings.bindings.len(),
        )?)
        .saturating_add(slots::<String>(
            settings.credential_bindings.missing_required.len(),
        )?);
    for (key, binding) in &settings.credential_bindings.bindings {
        bytes = bytes
            .saturating_add(owned(key)?)
            .saturating_add(owned(&binding.credential_id)?);
    }
    for id in &settings.credential_bindings.missing_required {
        bytes = bytes.saturating_add(owned(id)?);
    }
    bounded(bytes)
}
/* === Private Functions === */
fn slots<T>(len: usize) -> Result<usize, Status> {
    value::collection(len)?;
    // Result Collection May Grow Its Vector Before The Iterator Finishes.
    bounded(len.saturating_mul(4).saturating_mul(size_of::<T>()))
}
fn text(text: &str) -> Result<usize, Status> {
    value::text(text)?;
    Ok(text.len())
}
fn owned(text: &String) -> Result<usize, Status> {
    value::text(text)?;
    Ok(text.capacity())
}
fn optional(text: &Option<String>) -> Result<usize, Status> {
    text.as_ref().map(owned).transpose().map(|n| n.unwrap_or(0))
}
fn bounded(bytes: usize) -> Result<usize, Status> {
    if bytes == usize::MAX {
        Err(Status::resource_exhausted(
            "settings response memory overflows",
        ))
    } else {
        Ok(bytes)
    }
}
/* === Tests === */
