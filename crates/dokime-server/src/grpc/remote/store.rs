/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Atomic Remote Security Material Persistence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    net::{IpAddr, SocketAddr},
    path::Path,
};

// Third Party Imports
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

// Local Imports
use dokime::domain::model::remote::RemoteListenerSettings;

/* === Constants === */

const MAXIMUM_BYTES: usize = 256 * 1024;
const MAXIMUM_NAMES: usize = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Document {
    pub version: u32,
    pub settings: RemoteListenerSettings,
    pub key_digest: Option<[u8; 32]>,
    pub identity: Option<Identity>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub certificate: String,
    pub private_key: String,
    pub names: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Document {
    pub fn initial() -> Self {
        Self {
            version: 1,
            settings: RemoteListenerSettings {
                enabled: false,
                address: "0.0.0.0:8765".into(),
                tls_enabled: true,
                authentication_enabled: true,
                certificate_names: vec![],
            },
            key_digest: None,
            identity: None,
        }
    }
}

impl Identity {
    pub fn generate(names: Vec<String>) -> Result<Self, super::Error> {
        if names.is_empty() {
            return Err(super::Error::Configuration("TLS requires at least one advertised DNS name or IP address"));
        }
        let generated = rcgen::generate_simple_self_signed(names.clone())
            .map_err(|_| super::Error::Configuration("cannot generate remote TLS identity"))?;
        Ok(Self { certificate: generated.cert.pem(), private_key: generated.signing_key.serialize_pem(), names })
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

/* === Public Functions === */

pub(super) fn read(path: &Path) -> Result<Document, super::Error> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Document::initial()),
        Err(error) => return Err(super::Error::RemoteStorage(error)),
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(super::Error::Configuration("remote security must be a regular file"));
        }
        Ok(_) => {}
    }
    let bytes = Zeroizing::new(harmos::filesystem::input::read(path, MAXIMUM_BYTES).map_err(super::Error::RemoteStorage)?);
    let document: Document = serde_json::from_slice(&bytes).map_err(|_| super::Error::Configuration("invalid remote security document"))?;
    if document.version != 1 {
        return Err(super::Error::Configuration("unsupported remote security document version"));
    }
    validate(&document.settings)?;
    if let Some(identity) = &document.identity {
        let mut settings = document.settings.clone();
        settings.certificate_names = identity.names.clone();
        validate(&settings)?;
        if settings.certificate_names != document.settings.certificate_names {
            return Err(super::Error::Configuration("persisted certificate names differ from listener settings"));
        }
    }
    Ok(document)
}

pub(super) fn write(path: &Path, document: &Document) -> Result<(), super::Error> {
    let parent = path.parent().ok_or(super::Error::Configuration("remote security path has no parent"))?;
    private_directory(parent).map_err(super::Error::RemoteStorage)?;
    let bytes =
        Zeroizing::new(serde_json::to_vec(document).map_err(|_| super::Error::Configuration("cannot encode remote security document"))?);
    if bytes.len() > MAXIMUM_BYTES {
        return Err(super::Error::Configuration("remote security document exceeds its byte limit"));
    }

    harmos::filesystem::publication::write(path, &bytes)?;
    Ok(())
}

pub(super) fn validate(settings: &RemoteListenerSettings) -> Result<SocketAddr, super::Error> {
    let address = settings
        .address
        .parse::<SocketAddr>()
        .map_err(|_| super::Error::Configuration("listener address must be an IP address and port"))?;
    if settings.certificate_names.len() > MAXIMUM_NAMES {
        return Err(super::Error::Configuration("at most 32 certificate identities are allowed"));
    }
    for (index, name) in settings.certificate_names.iter().enumerate() {
        if name.len() > 253 || name.is_empty() || settings.certificate_names[..index].contains(name) {
            return Err(super::Error::Configuration("certificate identities must be distinct bounded DNS names or IP addresses"));
        }
        if name.parse::<IpAddr>().is_ok() {
            continue;
        }
        if !valid_dns(name) {
            return Err(super::Error::Configuration(
                "certificate identities must be exact DNS names or IP addresses without wildcards or ports",
            ));
        }
    }
    Ok(address)
}

// Creates the owner-only directory that holds remote security material and the session log.
pub(super) fn private_directory(path: &Path) -> std::io::Result<()> {
    let mut directory = std::fs::DirBuilder::new();
    directory.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        directory.mode(0o700);
    }
    directory.create(path)
}

// The identities this host answers to: the listen address when it is specific, the host name
// and every non-loopback interface address, falling back to loopback when nothing else exists.
pub(super) fn host_names(address: SocketAddr) -> Vec<String> {
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let interfaces = networks.values().flat_map(|network| network.ip_networks().iter().map(|network| network.addr));
    default_names(address, sysinfo::System::host_name(), interfaces)
}

