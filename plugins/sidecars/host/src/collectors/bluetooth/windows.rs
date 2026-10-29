/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

use super::*;
use crate::collectors::maybe_insert;

/* === Private Functions === */

impl BluetoothCollector {
    pub(super) fn bluetooth_values(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
        crate::platform::windows::bluetooth::snapshot(options, timeout).map(snapshot_values).map_err(|error| error.to_string())
    }
}

fn snapshot_values(snapshot: crate::platform::windows::bluetooth::BluetoothSnapshot) -> BTreeMap<String, Value> {
    let mut values = BTreeMap::new();
    maybe_insert(&mut values, "bluetooth_adapter", snapshot.adapter_name.map(Value::from));
    values.insert("bluetooth_adapter_count".to_owned(), Value::Integer(snapshot.adapter_count as i64));
    maybe_insert(
        &mut values,
        "bluetooth_connected_device_count",
        snapshot.connected_device_count.map(|count| Value::Integer(count as i64)),
    );
    values.insert("bluetooth_present_device_count".to_owned(), Value::Integer(snapshot.present_device_count as i64));
    maybe_insert(&mut values, "bluetooth_radio_state", snapshot.radio_state.map(Value::from));
    values.insert("bluetooth_device_names".to_owned(), Value::Array(snapshot.device_names.into_iter().map(Value::from).collect()));

    values
}
