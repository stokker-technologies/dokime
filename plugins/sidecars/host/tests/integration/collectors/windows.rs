/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime_host::collectors::wifi::WifiLink;

#[test]
fn rssi_long_conversion() {
    assert_eq!(WifiLink::rssi_dbm_from_query(&(-46_i32).to_ne_bytes()), Some(-46.0));
    assert_eq!(WifiLink::rssi_dbm_from_query(&[0xd2, 0xff]), None);
    assert_eq!(WifiLink::rssi_dbm_from_query(&12_i32.to_ne_bytes()), None);
    assert_eq!(WifiLink::rssi_dbm_from_query(&(-256_i32).to_ne_bytes()), None);
}
