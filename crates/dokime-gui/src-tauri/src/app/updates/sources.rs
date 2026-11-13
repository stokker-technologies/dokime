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
use std::{collections::BTreeMap, time::Duration};

// Third Party Imports
use serde::Deserialize;
use url::Url;

// Local Imports
use super::policy::{Mode, Source, UpdatePolicy, https_url};

/* === Constants === */

pub(super) const DESKTOP_UPDATE_MANIFEST_BYTES: u64 = 1_048_576;
const DESKTOP_UPDATE_REDIRECTS: u64 = 10;
const DESKTOP_UPDATE_TIMEOUT_MS: u64 = 300_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitlabRelease {
    assets: GitlabAssets,
}

#[derive(Deserialize)]
struct GitlabAssets {
    links: Vec<GitlabAsset>,
}

#[derive(Deserialize)]
struct GitlabAsset {
    name: String,
    url: String,
    direct_asset_url: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn client(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    // Reqwest Removes Authorization On Origin Changes, Including Scheme And Port.
    let redirects_maximum = DESKTOP_UPDATE_REDIRECTS;
    let redirects = reqwest::redirect::Policy::custom(move |attempt| {
        if https_url(attempt.url().as_str()).is_err() {
            return attempt.error("Release redirects require HTTPS without embedded credentials");
        }
        if attempt.previous().len() as u64 >= redirects_maximum {
            return attempt.error("Release redirect limit exceeded");
        }
        attempt.follow()
    });
    builder.redirect(redirects).referer(false).timeout(Duration::from_millis(DESKTOP_UPDATE_TIMEOUT_MS)).user_agent("Dokime Desktop")
}

pub(super) async fn fetch(url: &Url, maximum: usize) -> Result<Vec<u8>, String> {
    https_url(url.as_str())?;
    let client = client(crate::app::http::client()).build().map_err(|_| "Release HTTP client initialization failed")?;
    let response = client.get(url.clone()).send().await.map_err(|_| "Release request failed")?;
    let mut response = response.error_for_status().map_err(|_| "Public release source is unavailable")?;
    if response.content_length().is_some_and(|length| length > maximum as u64) {
        return Err("Release response exceeds its size limit".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "Release response could not be read")? {
        if chunk.len() > maximum.saturating_sub(bytes.len()) {
            return Err("Release response exceeds its size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub(super) async fn resolve(policy: &UpdatePolicy, feed_name: &str) -> Result<Url, String> {
    policy.validate()?;
    if policy.mode == Mode::Disabled {
        return Err("Updates are disabled".into());
    }
    let selected = policy.source.as_ref().ok_or("No release source is selected")?;
    let source = policy.sources.get(selected).ok_or("The selected release source is unavailable")?;
    let entries: Vec<(String, String)> = match source {
        Source::Direct { manifest, .. } => {
            return https_url(manifest);
        }
        Source::Github { repository } => {
            let url = https_url(&format!("https://api.github.com/repos/{repository}/releases/latest"))?;
            let bytes = fetch(&url, DESKTOP_UPDATE_MANIFEST_BYTES as usize).await?;
            let release: GithubRelease = serde_json::from_slice(&bytes).map_err(|_| "GitHub release metadata is invalid")?;
            release.assets.into_iter().map(|entry| (entry.name, entry.browser_download_url)).collect()
        }
        Source::Gitlab { connection, project } => {
            let connection = policy.connections.get(connection).ok_or("The release connection is unavailable")?;
            let mut url = https_url(&connection.host)?;
            url.path_segments_mut().map_err(|_| "The GitLab origin cannot hold a release path")?.extend([
                "api",
                "v4",
                "projects",
                project,
                "releases",
                "permalink",
                "latest",
            ]);
            let bytes = fetch(&url, DESKTOP_UPDATE_MANIFEST_BYTES as usize).await?;
            let release: GitlabRelease = serde_json::from_slice(&bytes).map_err(|_| "GitLab release metadata is invalid")?;
            release.assets.links.into_iter().map(|entry| (entry.name, entry.direct_asset_url.unwrap_or(entry.url))).collect()
        }
    };
    let assets = named_assets(entries)?;
    let feed = assets.get(feed_name).cloned().ok_or_else(|| format!("The release has no {feed_name} asset"))?;
    Ok(feed)
}

/* === Private Functions === */

fn named_assets(entries: impl IntoIterator<Item = (String, String)>) -> Result<BTreeMap<String, Url>, String> {
    let mut assets = BTreeMap::new();
    for (name, location) in entries {
        if assets.insert(name, https_url(&location)?).is_some() {
            return Err("Release asset names must be unique".into());
        }
    }
    Ok(assets)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_sources_are_local_and_duplicate_assets_are_rejected() {
        let disabled = UpdatePolicy::default();
        assert_eq!(resolve(&disabled, "release.json").await.unwrap_err(), "Updates are disabled");
        let target = https_url("https://example.test/release.json").unwrap();
        let duplicate = vec![("release.json".into(), target.to_string()), ("release.json".into(), target.to_string())];
        assert!(named_assets(duplicate).is_err());
    }
}
