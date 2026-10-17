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

// Local Imports
use super::super::value::canonical;
use super::{contract as v1, value};
use dokime_runtime::plugins::{PluginEntry, PluginLastBoot, PluginState};
use harmos::Protobuf;
use tonic::Status;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn entry(
    mut entry: PluginEntry,
    limit: usize,
) -> Result<v1::PluginInstallation, Status> {
    normalize(&mut entry, limit)?;
    let installation = entry.installation;
    let metadata = entry.metadata.map(|metadata| v1::PluginMetadata {
        identity: Some(v1::PluginIdentity {
            name: metadata.identity.name,
            version: metadata.identity.version,
            description: metadata.identity.description,
        }),
        owners: metadata.owners,
        size_bytes: metadata.size_bytes,
        environment_references: metadata
            .environment_references
            .into_iter()
            .map(|reference| v1::PluginEnvironmentReference {
                owner: reference.owner,
                setting: reference.setting,
                variable: reference.variable,
            })
            .collect(),
    });
    let last_boot = installation
        .last_boot
        .map(|boot| {
            let (at, result) = match boot {
                PluginLastBoot::Loaded { at } => (at, v1::PluginLastBootResult::Loaded(true)),
                PluginLastBoot::Failed { error, at } => {
                    (at, v1::PluginLastBootResult::Error(error))
                }
            };
            Ok::<_, Status>(v1::PluginLastBoot {
                at: Some(canonical::timestamp(&at)?),
                result: Some(result),
            })
        })
        .transpose()?;
    let result = v1::PluginInstallation {
        install_id: installation.install_id.to_string(),
        sha256: installation.sha256,
        staged_at: Some(canonical::timestamp(&installation.staged_at)?),
        consented_at: installation
            .consented_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        state: match installation.state {
            PluginState::Staged => v1::PluginInstallationState::Staged,
            PluginState::Installed => v1::PluginInstallationState::Installed,
            PluginState::Disabled => v1::PluginInstallationState::Disabled,
            PluginState::RemovalPending => v1::PluginInstallationState::RemovalPending,
        },
        last_boot,
        metadata,
        error: entry.error,
        active: entry.active,
        restart_required: entry.restart_required,
        boot_error: entry.boot_error,
    };
    if result.encoded_len() > limit {
        return Err(Status::resource_exhausted(
            "plugin metadata exceeds response budget",
        ));
    }
    Ok(result)
}

pub(super) fn disclosure(entry: &PluginEntry, limit: usize) -> Result<(), Status> {
    if entry.metadata.is_none() {
        return Err(Status::failed_precondition(
            "verified permission metadata is unavailable; consent refused",
        ));
    }
    dokime_runtime::document::bounded::document_size(entry, limit / 2, 32).map_err(|_| {
        Status::resource_exhausted(
            "complete permission metadata exceeds response budget; consent refused",
        )
    })?;
    Ok(())
}

pub(super) fn normalize(entry: &mut PluginEntry, limit: usize) -> Result<(), Status> {
    if entry.metadata.is_some() && disclosure(entry, limit).is_err() {
        entry.metadata = None;
        entry.error = Some("Verified permission metadata exceeds response budget; consent is unavailable. Installation identity and state remain accessible.".into());
    }
    dokime_runtime::document::bounded::document_size(&entry, limit / 2, 32).map_err(|_| {
        Status::resource_exhausted("plugin installation state exceeds response budget")
    })?;
    Ok(())
}
pub(super) fn memory(entry: &PluginEntry) -> Result<usize, Status> {
    let mut bytes = std::mem::size_of::<v1::PluginInstallation>() + 36;
    let mut add = |text: &str| -> Result<(), Status> {
        value::text(text)?;
        bytes = bytes.saturating_add(text.len());
        Ok(())
    };
    add(&entry.installation.sha256)?;
    if let Some(text) = &entry.error {
        add(text)?;
    }
    if let Some(text) = &entry.boot_error {
        add(text)?;
    }
    if let Some(PluginLastBoot::Failed { error, .. }) = &entry.installation.last_boot {
        add(error)?;
    }
    if let Some(metadata) = &entry.metadata {
        for text in [
            &metadata.identity.name,
            &metadata.identity.version,
            &metadata.identity.description,
        ] {
            add(text)?;
        }
        for text in &metadata.owners {
            add(text)?;
        }
        for reference in &metadata.environment_references {
            for text in [&reference.owner, &reference.setting, &reference.variable] {
                add(text)?;
            }
        }
    }
    if let Some(metadata) = &entry.metadata {
        bytes = bytes
            .saturating_add(
                metadata
                    .owners
                    .len()
                    .saturating_mul(4 * std::mem::size_of::<String>()),
            )
            .saturating_add(
                metadata
                    .environment_references
                    .len()
                    .saturating_mul(4 * std::mem::size_of::<v1::PluginEnvironmentReference>()),
            );
    }
    Ok(bytes)
}

/* === Private Functions === */

/* === Tests === */
