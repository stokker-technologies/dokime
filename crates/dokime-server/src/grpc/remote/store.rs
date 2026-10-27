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

const SERVER_REMOTE_STORE_BYTES: u64 = 262_144;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Document {
    pub version: u32,
    pub settings: RemoteListenerSettings,
    pub clients: Vec<Client>,
    pub identity: Option<Identity>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Client {
    pub metadata: dokime::domain::model::remote::PairedClient,
    pub verifier: [u8; 32],
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    pub certificate: String,
    pub private_key: String,
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Security {
    version: u32,
    clients: Vec<Client>,
    identity: Option<Identity>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Document {
    pub fn initial() -> Self {
        Self {
            version: 1,
            settings: dokime::domain::model::settings::wire::ApplicationSettings::default().listener,
            clients: Vec::new(),
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

pub(super) fn read(path: &Path, settings: RemoteListenerSettings) -> Result<Document, super::Error> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Document { settings, ..Document::initial() });
        }
        Err(error) => return Err(super::Error::RemoteStorage(error)),
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(super::Error::Configuration("remote security must be a regular file"));
        }
        Ok(_) => {}
    }
    let bytes = Zeroizing::new(
        dokime::harmos::filesystem::input::read(path, usize::try_from(SERVER_REMOTE_STORE_BYTES).expect("validated remote security size"))
            .map_err(super::Error::RemoteStorage)?,
    );
    let security: Security = serde_json::from_slice(&bytes)
        .map_err(|_| super::Error::Configuration("incompatible remote security document; use a fresh application root"))?;
    let document = Document { version: security.version, settings, clients: security.clients, identity: security.identity };
    if document.version != 1 {
        return Err(super::Error::Configuration("unsupported remote security document version"));
    }
    validate(&document.settings)?;
    validate_clients(&document.clients)?;
    Ok(document)
}

pub(super) fn write(path: &Path, document: &Document) -> Result<(), super::Error> {
    validate_clients(&document.clients)?;
    let parent = path.parent().ok_or(super::Error::Configuration("remote security path has no parent"))?;
    private_directory(parent).map_err(super::Error::RemoteStorage)?;
    let bytes = Zeroizing::new(
        serde_json::to_vec(&Security { version: document.version, clients: document.clients.clone(), identity: document.identity.clone() })
            .map_err(|_| super::Error::Configuration("cannot encode remote security document"))?,
    );
    if bytes.len() as u64 > SERVER_REMOTE_STORE_BYTES {
        return Err(super::Error::Configuration("remote security document exceeds its byte limit"));
    }

    dokime::harmos::filesystem::publication::write(path, &bytes)?;
    Ok(())
}

pub(super) fn validate_clients(clients: &[Client]) -> Result<(), super::Error> {
    if clients.len() > 512 {
        return Err(super::Error::Configuration("paired client limit reached; revoke an unused client"));
    }
    for (index, client) in clients.iter().enumerate() {
        let metadata = &client.metadata;
        if uuid::Uuid::parse_str(&metadata.identifier).is_err()
            || metadata.label.trim().is_empty()
            || metadata.label.len() > 256
            || metadata.label.chars().any(char::is_control)
            || clients[..index].iter().any(|other| other.metadata.identifier == metadata.identifier || other.verifier == client.verifier)
        {
            return Err(super::Error::Configuration("invalid paired client security record"));
        }
    }
    Ok(())
}

pub(super) fn validate(settings: &RemoteListenerSettings) -> Result<SocketAddr, super::Error> {
    settings.validate().map_err(super::Error::Listener)?;
    settings.address.parse::<SocketAddr>().map_err(|_| super::Error::Configuration("invalid listener address"))
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
pub(super) fn host_names(address: SocketAddr, maximum: usize) -> Vec<String> {
    let networks = sysinfo::Networks::new_with_refreshed_list();
    let interfaces = networks.values().flat_map(|network| network.ip_networks().iter().map(|network| network.addr));
    default_names(address, sysinfo::System::host_name(), interfaces, maximum)
}

/* === Private Functions === */

fn default_names(address: SocketAddr, host_name: Option<String>, interfaces: impl Iterator<Item = IpAddr>, maximum: usize) -> Vec<String> {
    let reachable = |ip: &IpAddr| !ip.is_loopback() && !ip.is_unspecified() && !matches!(ip, IpAddr::V6(v6) if v6.is_unicast_link_local());
    let host =
        host_name.map(|name| name.trim().trim_end_matches('.').to_ascii_lowercase()).filter(|name| name.len() <= 253 && valid_dns(name));
    let addresses = std::iter::once(address.ip()).chain(interfaces).filter(reachable).map(|ip| ip.to_string());
    let mut names: Vec<String> = Vec::new();
    for name in host.into_iter().chain(addresses) {
        if names.len() < maximum && !names.contains(&name) {
            names.push(name);
        }
    }
    if names.is_empty() {
        names = vec!["localhost".into(), "127.0.0.1".into()];
    }
    names.truncate(maximum);
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
    use crate::grpc::remote::SERVER_REMOTE_NAMES;

    #[test]
    fn persistent_identity_and_verifier() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("remote/access.json");
        let mut document = read(&path, Document::initial().settings).unwrap();
        document.settings.certificate_names = vec!["localhost".into(), "127.0.0.1".into()];
        document.identity = Some(Identity::generate(document.settings.certificate_names.clone()).unwrap());
        document.clients.push(Client {
            metadata: dokime::domain::model::remote::PairedClient {
                identifier: uuid::Uuid::new_v4().to_string(),
                label: "Lab".into(),
                created_at: 1,
                last_seen_at: None,
            },
            verifier: [17; 32],
        });
        write(&path, &document).unwrap();
        let before = std::fs::read(&path).unwrap();
        let restored = read(&path, Document::initial().settings).unwrap();
        assert_eq!(restored.clients[0].verifier, document.clients[0].verifier);
        assert_eq!(restored.identity.as_ref().unwrap().certificate, document.identity.as_ref().unwrap().certificate);
        assert!(restored.identity.as_ref().unwrap().private_key == document.identity.as_ref().unwrap().private_key);
        let security = serde_json::from_slice::<serde_json::Value>(&before).unwrap();
        assert!(security.get("access_key").is_none());
        assert!(security.get("settings").is_none());
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
        let error = read(&path, Document::initial().settings).err().unwrap();
        assert!(!error.to_string().contains("private material"));
        assert_eq!(std::fs::read(&path).unwrap(), b"invalid private material");
        assert!(read(root.path(), Document::initial().settings).is_err());
        let mut document = Document::initial();
        document.version = 2;
        write(&path, &document).unwrap();
        assert!(read(&path, Document::initial().settings).is_err());
        std::fs::write(&path, vec![b'a'; usize::try_from(SERVER_REMOTE_STORE_BYTES).unwrap() + 1]).unwrap();
        assert!(read(&path, Document::initial().settings).is_err());
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
        let names =
            default_names(wildcard, Some("Lab-Host.".into()), interfaces.into_iter(), usize::try_from(SERVER_REMOTE_NAMES).unwrap());
        assert_eq!(names, ["lab-host", "192.168.1.20", "2001:db8::7"]);
        let mut settings = Document::initial().settings;
        settings.certificate_names = names;
        validate(&settings).unwrap();
        let specific: SocketAddr = "10.0.0.5:8765".parse().unwrap();
        assert_eq!(
            default_names(specific, Some("bad_name".into()), std::iter::empty(), usize::try_from(SERVER_REMOTE_NAMES).unwrap()),
            ["10.0.0.5"]
        );
        assert_eq!(
            default_names(wildcard, None, std::iter::empty(), usize::try_from(SERVER_REMOTE_NAMES).unwrap()),
            ["localhost", "127.0.0.1"]
        );
        let many = (0..64u8).map(|index| IpAddr::from([10, 0, 1, index]));
        assert_eq!(
            default_names(wildcard, None, many, usize::try_from(SERVER_REMOTE_NAMES).unwrap()).len(),
            usize::try_from(SERVER_REMOTE_NAMES).unwrap()
        );
    }
}
