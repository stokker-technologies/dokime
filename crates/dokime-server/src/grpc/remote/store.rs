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
use dokime::domain::model::remote::{RemoteEndpoint, RemoteListenerSettings};

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
    pub ca_certificate: String,
    pub ca_private_key: String,
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
        let mut params = rcgen::CertificateParams::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![rcgen::KeyUsagePurpose::KeyCertSign, rcgen::KeyUsagePurpose::CrlSign];
        params.distinguished_name.push(rcgen::DnType::CommonName, "Dokime Remote Access CA");
        let key = rcgen::KeyPair::generate().map_err(|_| super::Error::Configuration("cannot generate remote CA key"))?;
        let certificate = params.self_signed(&key).map_err(|_| super::Error::Configuration("cannot generate remote CA"))?;
        let mut identity = Self {
            ca_certificate: certificate.pem(),
            ca_private_key: key.serialize_pem(),
            certificate: String::new(),
            private_key: String::new(),
            names: Vec::new(),
        };
        identity.reissue(names)?;
        Ok(identity)
    }

    pub fn reissue(&mut self, names: Vec<String>) -> Result<(), super::Error> {
        let invalid = |_| super::Error::Configuration("cannot issue remote TLS certificate from persisted CA");
        let key = rcgen::KeyPair::from_pem(&self.ca_private_key).map_err(invalid)?;
        let issuer = rcgen::Issuer::from_ca_cert_pem(&self.ca_certificate, key).map_err(invalid)?;
        let leaf_key = rcgen::KeyPair::generate().map_err(invalid)?;
        let mut params = rcgen::CertificateParams::new(names.clone()).map_err(invalid)?;
        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
        let leaf = params.signed_by(&leaf_key, &issuer).map_err(invalid)?;
        self.private_key.zeroize();
        self.certificate = leaf.pem();
        self.private_key = leaf_key.serialize_pem();
        self.names = names;
        Ok(())
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        self.private_key.zeroize();
        self.ca_private_key.zeroize();
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
    let document = Document {
        version: security.version,
        settings,
        clients: security.clients,
        identity: security.identity,
    };
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
        serde_json::to_vec(&Security {
            version: document.version,
            clients: document.clients.clone(),
            identity: document.identity.clone(),
        })
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
// and ranked real interface addresses; virtual adapters are excluded. Loopback is same-PC only.
pub(super) fn host_names(address: SocketAddr, maximum: usize) -> Vec<String> {
    host_endpoints(address, maximum).into_iter().map(|endpoint| endpoint.address).collect()
}

pub(super) fn host_endpoints(address: SocketAddr, maximum: usize) -> Vec<RemoteEndpoint> {
    let networks = sysinfo::Networks::new_with_refreshed_list().ok();
    let interfaces = networks
        .iter()
        .flat_map(|networks| networks.iter())
        .flat_map(|(name, network)| network.ip_networks().iter().map(move |network| (name.to_str().unwrap_or(""), network.addr)));
    detected_endpoints(address, sysinfo::System::host_name().ok(), interfaces, maximum)
}

/* === Private Functions === */

fn detected_endpoints<'a>(
    address: SocketAddr, host_name: Option<String>, interfaces: impl Iterator<Item = (&'a str, IpAddr)>, maximum: usize,
) -> Vec<RemoteEndpoint> {
    if matches!(address.ip(), IpAddr::V6(ip) if ip.is_unicast_link_local()) {
        return Vec::new();
    }
    let interfaces: Vec<_> = interfaces.collect();
    if !address.ip().is_unspecified() {
        let adapter_name = interfaces.iter().find(|(_, ip)| *ip == address.ip()).map(|(name, _)| (*name).to_owned());
        return vec![RemoteEndpoint {
            address: address.ip().to_string(),
            adapter_name,
        }];
    }
    let host =
        host_name.map(|name| name.trim().trim_end_matches('.').to_ascii_lowercase()).filter(|name| name.len() <= 253 && valid_dns(name));
    let mut addresses: Vec<_> = interfaces
        .into_iter()
        .filter_map(|(name, ip)| {
            let adapter = name.to_ascii_lowercase();
            let virtual_adapter = [
                "wsl",
                "hyper-v",
                "vethernet",
                "docker",
                "loopback",
                "vpn",
                "tun",
                "tap",
                "tailscale",
                "zerotier",
                "wireguard",
                "vmware",
                "virtualbox",
                "vbox",
                "virbr",
                "veth",
                "br-",
            ]
            .iter()
            .any(|pattern| adapter.contains(pattern));
            if virtual_adapter
                || ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || address.is_ipv4() != ip.is_ipv4()
                || matches!(ip, IpAddr::V6(v6) if v6.is_unicast_link_local())
            {
                return None;
            }
            Some((ip, name.to_owned()))
        })
        .collect();
    addresses.sort_unstable_by_key(|(ip, adapter)| {
        let private = match ip {
            IpAddr::V4(ip) => ip.is_private(),
            IpAddr::V6(ip) => ip.is_unique_local(),
        };
        let link_local = matches!(ip, IpAddr::V4(ip) if ip.is_link_local());
        (link_local, !private, *ip, adapter.clone())
    });
    let mut names = Vec::new();
    for (name, adapter_name) in
        host.into_iter().map(|host| (host, None)).chain(addresses.into_iter().map(|(ip, adapter)| (ip.to_string(), Some(adapter))))
    {
        if names.len() < maximum && !names.iter().any(|endpoint: &RemoteEndpoint| endpoint.address == name) {
            names.push(RemoteEndpoint { address: name, adapter_name });
        }
    }
    if names.is_empty() {
        names.push(RemoteEndpoint {
            address: if address.is_ipv4() { "127.0.0.1" } else { "::1" }.into(),
            adapter_name: None,
        });
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

    fn default_names<'a>(
        address: SocketAddr, host_name: Option<String>, interfaces: impl Iterator<Item = (&'a str, IpAddr)>, maximum: usize,
    ) -> Vec<String> {
        detected_endpoints(address, host_name, interfaces, maximum).into_iter().map(|endpoint| endpoint.address).collect()
    }

    #[test]
    fn persistent_identity_and_verifier() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("remote/access.json");
        let mut document = read(&path, Document::initial().settings).unwrap();
        document.settings.certificate_names = vec!["localhost".into(), "127.0.0.1".into()];
        document.identity = Some(Identity::generate(document.settings.certificate_names.clone()).unwrap());
        document.clients.push(Client {
            metadata: dokime::domain::model::remote::PairedClient {
                identifier: uuid::Uuid::now_v7().to_string(),
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
        let names = default_names(
            wildcard,
            Some("Lab-Host.".into()),
            interfaces.into_iter().map(|ip| ("Ethernet", ip)),
            usize::try_from(SERVER_REMOTE_NAMES).unwrap(),
        );
        assert_eq!(names, ["lab-host", "192.168.1.20"]);
        let mut settings = Document::initial().settings;
        settings.certificate_names = names;
        validate(&settings).unwrap();
        let specific: SocketAddr = "10.0.0.5:8765".parse().unwrap();
        assert_eq!(
            default_names(specific, Some("bad_name".into()), std::iter::empty(), usize::try_from(SERVER_REMOTE_NAMES).unwrap()),
            ["10.0.0.5"]
        );
        assert_eq!(default_names(wildcard, None, std::iter::empty(), usize::try_from(SERVER_REMOTE_NAMES).unwrap()), ["127.0.0.1"]);
        let many = (0..64u8).map(|index| IpAddr::from([10, 0, 1, index]));
        assert_eq!(
            default_names(wildcard, None, many.map(|ip| ("Ethernet", ip)), usize::try_from(SERVER_REMOTE_NAMES).unwrap()).len(),
            usize::try_from(SERVER_REMOTE_NAMES).unwrap()
        );
    }
    #[test]
    fn physical_interfaces_rank_before_public_and_virtual_adapters_are_excluded() {
        let interfaces = [
            ("Docker", "172.17.0.1"),
            ("vEthernet (WSL)", "172.25.0.1"),
            ("Hyper-V", "10.1.1.1"),
            ("WireGuard VPN", "10.5.0.1"),
            ("Loopback", "127.0.0.1"),
            ("Ethernet", "203.0.113.8"),
            ("Wi-Fi", "192.168.1.20"),
            ("Ethernet", "10.0.0.2"),
            ("Cable Ethernet", "169.254.23.41"),
            ("Virtual VPN", "169.254.8.9"),
            ("Cable Ethernet", "fe80::1234"),
        ]
        .into_iter()
        .map(|(name, ip)| (name, ip.parse().unwrap()));
        assert_eq!(
            default_names("0.0.0.0:8765".parse().unwrap(), Some("Lab".into()), interfaces, 32),
            ["lab", "10.0.0.2", "192.168.1.20", "203.0.113.8", "169.254.23.41"]
        );
        assert_eq!(default_names("192.168.1.20:8765".parse().unwrap(), Some("Lab".into()), std::iter::empty(), 32), ["192.168.1.20"]);
    }

    #[test]
    fn cable_certificate_endpoints() {
        use rustls::{
            client::danger::ServerCertVerifier,
            pki_types::{CertificateDer, ServerName, UnixTime, pem::PemObject},
        };

        let detect = |cable| {
            detected_endpoints(
                "0.0.0.0:8765".parse().unwrap(),
                Some("Bench".into()),
                [("Wireless", "192.168.7.4"), ("Cable Ethernet", cable), ("Cable Ethernet", cable)]
                    .into_iter()
                    .map(|(name, ip)| (name, ip.parse().unwrap())),
                32,
            )
        };
        let hosts = detect("169.254.23.41");
        assert_eq!(
            hosts.iter().map(|host| (host.address.as_str(), host.adapter_name.as_deref())).collect::<Vec<_>>(),
            [("bench", None), ("192.168.7.4", Some("Wireless")), ("169.254.23.41", Some("Cable Ethernet"))]
        );
        let specific = detected_endpoints(
            "169.254.23.41:8765".parse().unwrap(),
            None,
            [("Cable Ethernet", "169.254.23.41".parse().unwrap())].into_iter(),
            32,
        );
        assert_eq!(specific, vec![hosts[2].clone()]);
        let cable_only = detected_endpoints(
            "0.0.0.0:8765".parse().unwrap(),
            None,
            [("Cable Ethernet", "169.254.23.41".parse().unwrap())].into_iter(),
            32,
        );
        assert_eq!(cable_only, specific);
        let ipv6 =
            [("Cable Ethernet", "fe80::1234"), ("Ethernet", "2001:db8::7")].into_iter().map(|(name, ip)| (name, ip.parse().unwrap()));
        assert_eq!(default_names("[::]:8765".parse().unwrap(), None, ipv6, 32), ["2001:db8::7"]);
        assert!(detected_endpoints("[fe80::1234]:8765".parse().unwrap(), None, std::iter::empty(), 32).is_empty());

        let mut identity = Identity::generate(hosts.iter().map(|host| host.address.clone()).collect()).unwrap();
        let ca = identity.ca_certificate.clone();
        let mut roots = rustls::RootCertStore::empty();
        roots.add(CertificateDer::from_pem_slice(ca.as_bytes()).unwrap()).unwrap();
        let verifier = rustls::client::WebPkiServerVerifier::builder_with_provider(
            std::sync::Arc::new(roots),
            std::sync::Arc::new(rustls::crypto::ring::default_provider()),
        )
        .build()
        .unwrap();
        let now = UnixTime::since_unix_epoch(std::time::Duration::from_secs(1_800_000_000));
        for hosts in [hosts, detect("169.254.23.42")] {
            let names = hosts.iter().map(|host| host.address.clone()).collect();
            identity.reissue(names).unwrap();
            assert_eq!(identity.ca_certificate, ca);
            let leaf = CertificateDer::from_pem_slice(identity.certificate.as_bytes()).unwrap();
            let origins = super::super::pairing::endpoints(hosts, 8765, true);
            for origin in origins {
                let host = origin.strip_prefix("https://").unwrap().rsplit_once(':').unwrap().0;
                let name = ServerName::try_from(host).unwrap();
                verifier.verify_server_cert(&leaf, &[], &name, &[], now).unwrap();
            }
            let obsolete = ServerName::try_from("169.254.23.99").unwrap();
            assert!(verifier.verify_server_cert(&leaf, &[], &obsolete, &[], now).is_err());
        }
    }

    #[test]
    fn leaf_reissue_preserves_persistent_ca() {
        let mut identity = Identity::generate(vec!["localhost".into()]).unwrap();
        let ca = identity.ca_certificate.clone();
        let ca_key = identity.ca_private_key.clone();
        let leaf = identity.certificate.clone();
        identity.reissue(vec!["localhost".into(), "192.168.1.21".into()]).unwrap();
        assert_eq!(identity.ca_certificate, ca);
        assert!(identity.ca_private_key == ca_key);
        assert_ne!(identity.certificate, leaf);
        let key = rcgen::KeyPair::from_pem(&ca_key).unwrap();
        assert!(rcgen::Issuer::from_ca_cert_pem(&ca, key).is_ok());
        let root = tempfile::tempdir().unwrap();
        let mut document = Document::initial();
        document.identity = Some(identity);
        let path = root.path().join("access.json");
        write(&path, &document).unwrap();
        let mut restored = read(&path, document.settings).unwrap().identity.unwrap();
        restored.reissue(vec!["localhost".into(), "192.168.1.22".into()]).unwrap();
        assert_eq!(restored.ca_certificate, ca);
    }
}
