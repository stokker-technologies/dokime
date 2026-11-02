/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Resolve And Cache Integrity-Checked Release Artifacts.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

// Third Party Imports
use reqwest::{
    StatusCode,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::Deserialize;

// Local Imports
use crate::Error;
use dokime::{
    distribution,
    domain::model::plugins::release::{self, Feed},
};

/* === Constants === */

const METADATA_MAX_BYTES: usize = 1024 * 1024;
const RELEASE_API: &str = "https://api.github.com/repos/stokker-technologies/dokime/releases/tags";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Release {
    client: reqwest::Client,
    assets: BTreeMap<String, reqwest::Url>,
    sums: BTreeMap<String, String>,
    cache: PathBuf,
}

#[derive(Deserialize)]
struct Metadata {
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: reqwest::Url,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Release {
    pub async fn resolve(version: &str) -> Result<Self, Error> {
        if version != "dev-channel" {
            semver::Version::parse(version).map_err(|error| Error::Invalid(error.to_string()))?;
        }
        let tag = if version == "dev-channel" { version.to_owned() } else { format!("v{version}") };
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .https_only(true)
            .user_agent("cargo-dokime")
            .build()
            .map_err(|error| Error::Invalid(error.to_string()))?;
        let url = reqwest::Url::parse(&format!("{RELEASE_API}/{tag}")).map_err(|error| Error::Invalid(error.to_string()))?;
        let bytes = fetch(&client, &url, METADATA_MAX_BYTES, "release metadata").await?;
        let metadata: Metadata = serde_json::from_slice(&bytes)?;
        let mut assets = BTreeMap::new();
        for asset in metadata.assets {
            let url = &asset.browser_download_url;
            let safe_url = url.scheme() == "https" && url.username().is_empty() && url.password().is_none();
            if !safe_url || url.fragment().is_some() || !release::filename(&asset.name) {
                return Err(Error::Invalid("invalid release asset location".into()));
            }
            if assets.insert(asset.name, asset.browser_download_url).is_some() {
                return Err(Error::Invalid("duplicate release asset name".into()));
            }
        }
        Self::open(client, assets, cache_root()?).await
    }

    pub async fn open(client: reqwest::Client, assets: BTreeMap<String, reqwest::Url>, cache: PathBuf) -> Result<Self, Error> {
        let sums_url = assets.get("SHA256SUMS").ok_or_else(|| Error::Invalid("release has no SHA256SUMS".into()))?;
        let bytes = fetch(&client, sums_url, METADATA_MAX_BYTES, "SHA256SUMS").await?;
        let sums = distribution::checksums(&bytes).map_err(Error::Invalid)?;
        std::fs::create_dir_all(&cache)?;
        Ok(Self { client, assets, sums, cache })
    }

    pub async fn distribution(&self, version: &str, target: &str, destination: &Path) -> Result<String, Error> {
        let name = self.distribution_name(version, target)?;
        let suffix = format!("-{target}.zip");
        let version = name
            .strip_prefix("dokime-distribution-")
            .and_then(|name| name.strip_suffix(&suffix))
            .ok_or_else(|| Error::Invalid("invalid distribution archive name".into()))?;
        semver::Version::parse(version).map_err(|error| Error::Invalid(error.to_string()))?;
        let bytes = self.download(&name, distribution::ARCHIVE_MAX_BYTES).await?;
        super::distribution::extract(&bytes, destination)?;
        Ok(version.to_owned())
    }

    pub async fn plugins(&self, names: &[String], target: &str, version: &str, destination: &Path) -> Result<Vec<PathBuf>, Error> {
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let bytes = self.download("plugins.json", METADATA_MAX_BYTES).await?;
        let feed = Feed::parse(&bytes).map_err(Error::Invalid)?;
        let mut selected = BTreeSet::new();
        let mut members = BTreeSet::new();
        let mut paths = Vec::new();
        std::fs::create_dir(destination)?;
        for name in names {
            if !selected.insert(name) {
                return Err(Error::Invalid(format!("duplicate requested plugin: {name}")));
            }
            let plugin = feed
                .plugins
                .iter()
                .find(|plugin| &plugin.name == name)
                .ok_or_else(|| Error::Invalid(format!("release has no plugin named {name}")))?;
            let member = plugin.select(version, target).map_err(Error::Invalid)?;
            if !members.insert(member.member.to_ascii_lowercase()) {
                return Err(Error::Invalid("release plugins have colliding member names".into()));
            }
            let archive = self.download(&member.archive, distribution::ARCHIVE_MAX_BYTES).await?;
            let bytes = distribution::plugin(&archive, member, plugin).map_err(Error::Invalid)?;
            let path = destination.join(&member.member);
            std::fs::write(&path, bytes)?;
            paths.push(path);
        }
        Ok(paths)
    }

    fn distribution_name(&self, version: &str, target: &str) -> Result<String, Error> {
        if version != "dev-channel" {
            return Ok(format!("dokime-distribution-{version}-{target}.zip"));
        }
        let suffix = format!("-{target}.zip");
        let mut candidates = self.assets.keys().filter(|name| name.starts_with("dokime-distribution-") && name.ends_with(&suffix));
        let name = candidates.next().ok_or_else(|| Error::Invalid("release has no native distribution archive".into()))?;
        if candidates.next().is_some() {
            return Err(Error::Invalid("release has multiple native distribution archives".into()));
        }
        let version = name.trim_start_matches("dokime-distribution-").trim_end_matches(&suffix);
        semver::Version::parse(version).map_err(|error| Error::Invalid(error.to_string()))?;
        Ok(name.clone())
    }

    async fn download(&self, name: &str, maximum: usize) -> Result<Vec<u8>, Error> {
        let terminal = dokime::terminal::Terminal::default();
        let expected = self.sums.get(name).ok_or_else(|| Error::Invalid(format!("SHA256SUMS has no {name} entry")))?;
        let cached = self.cache.join(expected.to_ascii_lowercase());
        match std::fs::symlink_metadata(&cached) {
            Ok(metadata) => {
                if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum as u64 {
                    return Err(Error::Invalid("invalid release cache entry".into()));
                }
                let bytes = std::fs::read(&cached)?;
                if distribution::verify(&bytes, expected).is_ok() {
                    terminal.status(format!("Verified {name} ({}, cached)", dokime::terminal::size_label(bytes.len() as u64)));
                    return Ok(bytes);
                }
                std::fs::remove_file(&cached)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let url = self.assets.get(name).ok_or_else(|| Error::Invalid(format!("release has no {name} asset")))?;
        let bytes = fetch(&self.client, url, maximum, name).await?;
        terminal.status(format!("Verifying {name} ({})", dokime::terminal::size_label(bytes.len() as u64)));
        distribution::verify(&bytes, expected).map_err(Error::Invalid)?;
        let mut staged = tempfile::NamedTempFile::new_in(&self.cache)?;
        std::io::Write::write_all(&mut staged, &bytes)?;
        match staged.persist_noclobber(&cached) {
            Ok(_) => {}
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = std::fs::symlink_metadata(&cached)?;
                if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum as u64 {
                    return Err(Error::Invalid("invalid concurrent release cache entry".into()));
                }
                distribution::verify(&std::fs::read(&cached)?, expected).map_err(Error::Invalid)?;
            }
            Err(error) => return Err(error.error.into()),
        }
        Ok(bytes)
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn fetch(client: &reqwest::Client, url: &reqwest::Url, maximum: usize, name: &str) -> Result<Vec<u8>, Error> {
    let terminal = dokime::terminal::Terminal::default();
    let request = request(client, url, |name| std::env::var(name).ok())?;
    let response = request.send().await.map_err(|error| Error::Invalid(error.to_string()))?;
    if response.url().host_str() == Some("api.github.com") {
        rate_limit(response.status(), response.headers())?;
    }
    let mut response = response.error_for_status().map_err(|error| Error::Invalid(error.to_string()))?;
    if response.content_length().is_some_and(|length| length > maximum as u64) {
        return Err(Error::Invalid("release response exceeds its byte limit".into()));
    }
    let size = response.content_length().map(dokime::terminal::size_label).unwrap_or_else(|| "size unknown".into());
    terminal.status(format!("Downloading {name} ({size})"));
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| Error::Invalid(error.to_string()))? {
        if chunk.len() > maximum.saturating_sub(bytes.len()) {
            return Err(Error::Invalid("release response exceeds its byte limit".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn request(
    client: &reqwest::Client, url: &reqwest::Url, mut variable: impl FnMut(&str) -> Option<String>,
) -> Result<reqwest::RequestBuilder, Error> {
    let request = client.get(url.clone());
    if url.scheme() != "https" || url.host_str() != Some("api.github.com") {
        return Ok(request);
    }
    let token =
        variable("GITHUB_TOKEN").filter(|token| !token.is_empty()).or_else(|| variable("GH_TOKEN").filter(|token| !token.is_empty()));
    let Some(token) = token else {
        return Ok(request);
    };
    let mut authorization =
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| Error::Invalid("invalid GitHub token header value".into()))?;
    authorization.set_sensitive(true);
    // Reqwest Strips Authorization On Cross-Host Redirects; Never Use Client Default Headers.
    Ok(request.header(AUTHORIZATION, authorization))
}

fn rate_limit(status: StatusCode, headers: &HeaderMap) -> Result<(), Error> {
    let exhausted = headers.get("x-ratelimit-remaining").is_some_and(|value| value == "0");
    let limited =
        status == StatusCode::TOO_MANY_REQUESTS || (status == StatusCode::FORBIDDEN && (exhausted || headers.contains_key("retry-after")));
    if !limited {
        return Ok(());
    }
    let number = |name: &str| headers.get(name).and_then(|value| value.to_str().ok()).and_then(|value| value.parse::<u64>().ok());
    Err(Error::GithubRateLimit {
        limit: number("x-ratelimit-limit"),
        reset: number("x-ratelimit-reset"),
    })
}

fn cache_root() -> Result<PathBuf, Error> {
    #[cfg(windows)]
    let root = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).map(|path| path.join("dokime/cache"));
    #[cfg(not(windows))]
    let root = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .map(|path| path.join("dokime"));
    let root =
        root.filter(|path| path.is_absolute()).ok_or_else(|| Error::Invalid("no absolute per-user cache directory is available".into()))?;
    Ok(root.join("releases/sha256"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> reqwest::Client {
        let _ = rustls::crypto::ring::default_provider().install_default();
        reqwest::Client::builder().build().unwrap()
    }

    #[test]
    fn github_authentication() {
        let client = client();
        let url = reqwest::Url::parse("https://api.github.com/repos/example/project/releases/latest").unwrap();
        for (github, gh, expected) in [
            (Some("primary"), Some("fallback"), Some("Bearer primary")),
            (None, Some("fallback"), Some("Bearer fallback")),
            (Some(""), Some("fallback"), Some("Bearer fallback")),
            (None, None, None),
            (Some(""), Some(""), None),
        ] {
            let request =
                request(&client, &url, |name| if name == "GITHUB_TOKEN" { github.map(str::to_owned) } else { gh.map(str::to_owned) })
                    .unwrap()
                    .build()
                    .unwrap();
            let authorization = request.headers().get(AUTHORIZATION);
            assert_eq!(authorization.map(|value| value.to_str().unwrap()), expected);
            if let Some(authorization) = authorization {
                assert!(authorization.is_sensitive());
                assert!(!format!("{request:?}").contains(github.filter(|value| !value.is_empty()).or(gh).unwrap()));
            }
        }
    }

    #[test]
    fn host_isolation() {
        let client = client();
        for location in [
            "https://objects.githubusercontent.com/asset",
            "https://github.com/example/project/releases/download/v1/asset",
            "https://api.github.com.example.com/asset",
            "https://example.com/asset",
            "http://api.github.com/asset",
        ] {
            let url = reqwest::Url::parse(location).unwrap();
            let request = request(&client, &url, |_| Some("private-token".into())).unwrap().build().unwrap();
            assert!(!request.headers().contains_key(AUTHORIZATION));
        }
    }

    #[test]
    fn token_redaction() {
        let client = client();
        let url = reqwest::Url::parse("https://api.github.com/").unwrap();
        let token = "private-token\r\ninjected: value";
        let error = request(&client, &url, |_| Some(token.into())).unwrap_err();
        assert!(!error.to_string().contains("private-token"));
        assert!(!format!("{error:?}").contains("private-token"));
    }

    #[test]
    fn rate_limit_errors() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", HeaderValue::from_static("0"));
        headers.insert("x-ratelimit-limit", HeaderValue::from_static("60"));
        headers.insert("x-ratelimit-reset", HeaderValue::from_static("1793534400"));
        for status in [StatusCode::FORBIDDEN, StatusCode::TOO_MANY_REQUESTS] {
            let error = rate_limit(status, &headers).unwrap_err();
            assert!(matches!(error, Error::GithubRateLimit { limit: Some(60), reset: Some(1793534400) }));
            let message = error.to_string();
            assert!(message.contains("GitHub API rate limit"));
            assert!(message.contains("60 requests"));
            assert!(message.contains("GITHUB_TOKEN raises the limit"));
            assert!(message.contains("1793534400 (UTC epoch seconds)"));
        }
        assert!(rate_limit(StatusCode::OK, &headers).is_ok());
        assert!(rate_limit(StatusCode::NOT_FOUND, &headers).is_ok());
        let mut headers = HeaderMap::new();
        assert!(rate_limit(StatusCode::FORBIDDEN, &headers).is_ok());
        let error = rate_limit(StatusCode::TOO_MANY_REQUESTS, &headers).unwrap_err();
        assert!(matches!(error, Error::GithubRateLimit { limit: None, reset: None }));
        assert!(!error.to_string().contains("resets at"));
        headers.insert("retry-after", HeaderValue::from_static("30"));
        headers.insert("x-ratelimit-reset", HeaderValue::from_static("invalid"));
        assert!(matches!(rate_limit(StatusCode::FORBIDDEN, &headers), Err(Error::GithubRateLimit { reset: None, .. })));
    }
}
