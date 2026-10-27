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

const DESKTOP_ORGANIZATION_IDENTIFIER_BYTES: u64 = 128;
const DESKTOP_ORGANIZATION_REFERENCE_BYTES: u64 = 512;
const DESKTOP_ORGANIZATION_SOURCES: u64 = 32;
const DESKTOP_ORGANIZATION_URL_BYTES: u64 = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Organization Defaults Contain Source Locations, Never Authentication Values.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UpdatePolicy {
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
pub(crate) struct Connection {
    pub host: String,
    #[serde(default)]
    pub oauth_client_id: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mode {
    #[default]
    Disabled,
    Manual,
    AutomaticCheck,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Source {
    Direct { manifest: String, plugins: Option<String>, connection: Option<String> },
    Github { repository: String },
    Gitlab { connection: String, project: String },
}

/* === Traits === */

/* === Implementations === */

impl UpdatePolicy {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.sources.len() > DESKTOP_ORGANIZATION_SOURCES as usize || self.connections.len() > DESKTOP_ORGANIZATION_SOURCES as usize {
            return Err("organization updates exceed the configured source or connection count".into());
        }
        for (name, connection) in &self.connections {
            validate_name(name, DESKTOP_ORGANIZATION_IDENTIFIER_BYTES)?;
            let host = https_url(&connection.host, DESKTOP_ORGANIZATION_URL_BYTES)?;
            if host.path() != "/" || host.query().is_some() {
                return Err(format!("connection {name} must name an HTTPS origin without a path or query"));
            }
            if connection.oauth_client_id.len() as u64 > DESKTOP_ORGANIZATION_REFERENCE_BYTES
                || connection.oauth_client_id.chars().any(char::is_control)
            {
                return Err(format!("connection {name} has an invalid OAuth client identifier"));
            }
        }
        for (name, source) in &self.sources {
            validate_name(name, DESKTOP_ORGANIZATION_IDENTIFIER_BYTES)?;
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
            Source::Direct { manifest, plugins, connection } => {
                https_url(manifest, DESKTOP_ORGANIZATION_URL_BYTES)?;
                if let Some(plugins) = plugins {
                    https_url(plugins, DESKTOP_ORGANIZATION_URL_BYTES)?;
                }
                if let Some(connection) = connection {
                    self.connection(connection)?;
                }
            }
            Source::Github { repository } => {
                let parts: Vec<_> = repository.split('/').collect();
                if parts.len() != 2 || parts.iter().any(|part| validate_name(part, DESKTOP_ORGANIZATION_IDENTIFIER_BYTES).is_err()) {
                    return Err("GitHub update repository must be owner/repository".into());
                }
            }
            Source::Gitlab { connection, project } => {
                self.connection(connection)?;
                if project.is_empty()
                    || project.len() as u64 > DESKTOP_ORGANIZATION_REFERENCE_BYTES
                    || project.split('/').any(|part| validate_name(part, DESKTOP_ORGANIZATION_IDENTIFIER_BYTES).is_err())
                {
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

pub(crate) fn https_url(value: &str, maximum: u64) -> Result<Url, String> {
    if value.len() as u64 > maximum {
        return Err("organization release URL exceeds its configured byte budget".into());
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

fn validate_name(value: &str, maximum: u64) -> Result<(), String> {
    let safe = !value.is_empty()
        && value.len() as u64 <= maximum
        && value != "."
        && value != ".."
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    if !safe {
        return Err("organization source and connection names exceed their configured bound or use unsupported characters".into());
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_requires_explicit_valid_sources_and_disallows_secret_fields() {
        let text = r#"
mode = "manual"
source = "team"
[connections.office]
host = "https://gitlab.example.test"
oauth_client_id = "desktop-public-client"
[sources.team]
kind = "gitlab"
connection = "office"
project = "lab/instruments"
"#;
        let policy: UpdatePolicy = toml::from_str(text).unwrap();
        policy.validate().unwrap();
        for altered in [
            text.replace("source = \"team\"", "source = \"missing\""),
            text.replace("connection = \"office\"", "connection = \"missing\""),
            text.replace("https://gitlab.example.test", "https://gitlab.example.test/private"),
            text.replace("https://gitlab.example.test", "http://gitlab.example.test"),
        ] {
            let invalid: UpdatePolicy = toml::from_str(&altered).unwrap();
            assert!(invalid.validate().is_err());
        }
        let secret = text.replace("oauth_client_id =", "access_token =");
        assert!(toml::from_str::<UpdatePolicy>(&secret).is_err());
        assert!(toml::from_str::<UpdatePolicy>("mode = 'manual'").unwrap().validate().is_err());
        UpdatePolicy::default().validate().unwrap();
    }

    #[test]
    fn public_sources_need_no_authentication_and_urls_cannot_embed_credentials() {
        for text in [
            "mode='automatic_check'\nsource='public'\n[sources.public]\nkind='github'\nrepository='org/product'",
            "mode='manual'\nsource='public'\n[sources.public]\nkind='direct'\nmanifest='https://example.test/latest.json'\nplugins='https://example.test/plugins.json'",
        ] {
            let policy: UpdatePolicy = toml::from_str(text).unwrap();
            policy.validate().unwrap();
        }
        for url in ["file:///tmp/latest.json", "https://user:password@example.test/latest.json", "https://example.test/#fragment"] {
            assert!(https_url(url, DESKTOP_ORGANIZATION_URL_BYTES).is_err());
        }
    }
}
