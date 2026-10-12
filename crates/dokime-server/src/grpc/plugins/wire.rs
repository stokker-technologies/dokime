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
use super::*;
use dokime_app::plugins::{PluginEntry, PluginLastBoot, PluginState};

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
    if entry.metadata.is_some() && disclosure(&entry, limit).is_err() {
        entry.metadata = None;
        entry.error = Some("Verified permission metadata exceeds response budget; consent is unavailable. Installation identity and state remain accessible.".into());
    }
    dokime_app::document::bounded::document_size(&entry, limit / 2, 32).map_err(|_| {
        Status::resource_exhausted("plugin installation state exceeds response budget")
    })?;
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
                PluginLastBoot::Loaded { at } => (at, v1::plugin_last_boot::Result::Loaded(true)),
                PluginLastBoot::Failed { error, at } => {
                    (at, v1::plugin_last_boot::Result::Error(error))
                }
            };
            Ok::<_, Status>(v1::PluginLastBoot {
                at: Some(value::timestamp(&at)?),
                result: Some(result),
            })
        })
        .transpose()?;
    let result = v1::PluginInstallation {
        install_id: installation.install_id.to_string(),
        sha256: installation.sha256,
        staged_at: Some(value::timestamp(&installation.staged_at)?),
        consented_at: installation
            .consented_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        state: match installation.state {
            PluginState::Staged => 1,
            PluginState::Installed => 2,
            PluginState::Disabled => 3,
            PluginState::RemovalPending => 4,
        },
        last_boot,
        metadata,
        error: entry.error,
        active: entry.active,
        restart_required: entry.restart_required,
        boot_error: entry.boot_error,
    };
    if prost::Message::encoded_len(&result) > limit {
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
    dokime_app::document::bounded::document_size(entry, limit / 2, 32).map_err(|_| {
        Status::resource_exhausted(
            "complete permission metadata exceeds response budget; consent refused",
        )
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
