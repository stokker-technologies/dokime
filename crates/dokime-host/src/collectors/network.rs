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
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

// Third Party Imports
use sysinfo::Networks;

// Local Imports
use crate::{
    HostStreamSample, HostTelemetryStream, Result,
    collectors::{integer, maybe_insert},
};
use dokime_domain::model::values::Value;

/* === Constants === */

const NETWORK_LINK_INTERVAL_MS: u64 = 5_000;

const BYTES_PER_MEGABYTE: f64 = 1_000_000.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct NetworkCollector {
    sample_index: u64,
    networks: Networks,
    last_refresh: Instant,
    route_refresh_interval: Duration,
    last_route_refresh: Option<Instant>,
    route: RouteContext,
}

#[derive(Clone, Default)]
struct RouteContext {
    default_interface: Option<String>,
    default_gateway: Option<String>,
    local_addresses: Vec<String>,
}

#[derive(Clone)]
struct PrimaryNetwork {
    name: String,
    mtu: u64,
    mac: Option<String>,
    operational_state: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            sample_index: 0,
            networks: Networks::new_with_refreshed_list(),
            last_refresh: Instant::now(),
            route_refresh_interval: Duration::from_millis(NETWORK_LINK_INTERVAL_MS),
            last_route_refresh: None,
            route: RouteContext::default(),
        }
    }

    pub fn collect(&mut self) -> Result<HostStreamSample> {
        self.sample_index += 1;
        let now = Instant::now();
        let elapsed_s = now.duration_since(self.last_refresh).as_secs_f64().max(0.001);
        self.networks.refresh(true);
        self.last_refresh = now;

        let received = self.networks.values().map(|network| network.received()).sum::<u64>();
        let transmitted = self.networks.values().map(|network| network.transmitted()).sum::<u64>();
        let total_received = self.networks.values().map(|network| network.total_received()).sum::<u64>();
        let total_transmitted = self.networks.values().map(|network| network.total_transmitted()).sum::<u64>();
        let rx_errors = self.networks.values().map(|network| network.errors_on_received()).sum::<u64>();
        let tx_errors = self.networks.values().map(|network| network.errors_on_transmitted()).sum::<u64>();
        let primary = primary_network(&self.networks);
        let route = self.route_context(now);

        let mut values = BTreeMap::from([
            ("sample_index".to_owned(), Value::Integer(self.sample_index as i64)),
            ("probe_status".to_owned(), Value::from("ok")),
            ("network_interface_count".to_owned(), Value::Integer(self.networks.len() as i64)),
            ("network_recv_mb_per_second".to_owned(), Value::from((received as f64 / elapsed_s) / BYTES_PER_MEGABYTE)),
            ("network_sent_mb_per_second".to_owned(), Value::from((transmitted as f64 / elapsed_s) / BYTES_PER_MEGABYTE)),
            ("network_total_recv_bytes".to_owned(), integer(total_received)),
            ("network_total_sent_bytes".to_owned(), integer(total_transmitted)),
            ("network_rx_errors".to_owned(), integer(rx_errors)),
            ("network_tx_errors".to_owned(), integer(tx_errors)),
            ("local_address_count".to_owned(), Value::Integer(route.local_addresses.len() as i64)),
            ("local_addresses".to_owned(), Value::Array(route.local_addresses.into_iter().map(Value::from).collect())),
        ]);
        maybe_insert(&mut values, "default_interface", route.default_interface.map(Value::from));
        maybe_insert(&mut values, "default_gateway", route.default_gateway.map(Value::from));
        maybe_insert(&mut values, "radio_link", primary.as_ref().map(|network| Value::from(network.name.clone())));
        maybe_insert(&mut values, "network_mtu", primary.as_ref().map(|network| Value::Integer(network.mtu as i64)));
        maybe_insert(&mut values, "network_mac", primary.as_ref().and_then(|network| network.mac.clone()).map(Value::from));
        maybe_insert(
            &mut values,
            "network_operational_state",
            primary.as_ref().map(|network| Value::from(network.operational_state.clone())),
        );

        Ok(HostStreamSample::new(HostTelemetryStream::Network, values))
    }

    fn route_context(&mut self, now: Instant) -> RouteContext {
        let should_refresh =
            self.last_route_refresh.map(|last_refresh| now.duration_since(last_refresh) >= self.route_refresh_interval).unwrap_or(true);

        if should_refresh {
            self.route = route_context();
            self.last_route_refresh = Some(now);
        }

        self.route.clone()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn primary_network(networks: &Networks) -> Option<PrimaryNetwork> {
    networks
        .iter()
        .filter(|(_, network)| !network.mac_address().is_unspecified())
        .max_by_key(|(_, network)| network.total_received() + network.total_transmitted())
        .or_else(|| networks.iter().next())
        .map(|(name, network)| PrimaryNetwork {
            name: name.clone(),
            mtu: network.mtu(),
            mac: (!network.mac_address().is_unspecified()).then(|| network.mac_address().to_string()),
            operational_state: network.operational_state().to_string(),
        })
}

fn route_context() -> RouteContext {
    let local_addresses = getifs::best_local_addrs()
        .or_else(|_| getifs::local_addrs())
        .map(|addresses| addresses.iter().map(|address| address.addr().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let default_interface = getifs::best_local_addrs()
        .ok()
        .and_then(|addresses| addresses.first().copied())
        .and_then(|address| address.name().ok().map(|name| name.to_string()));
    let default_gateway =
        getifs::gateway_addrs().ok().and_then(|addresses| addresses.first().copied()).map(|address| address.addr().to_string());

    RouteContext {
        default_interface,
        default_gateway,
        local_addresses,
    }
}

/* === Tests === */
