/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", unix))]
mod unix;

#[cfg(all(feature = "runtime", target_os = "windows"))]
mod windows;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, time::Instant};

// Third Party Imports
use sysinfo::Networks;

// Local Imports
use crate::{
    HostStreamSample, HostTelemetryStream, Result,
    collectors::{integer, maybe_insert},
};
use dokime_domain::model::values::Value;

/* === Constants === */

const BYTES_PER_MEGABYTE: f64 = 1_000_000.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct NetworkCollector {
    sample_index: u64,
    networks: Networks,
    last_refresh: Instant,
    primary_interface: Option<String>,
    refresh_primary: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl NetworkCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            sample_index: 0,
            networks: Networks::new_with_refreshed_list()?,
            last_refresh: Instant::now(),
            primary_interface: None,
            refresh_primary: true,
        })
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
        if self.refresh_primary {
            self.primary_interface =
                crate::host::adapters::snapshot()?.adapters.into_iter().find(|adapter| adapter.primary).map(|adapter| adapter.id);
            self.refresh_primary = false;
        }

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
        ]);
        maybe_insert(&mut values, "primary_interface", self.primary_interface.clone().map(Value::from));
        maybe_insert(&mut values, "internet", Self::internet().map(Value::from));

        Ok(HostStreamSample::new(HostTelemetryStream::Network, values))
    }

    pub fn invalidate_primary(&mut self) {
        self.refresh_primary = true;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
