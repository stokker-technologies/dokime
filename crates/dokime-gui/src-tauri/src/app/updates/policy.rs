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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};
use url::Url;

// Local Imports

/* === Constants === */

const MAXIMUM_SOURCES: usize = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Organization Defaults Contain Source Locations, Never Authentication Values.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePolicy {
    #[serde(default)]
    pub mode: Mode,

    pub source: Option<String>,

    #[serde(default)]
    pub sources: BTreeMap<String, Source>,

    #[serde(default)]
    pub connections: BTreeMap<String, Connection>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub host: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Disabled,

    Manual,
    AutomaticCheck,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    Direct { manifest: String, plugins: Option<String> },
    Github { repository: String },
    Gitlab { connection: String, project: String },
}

/* === Traits === */

/* === Implementations === */

impl UpdatePolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.sources.len() > MAXIMUM_SOURCES || self.connections.len() > MAXIMUM_SOURCES {
            return Err("organization updates allow at most 32 sources and 32 connections".into());
        }
        for (name, connection) in &self.connections {
            validate_name(name)?;
            let host = https_url(&connection.host)?;
            if host.path() != "/" || host.query().is_some() {
                return Err(format!("connection {name} must name an HTTPS origin without a path or query"));
            }
        }
        for (name, source) in &self.sources {
            validate_name(name)?;
            self.validate_source(source)?;
        }
        if let Some(source) = &self.source {
            if !self.sources.contains_key(source) {
                return Err(format!("selected organization update source {source:?} does not exist"));
            }
        } else if self.mode != Mode::Disabled {
            return Err("enabled organization updates require a selected source".into());
        }
        Ok(())
    }

    fn validate_source(&self, source: &Source) -> Result<(), String> {
        match source {
            Source::Direct { manifest, plugins } => {
                https_url(manifest)?;
                if let Some(plugins) = plugins {
                    https_url(plugins)?;
                }
            }
            Source::Github { repository } => {
                let parts: Vec<_> = repository.split('/').collect();
                if parts.len() != 2 || parts.iter().any(|part| validate_name(part).is_err()) {
                    return Err("GitHub update repository must be owner/repository".into());
                }
            }
            Source::Gitlab { connection, project } => {
                self.connection(connection)?;
                if project.is_empty() || project.len() > 512 || project.split('/').any(|part| validate_name(part).is_err()) {
                    return Err("GitLab update project must be an ID or namespace/project path".into());
                }
            }
        }
        Ok(())
    }

    fn connection(&self, name: &str) -> Result<&Connection, String> {
        self.connections.get(name).ok_or_else(|| format!("organization connection {name:?} does not exist"))
    }
}

/* === Public Functions === */

pub fn https_url(value: &str) -> Result<Url, String> {
    if value.len() > 4096 {
        return Err("organization release URL exceeds 4096 bytes".into());
    }
    let url = Url::parse(value).map_err(|_| "organization release URL is invalid")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err("organization release URLs require HTTPS, a host, and no embedded credentials or fragment".into());
    }
    Ok(url)
}

/* === Private Functions === */

fn validate_name(value: &str) -> Result<(), String> {
    let safe = !value.is_empty()
        && value.len() <= 128
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if !safe {
        return Err("organization source and connection names must use 1–128 letters, digits, dots, underscores or hyphens".into());
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_selection_requires_trusted_public_locations() {
        let value = serde_json::json!({
            "mode": "manual", "source": "team",
            "connections": {"office": {"host": "https://gitlab.example.test"}},
            "sources": {"team": {"kind": "gitlab", "connection": "office", "project": "lab/instruments"}}
        });
        let policy: UpdatePolicy = serde_json::from_value(value.clone()).unwrap();
        policy.validate().unwrap();
        for (pointer, replacement) in [
            ("/source", "missing"),
            ("/sources/team/connection", "missing"),
            ("/connections/office/host", "http://gitlab.example.test"),
            ("/connections/office/host", "https://gitlab.example.test/private"),
        ] {
            let mut invalid = value.clone();
            *invalid.pointer_mut(pointer).unwrap() = replacement.into();
            assert!(serde_json::from_value::<UpdatePolicy>(invalid).unwrap().validate().is_err());
        }
        let mut secret = value;
        secret["connections"]["office"]["access_token"] = "forbidden".into();
        assert!(serde_json::from_value::<UpdatePolicy>(secret).is_err());
        UpdatePolicy::default().validate().unwrap();
    }

    #[test]
    fn public_sources_reject_credentials_and_non_https_urls() {
        for source in [
            serde_json::json!({"kind": "github", "repository": "org/product"}),
            serde_json::json!({"kind": "direct", "manifest": "https://example.test/release.json"}),
        ] {
            let policy: UpdatePolicy = serde_json::from_value(serde_json::json!({
                "mode": "manual", "source": "public", "sources": {"public": source}
            }))
            .unwrap();
            policy.validate().unwrap();
        }
        for url in ["file:///tmp/release.json", "https://user:password@example.test/release.json", "https://example.test/#fragment"] {
            assert!(https_url(url).is_err());
        }
    }
}
