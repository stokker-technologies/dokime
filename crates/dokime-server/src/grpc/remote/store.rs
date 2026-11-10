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

pub(super) const SERVER_REMOTE_STORE_BYTES: u64 = 262_144;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(super) struct Document {
    pub settings: RemoteListenerSettings,
    pub verifier: Option<[u8; 32]>,
    pub identity: Option<Identity>,
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
    verifier: Option<[u8; 32]>,
    identity: Option<Identity>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Document {
    pub fn initial() -> Self {
        Self {
            settings: dokime::domain::model::settings::wire::ApplicationSettings::default().listener,
            verifier: None,
            identity: None,
        }
    }
}

impl Identity {
    pub fn generate(names: Vec<String>) -> Result<Self, super::Error> {
        if names.is_empty() {
            return Err(super::Error::Configuration("TLS requires at least one advertised DNS name or IP address"));
        }
        let mut params =
            rcgen::CertificateParams::new(names.clone()).map_err(|_| super::Error::Configuration("invalid remote certificate names"))?;
        params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        params.key_usages = vec![rcgen::KeyUsagePurpose::DigitalSignature];
        let pack =
            dokime::config::organization::resolve().map_err(|_| super::Error::Configuration("cannot resolve certificate product label"))?;
        params.distinguished_name.push(rcgen::DnType::CommonName, format!("{} Remote Access", pack.label()));
        let key = rcgen::KeyPair::generate().map_err(|_| super::Error::Configuration("cannot generate remote TLS key"))?;
        let certificate = params.self_signed(&key).map_err(|_| super::Error::Configuration("cannot generate remote certificate"))?;
        Ok(Self {
            certificate: certificate.pem(),
            private_key: key.serialize_pem(),
            names,
        })
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
    let document = Document {
        settings,
        verifier: security.verifier,
        identity: security.identity,
    };
    validate(&document.settings)?;
    Ok(document)
}

pub(super) fn write(path: &Path, document: &Document) -> Result<(), super::Error> {
    let parent = path.parent().ok_or(super::Error::Configuration("remote security path has no parent"))?;
    private_directory(parent).map_err(super::Error::RemoteStorage)?;
    let bytes = Zeroizing::new(
        serde_json::to_vec(&Security {
            verifier: document.verifier,
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

pub(super) fn validate(settings: &RemoteListenerSettings) -> Result<SocketAddr, super::Error> {
    settings.validate().map_err(super::Error::Listener)?;
    settings.address.parse::<SocketAddr>().map_err(|_| super::Error::Configuration("invalid listener address"))
}

// Creates the owner-only directory that holds remote security material.
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

    fn default_names<'a>(
        address: SocketAddr, host_name: Option<String>, interfaces: impl Iterator<Item = (&'a str, IpAddr)>, maximum: usize,
    ) -> Vec<String> {
        detected_endpoints(address, host_name, interfaces, maximum).into_iter().map(|endpoint| endpoint.address).collect()
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
        let names = default_names(wildcard, Some("Lab-Host.".into()), interfaces.into_iter().map(|ip| ("Ethernet", ip)), 32);
        assert_eq!(names, ["lab-host", "192.168.1.20"]);
        let mut settings = Document::initial().settings;
        settings.certificate_names = names;
        validate(&settings).unwrap();
        let specific: SocketAddr = "10.0.0.5:8765".parse().unwrap();
        assert_eq!(default_names(specific, Some("bad_name".into()), std::iter::empty(), 32), ["10.0.0.5"]);
        assert_eq!(default_names(wildcard, None, std::iter::empty(), 32), ["127.0.0.1"]);
        let many = (0..64u8).map(|index| IpAddr::from([10, 0, 1, index]));
        assert_eq!(default_names(wildcard, None, many.map(|ip| ("Ethernet", ip)), 32).len(), 32);
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

        for hosts in [hosts, detect("169.254.23.42")] {
            let identity = Identity::generate(hosts.iter().map(|host| host.address.clone()).collect()).unwrap();
            let mut roots = rustls::RootCertStore::empty();
            let certificate = CertificateDer::from_pem_slice(identity.certificate.as_bytes()).unwrap();
            roots.add(certificate.clone()).unwrap();
            let verifier = rustls::client::WebPkiServerVerifier::builder_with_provider(
                std::sync::Arc::new(roots),
                std::sync::Arc::new(rustls::crypto::ring::default_provider()),
            )
            .build()
            .unwrap();
            let now = UnixTime::since_unix_epoch(std::time::Duration::from_secs(1_800_000_000));
            for endpoint in hosts {
                let name = ServerName::try_from(endpoint.address.as_str()).unwrap();
                verifier.verify_server_cert(&certificate, &[], &name, &[], now).unwrap();
            }
            let obsolete = ServerName::try_from("169.254.23.99").unwrap();
            assert!(verifier.verify_server_cert(&certificate, &[], &obsolete, &[], now).is_err());
        }
    }
}
