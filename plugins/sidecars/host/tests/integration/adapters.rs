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
    ExecutionContext, Message,
    operations::{OperationDeclaration, Visibility},
    sidecar::{Observer, Publisher},
};
use plugin_host::{
    HostPlugin,
    host::{
        adapters::{Adapters, Snapshot, snapshot},
        network::NetworkObserver,
    },
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

#[tokio::test]
async fn notification_lifetime() {
    let context = ExecutionContext::default();
    // Releasing The Observer Cancels Native Registrations And Joins Its Callbacks.
    let observer = <NetworkObserver as Observer<Publisher<HostPlugin>>>::initialize(&context).await.unwrap();
    drop(observer);
    let observer = <NetworkObserver as Observer<Publisher<HostPlugin>>>::initialize(&context).await.unwrap();
    drop(observer);
}
