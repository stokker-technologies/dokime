/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Remote Listener Settings And One-Time Secrets

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message(skip_default)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteListenerSettings {
    pub enabled: bool,
    pub address: String,
    pub tls_enabled: bool,
    pub authentication_enabled: bool,
    pub certificate_names: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteListenerStatus {
    pub settings: Option<RemoteListenerSettings>,
    pub listening_address: Option<String>,
    pub certificate_pem: Option<String>,
    pub key_configured: bool,
    pub error: Option<String>,
}

#[harmos::message(skip_debug)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairingFile {
    pub version: u32,
    pub endpoint: String,
    pub backend_identifier: String,
    pub label: String,

    #[serde(deserialize_with = "required_optional")]
    pub ca_certificate_pem: Option<String>,

    #[serde(deserialize_with = "required_optional")]
    pub access_key: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedClient {
    pub identifier: String,
    pub label: String,
    pub created_at: u64,
    pub last_seen_at: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairedClientPage {
    pub clients: Vec<PairedClient>,
    pub next_cursor: Option<String>,
}

#[harmos::message(skip_debug)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedPairing {
    pub client: Option<PairedClient>,
    pub pairing: PairingFile,
}

/// One remote client's calls from one peer address, admitted or rejected.
///
/// Times are Unix milliseconds. Transport drops are not observable, so a
/// session closes after a quiet interval and `disconnected_at` is its last call.
#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSession {
    pub id: String,
    pub peer_address: String,
    /// The paired client identifier for a verified key, otherwise `anonymous`.
    pub principal: String,
    pub connected_at: u64,
    pub last_seen_at: u64,
    pub disconnected_at: Option<u64>,
    pub calls: u64,
    pub outcome: RemoteSessionOutcome,
    /// Why the host refused the calls; set only for a rejected session.
    pub rejection: Option<String>,
}

/// Newest first; `next_cursor` continues with older sessions.
#[harmos::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteSessionPage {
    pub sessions: Vec<RemoteSession>,
    pub next_cursor: Option<String>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteSessionOutcome {
    Unspecified,
    Admitted,
    Rejected,
}

/* === Traits === */

/* === Implementations === */

impl Default for RemoteListenerSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            address: "0.0.0.0:8765".into(),
            tls_enabled: true,
            authentication_enabled: true,
            certificate_names: Vec::new(),
        }
    }
}

impl RemoteListenerSettings {
    pub fn validate(&self) -> Result<(), String> {
        self.address.parse::<std::net::SocketAddr>().map_err(|_| "listener address must be an IP address and port")?;
        if self.certificate_names.len() > 32 {
            return Err("at most 32 certificate identities are allowed".into());
        }
        for (index, name) in self.certificate_names.iter().enumerate() {
            if name.is_empty() || name.len() > 253 || self.certificate_names[..index].contains(name) {
                return Err("certificate identities must be distinct bounded DNS names or IP addresses".into());
            }
            if name.parse::<std::net::IpAddr>().is_ok() {
                continue;
            }
            if !name.split('.').all(valid_dns_label) {
                return Err("certificate identities must be exact DNS names or IP addresses without wildcards or ports".into());
            }
        }
        Ok(())
    }
}

impl PairingFile {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("unsupported pairing version".into());
        }
        if self.backend_identifier.trim().is_empty()
            || self.backend_identifier.len() > 128
            || self.backend_identifier.chars().any(|character| character.is_control() || character.is_whitespace())
        {
            return Err("invalid backend identifier".into());
        }
        if self.label.trim().is_empty() || self.label.len() > 256 || self.label.chars().any(char::is_control) {
            return Err("pairing label must be nonblank, control-free and at most 256 bytes".into());
        }
        if self.endpoint.len() > 2048 {
            return Err("pairing endpoint is too large".into());
        }
        let (scheme, address) = self.endpoint.split_once("://").ok_or("invalid pairing endpoint")?;
        if !matches!(scheme, "http" | "https") || address.contains(['/', '?', '#', '@', '%', '\\']) {
            return Err("pairing endpoint must be an HTTP(S) origin with an explicit port".into());
        }
        let (host, port) = address.rsplit_once(':').ok_or("pairing endpoint requires a port")?;
        if port.is_empty()
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || port.parse::<u16>().ok().filter(|port| *port != 0).is_none()
        {
            return Err("invalid pairing endpoint port".into());
        }
        if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
            return Err("IPv6 pairing endpoints require brackets".into());
        }
        let bracketed = host.starts_with('[');
        let host = host.strip_prefix('[').and_then(|host| host.strip_suffix(']')).unwrap_or(host);
        if bracketed && host.parse::<std::net::Ipv6Addr>().is_err() {
            return Err("bracketed pairing hosts must be IPv6 addresses".into());
        }
        match host.parse::<std::net::IpAddr>() {
            Ok(ip) if ip.is_unspecified() || ip.is_multicast() => return Err("pairing endpoint must be a concrete host".into()),
            Ok(_) => {}
            Err(_) if host.len() <= 253 && host.split('.').all(valid_dns_label) => {}
            Err(_) => return Err("invalid pairing endpoint host".into()),
        }
        match (scheme, &self.ca_certificate_pem) {
            ("https", Some(pem)) => validate_certificate_pem(pem)?,
            ("http", None) => {}
            _ => return Err("HTTPS requires a CA certificate; HTTP must not include one".into()),
        }
        if self.access_key.as_ref().is_some_and(|key| !(32..=4096).contains(&key.len()) || !key.bytes().all(|byte| byte.is_ascii_graphic()))
        {
            return Err("invalid pairing access key".into());
        }
        let bytes = serde_json::to_vec(self).map_err(|_| "cannot encode pairing file")?;
        if bytes.len() > 1_048_576 {
            return Err("pairing file exceeds 1 MiB".into());
        }
        Ok(())
    }
}

