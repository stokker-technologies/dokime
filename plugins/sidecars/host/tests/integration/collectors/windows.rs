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
use plugin_host::collectors::wifi::WifiLink;

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
fn rssi_long_conversion() {
    assert_eq!(WifiLink::rssi_dbm_from_query(&(-46_i32).to_ne_bytes()), Some(-46.0));
    assert_eq!(WifiLink::rssi_dbm_from_query(&[0xd2, 0xff]), None);
    assert_eq!(WifiLink::rssi_dbm_from_query(&12_i32.to_ne_bytes()), None);
    assert_eq!(WifiLink::rssi_dbm_from_query(&(-256_i32).to_ne_bytes()), None);
}
