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
    let after = request.after.as_ref().map(|cursor| cursor.after_plugin_id.as_str());
    let size = dokime::domain::model::execution::wire_cursor::page_size(request.page_size, after.unwrap_or_default())
        .map_err(crate::grpc::value::canonical::cursor_failure)?;
    let snapshot = runtime.watch_state().latest();
    let state = snapshot.state();
    let epoch = state.process_epoch().to_string();
    if let Some(after) = &request.after
        && (after.epoch != epoch || after.revision != snapshot.revision())
    {
        return Err(Status::failed_precondition("plugin inventory changed; restart listing"));
    }
    if after.is_some_and(|id| !state.installed_plugins.contains_key(id)) {
        return Err(Status::invalid_argument("plugin cursor is not in inventory"));
    }
    let start = after.map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded);
    let mut selected = state.installed_plugins.range::<str, _>((start, std::ops::Bound::Unbounded));
    let mut artifacts = Vec::new();
    for (id, artifact) in selected.by_ref().take(size) {
        let owner = state.plugin_owners.get(id);
        let label = match owner {
            Some(PluginOwner::Bundle { package, .. }) => package.as_str(),
            Some(PluginOwner::Host) => "Dokime",
            _ => id.as_str(),
        };
        artifacts.try_reserve(1).map_err(|_| Status::resource_exhausted("plugin page allocation refused"))?;
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