impl std::fmt::Debug for PairingFile {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PairingFile")
            .field("version", &self.version)
            .field("endpoint", &self.endpoint)
            .field("backend_identifier", &self.backend_identifier)
            .field("label", &self.label)
            .field("access_key", &self.access_key.as_ref().map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for CreatedPairing {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("CreatedPairing").field("client", &self.client).field("pairing", &self.pairing).finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn required_optional<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(deserializer)
}

fn validate_certificate_pem(pem: &str) -> Result<(), String> {
    use base64::Engine as _;
    if pem.len() > 262_144 {
        return Err("pairing certificate is too large".into());
    }
    let body = pem
        .trim()
        .strip_prefix("-----BEGIN CERTIFICATE-----")
        .and_then(|body| body.strip_suffix("-----END CERTIFICATE-----"))
        .ok_or("invalid CA certificate PEM")?;
    let body: String = body.chars().filter(|character| !character.is_ascii_whitespace()).collect();
    let der = base64::engine::general_purpose::STANDARD.decode(body).map_err(|_| "invalid CA certificate PEM")?;
    // Check The DER Envelope; Transport Imports Validate X.509 And Trust.
    if der.len() < 4 || der[0] != 0x30 {
        return Err("invalid CA certificate DER".into());
    }
    let (header, length) = if der[1] & 0x80 == 0 {
        (2, usize::from(der[1]))
    } else {
        let count = usize::from(der[1] & 0x7f);
        if count == 0 || count > 4 || der.len() < 2 + count {
            return Err("invalid CA certificate DER length".into());
        }
        let length = der[2..2 + count].iter().fold(0usize, |length, byte| (length << 8) | usize::from(*byte));
        (2 + count, length)
    };
    if length != der.len() - header || der.get(header) != Some(&0x30) {
        return Err("invalid CA certificate DER envelope".into());
    }
    Ok(())
}

fn valid_dns_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
        && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn lab() -> PairingFile {
        PairingFile {
            version: 1,
            endpoint: "http://127.0.0.1:8765".into(),
            backend_identifier: "backend-id".into(),
            label: "Lab".into(),
            ca_certificate_pem: None,
            access_key: None,
        }
    }

    #[test]
    fn pairing_shape_is_exact_and_secret_debug_is_redacted() {
        let mut pairing = lab();
        pairing.access_key = Some("a".repeat(64));
        pairing.validate().unwrap();
        let encoded = serde_json::to_value(&pairing).unwrap();
        assert_eq!(encoded.as_object().unwrap().len(), 6);
        assert!(!format!("{pairing:?}").contains(pairing.access_key.as_deref().unwrap()));
        let decoded: PairingFile = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(pairing, decoded);
        for version in [serde_json::json!(0), serde_json::json!(2), serde_json::json!(256), serde_json::json!(u32::MAX)] {
            let mut unsupported = encoded.clone();
            unsupported["version"] = version;
            let decoded: PairingFile = serde_json::from_value(unsupported).unwrap();
            assert!(decoded.validate().is_err());
        }
        for version in
            [serde_json::json!(-1), serde_json::json!(1.5), serde_json::json!("1"), serde_json::json!(true), serde_json::json!(u64::MAX)]
        {
            let mut malformed = encoded.clone();
            malformed["version"] = version;
            assert!(serde_json::from_value::<PairingFile>(malformed).is_err());
        }

        let mut unknown = encoded.clone();
        unknown["key"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<PairingFile>(unknown).is_err());
        for field in ["ca_certificate_pem", "access_key"] {
            let mut missing = encoded.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<PairingFile>(missing).is_err());
        }
    }

    #[test]
    fn pairing_rejects_invalid_origins_versions_and_tls() {
        for endpoint in [
            "http://0.0.0.0:8765",
            "http://[::]:8765",
            "http://*.example:8765",
            "http://host:0",
            "https://host:8765",
            "ftp://host:8765",
            "http://host:8765/path",
            "http://user@host:8765",
            "http://::1:8765",
            "http://host:8765?key=value",
        ] {
            let mut pairing = lab();
            pairing.endpoint = endpoint.into();
            assert!(pairing.validate().is_err(), "{endpoint}");
        }
        let mut pairing = lab();
        pairing.version = 2;
        assert!(pairing.validate().is_err());
        pairing = lab();
        pairing.backend_identifier = " ".into();
        assert!(pairing.validate().is_err());
        pairing = lab();
        pairing.ca_certificate_pem = Some("invalid".into());
        assert!(pairing.validate().is_err());
        pairing.endpoint = "https://localhost:8765".into();
        assert!(pairing.validate().is_err());
    }
}
