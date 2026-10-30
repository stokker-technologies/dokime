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
use dokime::harmos::{
    Message,
    operations::{OperationDeclaration, Visibility},
};
use dokime_host::host::{
    adapters::{AdapterKind, AdapterStatus, Adapters, Snapshot, snapshot},
    network::NetworkObserver,
};

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
fn adapter_inventory() {
    let snapshot = snapshot().unwrap();
    assert!(!snapshot.adapters.is_empty());
    assert_eq!(Adapters::VISIBILITY, Visibility::Public);
    let decoded = Snapshot::decode(&snapshot.encode()).unwrap();
    assert!(snapshot == decoded);
    let captured = serde_json::to_string(&snapshot).unwrap();
    let decoded = serde_json::from_str::<Snapshot>(&captured).unwrap();
    assert!(snapshot == decoded);

    let mut ids = std::collections::BTreeSet::new();
    for adapter in snapshot.adapters {
        assert!(ids.insert(adapter.id));
        assert!(!adapter.name.is_empty());
        for address in adapter.ipv4 {
            assert!(address.address.parse::<std::net::Ipv4Addr>().is_ok());
            assert!(address.prefix <= 32);
        }
        for address in adapter.ipv6 {
            assert!(address.address.parse::<std::net::Ipv6Addr>().is_ok());
            assert!(address.prefix <= 128);
        }
        for address in adapter.gateways.into_iter().chain(adapter.dns_servers) {
            assert!(address.parse::<std::net::IpAddr>().is_ok());
        }
    }
}

#[test]
fn inventory_rejects_unknown_kind_and_status() {
    assert!(serde_json::from_str::<AdapterKind>("\"satellite\"").is_err());
    assert!(serde_json::from_str::<AdapterStatus>("\"sleeping\"").is_err());
}

#[test]
fn notification_lifetime() {
    // Releasing The Observer Cancels Native Registrations And Joins Its Callbacks.
    let observer = NetworkObserver::new().unwrap();
    drop(observer);
    let observer = NetworkObserver::new().unwrap();
    drop(observer);
}
