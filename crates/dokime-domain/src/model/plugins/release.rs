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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use semver::{Version, VersionReq};
use serde::Deserialize;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feed {
    pub format_version: u32,
    pub plugins: Vec<Plugin>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    pub name: String,
    pub kind: Kind,
    pub version: String,
    pub requires_dokime: String,
    pub targets: BTreeMap<String, Member>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub archive: String,
    pub member: String,
    pub sha256: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Sidecar,
    Guest,
}

/* === Traits === */

/* === Implementations === */

impl Feed {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1024 * 1024 {
            return Err("Plugin release feed exceeds its byte limit".into());
        }
        let feed: Self = serde_json::from_slice(bytes).map_err(|error| format!("Invalid plugin release feed: {error}"))?;
        if feed.format_version != 2 {
            return Err("Only plugin release feed format 2 is supported".into());
        }
        let mut names = BTreeSet::new();
        for plugin in &feed.plugins {
            if !names.insert(&plugin.name) {
                return Err("Plugin releases require distinct names".into());
            }
            plugin.validate()?;
        }
        Ok(feed)
    }
}

impl Plugin {
    fn validate(&self) -> Result<(), String> {
        let targets_valid = !self.targets.is_empty() && self.targets.len() <= 32;
        let versions_bounded = self.version.len() <= 256 && self.requires_dokime.len() <= 256;
        if !identifier(&self.name) || !targets_valid || !versions_bounded {
            return Err("Plugin release name, versions or targets exceed their limits".into());
        }
        Version::parse(&self.version).map_err(|error| error.to_string())?;
        VersionReq::parse(&self.requires_dokime).map_err(|error| error.to_string())?;

        for (target, member) in &self.targets {
            let names_valid = identifier(target) && filename(&member.archive) && filename(&member.member);
            if !names_valid || !member.member.ends_with(".dokime") {
                return Err("Plugin release archive, member or target is invalid".into());
            }
            let archive_valid = match self.kind {
                Kind::Sidecar => member.archive.ends_with(".zip") || member.archive.ends_with(".tar.gz"),
                Kind::Guest => member.archive.ends_with(".zip"),
            };
            if !archive_valid || !sha256(&member.sha256) {
                return Err("Plugin release archive format or member SHA-256 is invalid".into());
            }
        }
        Ok(())
    }

    pub fn select(&self, host_version: &str, target: &str) -> Result<&Member, String> {
        let host = Version::parse(host_version).map_err(|error| error.to_string())?;
        let requirement = VersionReq::parse(&self.requires_dokime).map_err(|error| error.to_string())?;
        if !requirement.matches(&host) {
            return Err(format!("Plugin {} requires Dokime {}", self.name, self.requires_dokime));
        }
        if let Some(member) = self.targets.get(target) {
            return Ok(member);
        }
        if self.kind == Kind::Guest
            && let Some(member) = self.targets.get("any")
        {
            return Ok(member);
        }
        Err(format!("Plugin {} has no member for target {target}", self.name))
    }
}

/* === Public Functions === */

pub fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn filename(value: &str) -> bool {
    let length_valid = !value.is_empty() && value.len() <= 255;
    let ordinary_name = value != "." && value != "..";
    let characters_valid = value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
    length_valid && ordinary_name && characters_valid
}

/* === Private Functions === */

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_selection() {
        let plugin = serde_json::json!({"name":"opaque-name", "kind":"sidecar", "version":"1.2.3", "requires_dokime":"^0.1",
            "targets":{"native":{"archive":"sidecars.zip", "member":"opaque-name.dokime", "sha256":"a".repeat(64)}}});
        let mut value = serde_json::json!({"format_version":2,"plugins":[plugin]});
        let feed = Feed::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(feed.plugins[0].select("0.1.3", "native").unwrap().member, "opaque-name.dokime");
        assert!(feed.plugins[0].select("0.2.0", "native").is_err());
        assert!(feed.plugins[0].select("0.1.3", "foreign").is_err());
        value["format_version"] = 1.into();
        assert!(Feed::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["format_version"] = 2.into();
        let duplicate = value["plugins"][0].clone();
        value["plugins"].as_array_mut().unwrap().push(duplicate);
        assert!(Feed::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value["plugins"][1]["name"] = "second-plugin".into();
        let feed = Feed::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(feed.plugins.len(), 2);

        value["plugins"].as_array_mut().unwrap().pop();
        value["plugins"][0]["targets"]["native"]["member"] = "../escape.dokime".into();
        assert!(Feed::parse(&serde_json::to_vec(&value).unwrap()).is_err());

        value["plugins"][0]["kind"] = "guest".into();
        let member = serde_json::json!({"archive":"guests.zip","member":"guest.dokime","sha256":"a".repeat(64)});
        value["plugins"][0]["targets"] = serde_json::json!({"any": member});
        let feed = Feed::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(feed.plugins[0].select("0.1.3", "native").unwrap().archive, "guests.zip");
    }
}
