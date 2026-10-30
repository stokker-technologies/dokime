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

#[test]
fn active_socket_guard() {
    use dokime_host::host::{
        adapters::snapshot,
        controls::{administrator, connected_adapters, guard_connection, require_administrator},
    };
    use std::net::{Ipv4Addr, TcpListener, TcpStream};
    if !administrator().unwrap() {
        assert!(require_administrator().is_err());
    }
    let inventory = snapshot().unwrap();
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
    assert!(connected_adapters(pid).unwrap().contains(&id));
    assert!(guard_connection(pid, None, Some(&id), false, false).is_err());
    assert!(guard_connection(pid, None, Some(&id), true, false).is_err());
    assert!(guard_connection(pid, None, Some(&id), true, true).is_ok());
}