/* === Private Functions === */

fn default_names(address: SocketAddr, host_name: Option<String>, interfaces: impl Iterator<Item = IpAddr>) -> Vec<String> {
    let reachable = |ip: &IpAddr| !ip.is_loopback() && !ip.is_unspecified() && !matches!(ip, IpAddr::V6(v6) if v6.is_unicast_link_local());
    let host =
        host_name.map(|name| name.trim().trim_end_matches('.').to_ascii_lowercase()).filter(|name| name.len() <= 253 && valid_dns(name));
    let addresses = std::iter::once(address.ip()).chain(interfaces).filter(reachable).map(|ip| ip.to_string());
    let mut names: Vec<String> = Vec::new();
    for name in host.into_iter().chain(addresses) {
        if names.len() < MAXIMUM_NAMES && !names.contains(&name) {
            names.push(name);
        }
    }
    if names.is_empty() {
        names = vec!["localhost".into(), "127.0.0.1".into()];
    }
    names
}

fn valid_dns(name: &str) -> bool {
    name.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            && label.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
            && label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persistent_identity_and_verifier() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("remote/access.json");
        let mut document = read(&path).unwrap();
        document.settings.certificate_names = vec!["localhost".into(), "127.0.0.1".into()];
        document.identity = Some(Identity::generate(document.settings.certificate_names.clone()).unwrap());
        document.key_digest = Some([17; 32]);
        write(&path, &document).unwrap();
        let before = std::fs::read(&path).unwrap();
        let restored = read(&path).unwrap();
        assert_eq!(restored.settings, document.settings);
        assert_eq!(restored.key_digest, document.key_digest);
        assert_eq!(restored.identity.as_ref().unwrap().certificate, document.identity.as_ref().unwrap().certificate);
        assert!(restored.identity.as_ref().unwrap().private_key == document.identity.as_ref().unwrap().private_key);
        assert!(serde_json::from_slice::<serde_json::Value>(&before).unwrap().get("access_key").is_none());
        write(&path, &restored).unwrap();
        assert!(std::fs::read(&path).unwrap() == before);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(std::fs::metadata(path.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
        }
    }

    #[test]
    fn invalid_explicit_security_is_not_reset() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("access.json");
        std::fs::write(&path, b"invalid private material").unwrap();
        let error = read(&path).err().unwrap();
        assert!(!error.to_string().contains("private material"));
        assert_eq!(std::fs::read(&path).unwrap(), b"invalid private material");
        assert!(read(root.path()).is_err());
        let mut document = Document::initial();
        document.version = 2;
        write(&path, &document).unwrap();
        assert!(read(&path).is_err());
        std::fs::write(&path, vec![b'a'; MAXIMUM_BYTES + 1]).unwrap();
        assert!(read(&path).is_err());
    }

    #[test]
    fn certificate_identity_validation() {
        let mut settings = Document::initial().settings;
        for name in ["localhost", "lab.example", "192.0.2.3", "2001:db8::1"] {
            settings.certificate_names = vec![name.into()];
            validate(&settings).unwrap();
        }
        for name in ["", "*.example", "https://lab.example", "host:8765", "host/path", "-host", "host-", "host..example", " host", "host\n"]
        {
            settings.certificate_names = vec![name.into()];
            assert!(validate(&settings).is_err(), "{name:?}");
        }
        settings.certificate_names = vec!["localhost".into(), "localhost".into()];
        assert!(validate(&settings).is_err());
    }
    #[test]
    fn default_certificate_identities() {
        let wildcard: SocketAddr = "0.0.0.0:8765".parse().unwrap();
        let interfaces = ["127.0.0.1", "192.168.1.20", "fe80::1", "192.168.1.20", "2001:db8::7"].map(|ip| ip.parse().unwrap());
        let names = default_names(wildcard, Some("Lab-Host.".into()), interfaces.into_iter());
        assert_eq!(names, ["lab-host", "192.168.1.20", "2001:db8::7"]);
        let mut settings = Document::initial().settings;
        settings.certificate_names = names;
        validate(&settings).unwrap();
        let specific: SocketAddr = "10.0.0.5:8765".parse().unwrap();
        assert_eq!(default_names(specific, Some("bad_name".into()), std::iter::empty()), ["10.0.0.5"]);
        assert_eq!(default_names(wildcard, None, std::iter::empty()), ["localhost", "127.0.0.1"]);
        let many = (0..64u8).map(|index| IpAddr::from([10, 0, 1, index]));
        assert_eq!(default_names(wildcard, None, many).len(), MAXIMUM_NAMES);
    }
}
