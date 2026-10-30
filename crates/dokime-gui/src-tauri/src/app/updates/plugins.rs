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
use serde::Serialize;
use tauri::{State, WebviewWindow, ipc::Channel};
use url::Url;

// Local Imports
use super::{UpdateState, sources};
use crate::app::plugins::{self, InstallProgress};
use crate::connection::ConnectionSession;
use dokime::{
    distribution,
    domain::model::plugins::release::{Feed, Member, Plugin},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Candidate {
    id: String,
    generation: u64,
    connection: String,
    plugin: Plugin,
    member: Member,
    archive_url: Url,
    archive_sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub candidate: String,
    pub package: String,
    pub version: String,
    pub target: String,
    pub requires_dokime: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn desktop_plugin_release_check(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, state: State<'_, UpdateState>, generation: String,
) -> Result<Vec<Release>, String> {
    let lease = plugins::admit(&window, &connection, &generation)?;
    let (policy, policy_generation) = state.snapshot()?;
    let source = sources::release_source(&policy, "plugins.json").await?;
    let bytes = sources::fetch(&source.feed, sources::DESKTOP_UPDATE_MANIFEST_BYTES as usize).await?;
    let sums_url = source.asset("SHA256SUMS")?;
    let sums_bytes = sources::fetch(&sums_url, sources::DESKTOP_UPDATE_MANIFEST_BYTES as usize).await?;
    let sums = distribution::checksums(&sums_bytes)?;
    let feed_hash = sums.get("plugins.json").ok_or("SHA256SUMS has no plugins.json entry")?;
    distribution::verify(&bytes, feed_hash)?;
    let feed = Feed::parse(&bytes)?;

    let client = lease.client();
    let backend = client.backend();
    let mut releases = Vec::new();
    let mut candidates = Vec::new();
    for plugin in feed.plugins {
        let Ok(member) = plugin.select(&backend.product_version, &backend.target).cloned() else { continue };
        let archive_url = source.asset(&member.archive)?;
        let archive_sha256 = sums.get(&member.archive).cloned().ok_or("SHA256SUMS has no plugin archive entry")?;
        let id = format!("{:032x}", rand::random::<u128>());
        releases.push(Release {
            candidate: id.clone(),
            package: plugin.name.clone(),
            version: plugin.version.clone(),
            target: backend.target.clone(),
            requires_dokime: plugin.requires_dokime.clone(),
        });
        candidates.push(Candidate {
            id,
            generation: policy_generation,
            connection: generation.clone(),
            plugin,
            member,
            archive_url,
            archive_sha256,
        });
    }
    if releases.is_empty() {
        return Err("The plugin release feed has no compatible plugins for this backend".into());
    }

    let mut inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
    if inner.generation != policy_generation {
        return Err("The plugin release source changed while checking".into());
    }
    inner.plugin = candidates;
    Ok(releases)
}

pub async fn desktop_plugin_release_install(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, state: State<'_, UpdateState>, generation: String, candidate: String,
    progress: Channel<InstallProgress>,
) -> Result<String, String> {
    let lease = plugins::admit(&window, &connection, &generation)?;
    let selected = {
        let inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        inner
            .plugin
            .iter()
            .find(|item| item.id == candidate && item.connection == generation && item.generation == inner.generation)
            .cloned()
            .ok_or("The selected plugin release is no longer available")?
    };
    let (_, policy_generation) = state.snapshot()?;
    if selected.generation != policy_generation {
        return Err("The plugin release source changed".into());
    }
    let client = lease.client();
    selected.plugin.select(&client.backend().product_version, &client.backend().target)?;
    let archive = sources::fetch(&selected.archive_url, distribution::ARCHIVE_MAX_BYTES).await?;
    let expected = selected.member.sha256.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || validate_release_archive(&archive, &selected))
        .await
        .map_err(|error| error.to_string())??;

    if state.snapshot()?.1 != policy_generation {
        return Err("The plugin release source changed while inspecting the download".into());
    }
    let installed = plugins::install(&lease, bytes, progress, Some(&expected)).await?;
    Ok(installed.install_id)
}

/* === Private Functions === */

fn validate_release_archive(bytes: &[u8], selected: &Candidate) -> Result<Vec<u8>, String> {
    distribution::verify(bytes, &selected.archive_sha256)?;
    distribution::plugin(bytes, &selected.member, &selected.plugin)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use harmos_packaging::bundle::{Bundle, Identity, Limits, Source};
    use std::io::{Cursor, Write};

    #[test]
    fn release_archive_integrity() {
        let bundle = Bundle::build(
            Identity {
                name: "instruments".into(),
                version: "1.2.3".parse().unwrap(),
                description: String::new(),
                package: Default::default(),
            },
            &[Source {
                path: "resource.txt",
                bytes: b"verified resource",
                executable: None,
            }],
            Limits::default(),
        )
        .unwrap();
        let member = Member {
            archive: "sidecars.zip".into(),
            member: "instruments.dokime".into(),
            sha256: distribution::digest(bundle.bytes()),
        };
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.start_file(&member.member, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(bundle.bytes()).unwrap();
        writer.start_file("unselected.dokime", zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(b"unselected").unwrap();
        let archive = writer.finish().unwrap().into_inner();
        let plugin = Plugin {
            name: "instruments".into(),
            kind: dokime::domain::model::plugins::release::Kind::Sidecar,
            version: "1.2.3".into(),
            requires_dokime: "^0.1".into(),
            targets: [("native".into(), member.clone())].into(),
        };
        let mut selected = Candidate {
            id: "candidate".into(),
            generation: 1,
            connection: "connection".into(),
            plugin,
            member,
            archive_url: "https://example.test/sidecars.zip".parse().unwrap(),
            archive_sha256: distribution::digest(&archive),
        };

        assert_eq!(validate_release_archive(&archive, &selected).unwrap(), bundle.bytes());
        selected.member.sha256 = "0".repeat(64);
        assert!(validate_release_archive(&archive, &selected).is_err());
        selected.member.sha256 = distribution::digest(bundle.bytes());
        selected.plugin.name = "other".into();
        assert!(validate_release_archive(&archive, &selected).is_err());
        selected.plugin.name = "instruments".into();
        selected.archive_sha256 = "0".repeat(64);
        assert!(validate_release_archive(&archive, &selected).is_err());
    }
}
