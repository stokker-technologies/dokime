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

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn active_socket_guard() {
    use plugin_host::host::{
        adapters::snapshot,
        controls::{administrator, connected_adapters, guard_connection, require_administrator},
    };
    use std::net::{Ipv4Addr, TcpListener, TcpStream};
    if !administrator().unwrap() {
        assert!(require_administrator().is_err());
    }
    let timing = plugin_host::HostOptions::default().timing;
    let inventory = snapshot(std::time::Duration::from_millis(timing.network_probe_timeout_ms)).unwrap();
    let (id, address) = inventory
        .adapters
        .iter()
        .find_map(|adapter| {
            adapter.ipv4.iter().find_map(|address| {
                let ip = address.address.parse::<Ipv4Addr>().ok()?;
                (!ip.is_loopback() && !ip.is_link_local() && !ip.is_unspecified()).then(|| (adapter.id.clone(), ip))
            })
        })
        .expect("native machine has a non-loopback IPv4 interface");
    let listener = TcpListener::bind((address, 0)).unwrap();
    let _client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (_server, _) = listener.accept().unwrap();
    let pid = std::process::id();
    assert!(connected_adapters(pid, &timing).unwrap().contains(&id));
    assert!(guard_connection(pid, None, Some(&id), false, false, &timing).is_err());
    assert!(guard_connection(pid, None, Some(&id), true, false, &timing).is_err());
    assert!(guard_connection(pid, None, Some(&id), true, true, &timing).is_ok());
}
