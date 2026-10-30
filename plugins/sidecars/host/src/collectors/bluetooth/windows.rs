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
use super::*;
use crate::collectors::maybe_insert;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl BluetoothCollector {
    pub(super) fn bluetooth_values(options: &HostOptions, timeout: Duration) -> Result<BTreeMap<String, Value>, String> {
        crate::platform::windows::bluetooth::snapshot(options, timeout).map(snapshot_values).map_err(|error| error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn snapshot_values(snapshot: crate::platform::windows::bluetooth::BluetoothSnapshot) -> BTreeMap<String, Value> {
    let mut values = BTreeMap::new();
    maybe_insert(&mut values, "bluetooth_adapter", snapshot.adapter_name.map(Value::from));
    maybe_insert(&mut values, "bluetooth_adapter_count", snapshot.adapter_count.map(|count| Value::Integer(count as i64)));
    maybe_insert(
        &mut values,
        "bluetooth_connected_device_count",
        snapshot.connected_device_count.map(|count| Value::Integer(count as i64)),
    );
    values.insert("bluetooth_present_device_count".to_owned(), Value::Integer(snapshot.present_device_count as i64));
    values.insert("bluetooth_radio_state".into(), Value::from(snapshot.radio_state.map_or("unavailable", |power| power.as_str())));
    values.insert("bluetooth_device_names".to_owned(), Value::Array(snapshot.device_names.into_iter().map(Value::from).collect()));

    values
}

/* === Tests === */
