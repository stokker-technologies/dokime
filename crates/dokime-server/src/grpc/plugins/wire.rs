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
use tonic::Status;

// Local Imports
use super::super::value::canonical;
use crate::grpc::transcode::Transcode;
use dokime::domain::model::plugins::wire as plugins_model;
use dokime::plugins::{PluginEntry, PluginLastBoot, PluginState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn entry(mut entry: PluginEntry, limit: usize) -> Result<plugins_model::PluginInstallation, Status> {
    normalize(&mut entry, limit)?;
    let installation = entry.installation;
    let metadata = entry.metadata.map(|metadata| plugins_model::PluginMetadata {
        identity: Some(plugins_model::PluginIdentity {
            name: metadata.identity.name,
            version: metadata.identity.version,
            description: metadata.identity.description,
        }),
        owners: metadata.owners,
        size_bytes: metadata.size_bytes,
    });
    let last_boot = installation
        .last_boot
        .map(|boot| {
            let (at, result, failure) = match boot {
                PluginLastBoot::Loaded { at } => (at, plugins_model::PluginLastBootResult::Loaded(true), None),
                PluginLastBoot::Failed { error, failure, at } => (at, plugins_model::PluginLastBootResult::Error(error), failure),
            };
            Ok::<_, Status>(plugins_model::PluginLastBoot {
                at: Some(canonical::timestamp(&at)?),
                result: Some(result),
                failure,
            })
        })
        .transpose()?;
    let result = plugins_model::PluginInstallation {
        install_id: installation.install_id.to_string(),
        sha256: installation.sha256,
        installed_at: Some(canonical::timestamp(&installation.installed_at)?),
        state: match installation.state {
            PluginState::Installed => plugins_model::PluginInstallationState::Installed,
            PluginState::Disabled => plugins_model::PluginInstallationState::Disabled,
            PluginState::RemovalPending => plugins_model::PluginInstallationState::RemovalPending,
        },
        last_boot,
        metadata,
        error: entry.error,
        active: entry.active,
        restart_required: entry.restart_required,
        boot_error: entry.boot_error,
    };
    if result.wire_len()? > limit {
        return Err(Status::resource_exhausted("plugin metadata exceeds response budget"));
    }
    Ok(result)
}

fn metadata_budget(entry: &PluginEntry, limit: usize) -> Result<(), Status> {
    if entry.metadata.is_none() {
        return Err(Status::failed_precondition("verified plugin metadata is unavailable"));
    }
    dokime::document::bounded::document_size(entry, limit, 32)
        .map_err(|_| Status::resource_exhausted("plugin metadata exceeds response budget"))?;
    Ok(())
}

pub(super) fn normalize(entry: &mut PluginEntry, limit: usize) -> Result<(), Status> {
    if entry.metadata.is_some() && metadata_budget(entry, limit).is_err() {
        entry.metadata = None;
        entry.error = Some("Plugin metadata exceeds response budget. Installation identity and state remain accessible.".into());
    }
    dokime::document::bounded::document_size(&entry, limit, 32)
        .map_err(|_| Status::resource_exhausted("plugin installation state exceeds response budget"))?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
