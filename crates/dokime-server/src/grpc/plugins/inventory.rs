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
use crate::grpc::transcode::Transcode;
use dokime::domain::model::plugins::wire as plugins_model;
use dokime::state::PluginOwner;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn page(
    runtime: &RuntimeHandle, request: &mut v1::ListAdmittedPlugins, limit: usize, local_only: bool,
) -> Result<dokime::harmos::operations::Response<plugins_model::AdmittedPluginPage>, Status> {
    let size = page_size(request.page_size)?;
    let snapshot = runtime.watch_state().latest();
    let state = snapshot.state();
    let epoch = state.process_epoch().to_string();
    if let Some(after) = &request.after
        && (after.epoch != epoch || after.revision != snapshot.revision())
    {
        return Err(Status::failed_precondition("plugin inventory changed; restart listing"));
    }
    let after = request.after.as_ref().map(|cursor| cursor.after_plugin_id.as_str());
    if after.is_some_and(|id| !state.installed_plugins.contains_key(id)) {
        return Err(Status::invalid_argument("plugin cursor is not in inventory"));
    }
    let start = after.map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded);
    let mut selected = state.installed_plugins.range::<str, _>((start, std::ops::Bound::Unbounded));
    let mut artifacts = Vec::new();
    let mut bytes = 256usize;
    for (id, artifact) in selected.by_ref().take(size) {
        let owner = state.plugin_owners.get(id);
        let label = match owner {
            Some(PluginOwner::Bundle { package, .. }) => package.as_str(),
            Some(PluginOwner::Host) => "Dokime",
            _ => id.as_str(),
        };
        let encoded = id.len().saturating_add(label.len()).saturating_add(artifact.version.len()).saturating_add(64);
        let owner_bytes = match owner {
            Some(PluginOwner::Bundle { package, path, installation, content_digest }) => package
                .len()
                .saturating_add(path.len())
                .saturating_add(installation.as_ref().map_or(0, String::len))
                .saturating_add(content_digest.len()),
            Some(PluginOwner::Embedded { generation }) => generation.len(),
            _ => 0,
        };
        bytes = bytes.saturating_add(encoded).saturating_add(owner_bytes);
        if bytes > limit {
            return Err(Status::resource_exhausted("plugin page exceeds response budget"));
        }
        artifacts.push(plugins_model::AdmittedPlugin {
            plugin_id: id.clone(),
            label: label.to_owned(),
            version: artifact.version.clone(),
            enabled: artifact.enabled,
            owner: owner.map(|owner| match owner {
                PluginOwner::Host => plugins_model::AdmittedPluginOwner::Host(true),
                PluginOwner::Embedded { generation } => {
                    plugins_model::AdmittedPluginOwner::Embedded(plugins_model::AdmittedEmbeddedOwner { generation: generation.clone() })
                }
                PluginOwner::Bundle { package, path, installation, content_digest } => {
                    plugins_model::AdmittedPluginOwner::Bundle(plugins_model::AdmittedBundleOwner {
                        package: package.clone(),
                        path: local_only.then(|| path.clone()),
                        installation_id: installation.clone(),
                        content_digest: content_digest.clone(),
                    })
                }
            }),
        });
    }
    let next = selected.next().and_then(|_| {
        artifacts.last().map(|artifact| plugins_model::AdmittedPluginCursor {
            epoch: epoch.clone(),
            revision: snapshot.revision(),
            after_plugin_id: artifact.plugin_id.clone(),
        })
    });
    let reply = plugins_model::AdmittedPluginPage {
        plugins: artifacts,
        next,
        total_count: state.installed_plugins.len() as u64,
        epoch,
        revision: snapshot.revision(),
    };
    if reply.wire_len()? > limit {
        return Err(Status::resource_exhausted("plugin inventory exceeds wire budget"));
    }
    Ok(reply.into())
}

/* === Private Functions === */

/* === Tests === */
