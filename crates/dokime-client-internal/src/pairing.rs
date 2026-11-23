/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Endpoints a Pairing File Advertises

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::net::{IpAddr, SocketAddr};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Clients verify the pinned certificate against the host they dial, so only certificate names can pass TLS.
/// A wildcard bind advertises each certificate name of its address family, network names before loopback ones.
pub fn pairing_endpoints(bound: &str, certificate_names: &[String]) -> Result<Vec<String>, String> {
    let address: SocketAddr = bound
        .strip_prefix("https://")
        .unwrap_or(bound)
        .parse()
        .map_err(|_| "Enable the Remote Access Listener, Then Export Again".to_owned())?;
    let wildcard = address.ip().is_unspecified();
    let (mut network, mut loopback) = (Vec::new(), Vec::new());
    for name in certificate_names {
        let (endpoint, local) = match name.parse::<IpAddr>() {
            Ok(ip) if !ip.is_unspecified() && (ip == address.ip() || wildcard && ip.is_ipv4() == address.is_ipv4()) => {
                (format!("https://{}", SocketAddr::new(ip, address.port())), ip.is_loopback())
            }
            Err(_) if wildcard && !name.contains('*') => {
                (format!("https://{name}:{}", address.port()), name.eq_ignore_ascii_case("localhost"))
            }
            _ => continue,
        };
        let group = if local { &mut loopback } else { &mut network };
        if !group.contains(&endpoint) {
            group.push(endpoint);
        }
    }
    network.extend(loopback);
    if network.is_empty() {
        let target = if wildcard { "This Host's Network Address or Name".to_owned() } else { address.ip().to_string() };
        return Err(format!("Add {target} to the Remote Access Certificate Names, Then Export Again"));
    }
    Ok(network)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::pairing_endpoints;

    #[test]
    fn wildcard_binds_advertise_certificate_names_network_first() {
        let names = ["localhost", "127.0.0.1", "::1", "lab-pc", "192.168.1.20", "*.lab"].map(String::from);
        assert_eq!(
            pairing_endpoints("https://0.0.0.0:8765", &names).unwrap(),
            ["https://lab-pc:8765", "https://192.168.1.20:8765", "https://localhost:8765", "https://127.0.0.1:8765"]
        );
        assert_eq!(pairing_endpoints("https://192.168.1.20:8765", &names).unwrap(), ["https://192.168.1.20:8765"]);
        assert!(pairing_endpoints("https://10.0.0.5:8765", &names).unwrap_err().contains("10.0.0.5"));
        assert!(pairing_endpoints("", &names).is_err());
    }
}
