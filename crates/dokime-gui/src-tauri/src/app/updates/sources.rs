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
use std::time::Duration;

// Third Party Imports
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Deserialize;
use url::Url;
use zeroize::Zeroizing;

// Local Imports
use crate::app::organization::policy::{Mode, Source, UpdatePolicy, https_url};

/* === Constants === */

pub(super) const MANIFEST_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Authorization {
    pub origin: Url,
    pub token: Zeroizing<String>,
}

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

impl Authorization {
    pub(super) fn headers(&self, target: &Url) -> Result<HeaderMap, String> {
        let mut headers = HeaderMap::new();
        if target.origin() != self.origin.origin() {
            return Ok(headers);
        }
        let bearer = Zeroizing::new(format!("Bearer {}", self.token.as_str()));
        let mut value = HeaderValue::from_str(&bearer).map_err(|_| "The saved account token is not a valid authorization value")?;
        value.set_sensitive(true);
        headers.insert(AUTHORIZATION, value);
        Ok(headers)
    }
}

/* === Public Functions === */

pub(super) fn client(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    // Reqwest Removes Authorization On Origin Changes, Including Scheme And Port.
    let redirects = reqwest::redirect::Policy::custom(|attempt| {
        if https_url(attempt.url().as_str()).is_err() {
            return attempt.error("Release redirects require HTTPS without embedded credentials");
        }
        if attempt.previous().len() >= 10 {
            return attempt.error("Release redirect limit exceeded");
        }
        attempt.follow()
    });
    builder.redirect(redirects).referer(false).timeout(Duration::from_secs(300)).user_agent("Dokime Desktop")
}

pub(super) fn headers(auth: Option<&Authorization>, target: &Url) -> Result<HeaderMap, String> {
    match auth {
        Some(auth) => auth.headers(target),
        None => Ok(HeaderMap::new()),
    }
}

pub(super) async fn fetch(url: &Url, auth: Option<&Authorization>, maximum: usize) -> Result<Vec<u8>, String> {
    https_url(url.as_str())?;
    let headers = headers(auth, url)?;
    let client = client(reqwest::Client::builder()).build().map_err(|_| "Release HTTP client initialization failed")?;
    let response = client.get(url.clone()).headers(headers).send().await.map_err(|_| "Release request failed")?;
    let mut response = response.error_for_status().map_err(|_| "Release source is unavailable or requires another account")?;
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

pub(super) async fn resolve(policy: &UpdatePolicy, asset: &str, auth: Option<&Authorization>) -> Result<Url, String> {
    policy.validate()?;
    if policy.mode == Mode::Disabled {
        return Err("Updates are disabled".into());
    }
    let selected = policy.source.as_ref().ok_or("No release source is selected")?;
    let source = policy.sources.get(selected).ok_or("The selected release source is unavailable")?;
    match source {
        Source::Direct { manifest, plugins, .. } => {
            let location =
                if asset == "latest.json" { manifest } else { plugins.as_ref().ok_or("This source has no plugin release feed")? };
            https_url(location)
        }
        Source::Github { repository } => {
            let url = https_url(&format!("https://api.github.com/repos/{repository}/releases/latest"))?;
            let bytes = fetch(&url, auth, MANIFEST_BYTES).await?;
            let release: GithubRelease = serde_json::from_slice(&bytes).map_err(|_| "GitHub release metadata is invalid")?;
            let assets = release.assets.into_iter().map(|entry| (entry.name, entry.browser_download_url));
            named_asset(assets, asset)
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
            let bytes = fetch(&url, auth, MANIFEST_BYTES).await?;
            let release: GitlabRelease = serde_json::from_slice(&bytes).map_err(|_| "GitLab release metadata is invalid")?;
            let assets = release.assets.links.into_iter().map(|entry| (entry.name, entry.direct_asset_url.unwrap_or(entry.url)));
            named_asset(assets, asset)
        }
    }
}

/* === Private Functions === */

fn named_asset(assets: impl Iterator<Item = (String, String)>, wanted: &str) -> Result<Url, String> {
    let mut matching = assets.filter(|(name, _)| name == wanted);
    let (_, location) = matching.next().ok_or_else(|| format!("The release has no {wanted} asset"))?;
    if matching.next().is_some() {
        return Err(format!("The release has more than one {wanted} asset"));
    }
    https_url(&location)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_sources_are_local_and_authorization_is_origin_scoped() {
        let disabled = UpdatePolicy::default();
        assert_eq!(resolve(&disabled, "latest.json", None).await.unwrap_err(), "Updates are disabled");
        let origin = https_url("https://gitlab.example.test").unwrap();
        let auth = Authorization { origin, token: Zeroizing::new("test-token".into()) };
        for location in ["https://cdn.example.test/update", "https://gitlab.example.test:444/update", "http://gitlab.example.test/update"] {
            let target = Url::parse(location).unwrap();
            assert!(!auth.headers(&target).unwrap().contains_key(AUTHORIZATION));
        }
        let target = https_url("https://gitlab.example.test/api/v4/user").unwrap();
        let headers = auth.headers(&target).unwrap();
        assert!(headers[AUTHORIZATION].is_sensitive());
        let duplicate = vec![("latest.json".into(), target.to_string()), ("latest.json".into(), target.to_string())];
        assert!(named_asset(duplicate.into_iter(), "latest.json").is_err());
    }
}
