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
use std::collections::BTreeSet;

// Third Party Imports
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow, ipc::Channel};

// Local Imports
use super::{UpdateState, sources};
use crate::app::{
    organization::policy::https_url,
    plugins::{self, StageProgress},
};
use crate::connection::ConnectionSession;
use dokime::client::contracts;

/* === Constants === */

const DESKTOP_ORGANIZATION_IDENTIFIER_BYTES: u64 = 128;
const DESKTOP_ORGANIZATION_URL_BYTES: u64 = 4_096;
const DESKTOP_PLUGIN_BUNDLE_BYTES: u64 = 285_212_672;
const DESKTOP_PLUGIN_RELEASE_ASSETS: u64 = 32;
const DESKTOP_PLUGIN_RELEASE_VERSION_BYTES: u64 = 256;
const DESKTOP_UPDATE_MANIFEST_BYTES: u64 = 1_048_576;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Candidate {
    id: String,
    generation: u64,
    connection: String,
    manifest: Manifest,
    asset: Asset,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format_version: u32,
    package: String,
    version: String,
    requires_dokime: String,
    assets: Vec<Asset>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    target: String,
    url: String,
    sha256: String,
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

impl Manifest {
    fn select(&self, host_version: &str, target: &str) -> Result<Asset, String> {
        let package_valid = !self.package.is_empty()
            && self.package.len() as u64 <= DESKTOP_ORGANIZATION_IDENTIFIER_BYTES
            && self.package.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if self.format_version != 1
            || !package_valid
            || self.assets.len() as u64 > DESKTOP_PLUGIN_RELEASE_ASSETS
            || self.version.len() as u64 > DESKTOP_PLUGIN_RELEASE_VERSION_BYTES
            || self.requires_dokime.len() as u64 > DESKTOP_PLUGIN_RELEASE_VERSION_BYTES
        {
            return Err("Plugin release metadata has an unsupported format or exceeds its limits".into());
        }
        Version::parse(&self.version).map_err(|_| "Plugin release version is not SemVer")?;
        let requirement = VersionReq::parse(&self.requires_dokime).map_err(|_| "Plugin host requirement is not a SemVer requirement")?;
        let host = Version::parse(host_version).map_err(|_| "The backend version is not SemVer")?;
        if !requirement.matches(&host) {
            return Err("The plugin release does not support this backend version".into());
        }
        let mut targets = BTreeSet::new();
        for asset in &self.assets {
            if asset.target.is_empty()
                || asset.target.len() as u64 > DESKTOP_ORGANIZATION_IDENTIFIER_BYTES
                || !targets.insert(&asset.target)
            {
                return Err("Plugin releases require distinct nonempty targets".into());
            }
            https_url(&asset.url, DESKTOP_ORGANIZATION_URL_BYTES)?;
            if asset.sha256.len() != 64 || !asset.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err("Plugin release SHA-256 is invalid".into());
            }
        }
        self.assets
            .iter()
            .find(|asset| asset.target == target)
            .cloned()
            .ok_or_else(|| "The plugin release has no asset for this backend target".into())
    }
}

/* === Public Functions === */

pub async fn desktop_plugin_release_check(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, state: State<'_, UpdateState>, generation: String,
) -> Result<Release, String> {
    let lease = plugins::admit(&window, &connection, &generation)?;
    let (policy, policy_generation) = state.snapshot()?;
    if policy.mode == crate::app::organization::policy::Mode::Disabled {
        return Err("Updates are disabled".into());
    }
    let auth = state.authorization(&policy).await?;
    let feed = sources::resolve(&policy, "plugins.json", auth.as_ref()).await?;
    let bytes = sources::fetch(&feed, auth.as_ref(), DESKTOP_UPDATE_MANIFEST_BYTES as usize).await?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| "The plugin release feed is invalid")?;
    let client = lease.client();
    let backend = client.backend();
    let asset = manifest.select(&backend.product_version, &backend.target)?;
    let id = format!("{:032x}", rand::random::<u128>());
    let release = Release {
        candidate: id.clone(),
        package: manifest.package.clone(),
        version: manifest.version.clone(),
        target: asset.target.clone(),
        requires_dokime: manifest.requires_dokime.clone(),
    };
    let mut inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
    if inner.generation != policy_generation {
        return Err("The plugin release source changed while checking".into());
    }
    inner.plugin = Some(Candidate { id, generation: policy_generation, connection: generation, manifest, asset });
    Ok(release)
}

pub async fn desktop_plugin_release_stage(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, state: State<'_, UpdateState>, generation: String, candidate: String,
    progress: Channel<StageProgress>,
) -> Result<String, String> {
    let lease = plugins::admit(&window, &connection, &generation)?;
    let selected = {
        let inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        inner
            .plugin
            .as_ref()
            .filter(|item| item.id == candidate && item.connection == generation && item.generation == inner.generation)
            .cloned()
            .ok_or("The reviewed plugin release is no longer available")?
    };
    let (policy, policy_generation) = state.snapshot()?;
    if selected.generation != policy_generation {
        return Err("The plugin release source changed".into());
    }
    let auth = state.authorization(&policy).await?;
    let url = https_url(&selected.asset.url, DESKTOP_ORGANIZATION_URL_BYTES)?;
    let bytes = sources::fetch(&url, auth.as_ref(), DESKTOP_PLUGIN_BUNDLE_BYTES as usize).await?;
    let client = lease.client();
    selected.manifest.select(&client.backend().product_version, &client.backend().target)?;
    if state.snapshot()?.1 != policy_generation {
        return Err("The plugin release source changed while downloading".into());
    }
    let staged = plugins::stage(&lease, bytes, progress, Some(&selected.asset.sha256)).await?;
    let identity = staged.metadata.as_ref().and_then(|metadata| metadata.identity.as_ref());
    let matching = identity
        .is_some_and(|identity| identity.name == selected.manifest.package && identity.version == selected.manifest.version)
        && staged.sha256.eq_ignore_ascii_case(&selected.asset.sha256)
        && state.snapshot().is_ok_and(|(_, generation)| generation == policy_generation);
    if !matching {
        let removal = contracts::plugins::Remove { install_id: staged.install_id };
        client.plugins_remove(removal).await.map_err(|_| "Plugin release identity was rejected; its unconsented stage requires removal")?;
        return Err("The inspected plugin identity does not match the reviewed release or its source changed".into());
    }
    Ok(staged.install_id)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_selection_requires_compatible_host_unique_target_and_digest() {
        let asset =
            Asset { target: "x86_64-pc-windows-msvc".into(), url: "https://example.test/plugin.dokime".into(), sha256: "a".repeat(64) };
        let mut manifest = Manifest {
            format_version: 1,
            package: "instruments".into(),
            version: "1.2.3".into(),
            requires_dokime: ">=0.1.0, <0.2.0".into(),
            assets: vec![asset.clone()],
        };
        assert!(manifest.select("0.1.0", &asset.target).is_ok());
        assert!(manifest.select("0.2.0", &asset.target).is_err());
        assert!(manifest.select("0.1.0", "x86_64-unknown-linux-gnu").is_err());
        manifest.assets.push(asset.clone());
        assert!(manifest.select("0.1.0", &asset.target).is_err());
        manifest.assets.pop();
        manifest.assets[0].sha256 = "invalid".into();
        assert!(manifest.select("0.1.0", &asset.target).is_err());
    }
}
