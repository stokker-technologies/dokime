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
    plugins::{self, InstallProgress},
};
use crate::connection::ConnectionSession;
use dokime::domain::model::plugins::wire::INSTALL_ARCHIVE_MAX_BYTES;

/* === Constants === */

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
            && self.package.len() <= 128
            && self.package.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
        if self.format_version != 1
            || !package_valid
            || self.assets.len() > 32
            || self.version.len() > 256
            || self.requires_dokime.len() > 256
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
            if asset.target.is_empty() || asset.target.len() > 128 || !targets.insert(&asset.target) {
                return Err("Plugin releases require distinct nonempty targets".into());
            }
            https_url(&asset.url)?;
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
    let bytes = sources::fetch(&feed, auth.as_ref(), sources::MANIFEST_BYTES).await?;
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

pub async fn desktop_plugin_release_install(
    window: WebviewWindow, connection: State<'_, ConnectionSession>, state: State<'_, UpdateState>, generation: String, candidate: String,
    progress: Channel<InstallProgress>,
) -> Result<String, String> {
    let lease = plugins::admit(&window, &connection, &generation)?;
    let selected = {
        let inner = state.0.inner.lock().map_err(|_| "Update configuration is unavailable")?;
        inner
            .plugin
            .as_ref()
            .filter(|item| item.id == candidate && item.connection == generation && item.generation == inner.generation)
            .cloned()
            .ok_or("The selected plugin release is no longer available")?
    };
    let (policy, policy_generation) = state.snapshot()?;
    if selected.generation != policy_generation {
        return Err("The plugin release source changed".into());
    }
    let auth = state.authorization(&policy).await?;
    let url = https_url(&selected.asset.url)?;
    let bytes = sources::fetch(&url, auth.as_ref(), INSTALL_ARCHIVE_MAX_BYTES as usize).await?;
    let client = lease.client();
    selected.manifest.select(&client.backend().product_version, &client.backend().target)?;
    let manifest = selected.manifest.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        validate_release_bundle(&bytes, &manifest)?;
        Ok::<_, String>(bytes)
    })
    .await
    .map_err(|error| error.to_string())??;
    if state.snapshot()?.1 != policy_generation {
        return Err("The plugin release source changed while inspecting the download".into());
    }
    let installed = plugins::install(&lease, bytes, progress, Some(&selected.asset.sha256)).await?;
    Ok(installed.install_id)
}

/* === Private Functions === */

fn validate_release_bundle(bytes: &[u8], selected: &Manifest) -> Result<(), String> {
    let bundle =
        harmos_packaging::bundle::Bundle::inspect(bytes, harmos_packaging::bundle::Limits::default()).map_err(|error| error.to_string())?;
    let identity = &bundle.manifest().identity;
    if identity.name != selected.package || identity.version.to_string() != selected.version {
        return Err("The downloaded plugin identity does not match the selected release".into());
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_bundle_identity() {
        use harmos_packaging::bundle::{Bundle, Identity, Limits, Source};

        let bundle = Bundle::build(
            Identity {
                name: "instruments".into(),
                version: "1.2.3".parse().unwrap(),
                description: String::new(),
                package: Default::default(),
            },
            &[Source { path: "resource.txt", bytes: b"verified resource", executable: None }],
            Limits::default(),
        )
        .unwrap();
        let bytes = bundle.bytes();
        let mut selected = Manifest {
            format_version: 1,
            package: "instruments".into(),
            version: "1.2.3".into(),
            requires_dokime: "*".into(),
            assets: vec![],
        };
        validate_release_bundle(bytes, &selected).unwrap();
        selected.package = "other".into();
        assert!(validate_release_bundle(bytes, &selected).unwrap_err().contains("identity does not match"));
        selected.package = "instruments".into();
        selected.version = "2.0.0".into();
        assert!(validate_release_bundle(bytes, &selected).unwrap_err().contains("identity does not match"));
        assert!(validate_release_bundle(b"invalid archive", &selected).is_err());
    }

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
