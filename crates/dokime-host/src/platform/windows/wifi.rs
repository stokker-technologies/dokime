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
    ffi::c_void,
    ops::RangeInclusive,
    ptr::{addr_of, null_mut},
    slice, thread,
    time::Duration,
};

// Third Party Imports
use windows::{
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            DOT11_AUTH_ALGO_80211_OPEN, DOT11_AUTH_ALGO_80211_SHARED_KEY, DOT11_AUTH_ALGO_OWE, DOT11_AUTH_ALGO_RSNA,
            DOT11_AUTH_ALGO_RSNA_PSK, DOT11_AUTH_ALGO_WPA, DOT11_AUTH_ALGO_WPA_NONE, DOT11_AUTH_ALGO_WPA_PSK, DOT11_AUTH_ALGO_WPA3,
            DOT11_AUTH_ALGO_WPA3_ENT, DOT11_AUTH_ALGO_WPA3_SAE, DOT11_AUTH_ALGORITHM, DOT11_CIPHER_ALGO_CCMP, DOT11_CIPHER_ALGO_CCMP_256,
            DOT11_CIPHER_ALGO_GCMP, DOT11_CIPHER_ALGO_GCMP_256, DOT11_CIPHER_ALGO_NONE, DOT11_CIPHER_ALGO_TKIP, DOT11_CIPHER_ALGO_WEP,
            DOT11_CIPHER_ALGO_WEP40, DOT11_CIPHER_ALGO_WEP104, DOT11_CIPHER_ALGORITHM, DOT11_PHY_TYPE, DOT11_SSID, WLAN_API_VERSION_2_0,
            WLAN_BSS_ENTRY, WLAN_BSS_LIST, WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST, WLAN_INTERFACE_STATE,
            WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory, WlanGetNetworkBssList, WlanOpenHandle, WlanQueryInterface, WlanScan,
            dot11_BSS_type_any, wlan_interface_state_ad_hoc_network_formed, wlan_interface_state_associating,
            wlan_interface_state_authenticating, wlan_interface_state_connected, wlan_interface_state_disconnected,
            wlan_interface_state_disconnecting, wlan_interface_state_discovering, wlan_interface_state_not_ready,
            wlan_intf_opcode_current_connection, wlan_intf_opcode_rssi,
        },
    },
    core::GUID,
};

// Local Imports
use crate::{Error, HostOptions, Result};

/* === Constants === */

const RSSI_RANGE_DBM: RangeInclusive<f64> = -120.0..=0.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct WifiLink {
    pub adapter: Option<String>,
    pub state: Option<String>,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub profile: Option<String>,
    pub radio: Option<String>,
    pub auth: Option<String>,
    pub cipher: Option<String>,
    pub channel: Option<u32>,
    pub band: Option<String>,
    pub signal_percent: Option<f64>,
    pub rssi_dbm: Option<f64>,
    pub rx_phy_mbps: Option<f64>,
    pub tx_phy_mbps: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct WifiChannelOccupancy {
    pub adapter: Option<String>,
    pub channel: u32,
    pub band: Option<String>,
    pub active_channel: Option<u32>,
    pub active_band: Option<String>,
    pub network_count: u32,
    pub max_signal_percent: Option<f64>,
}

struct WlanClient {
    handle: HANDLE,
}

#[derive(Clone, Debug)]
struct WifiInterface {
    guid: GUID,
    description: String,
    state: WLAN_INTERFACE_STATE,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl WifiLink {
    pub fn rssi_dbm_from_query(bytes: &[u8]) -> Option<f64> {
        let bytes = <[u8; 4]>::try_from(bytes.get(..4)?).ok()?;
        let rssi_dbm = f64::from(i32::from_ne_bytes(bytes));

        RSSI_RANGE_DBM.contains(&rssi_dbm).then_some(rssi_dbm)
    }
}

impl Drop for WlanClient {
    fn drop(&mut self) {
        unsafe {
            let _ = WlanCloseHandle(self.handle, None);
        }
    }
}

impl WlanClient {
    fn open() -> Result<Self> {
        let mut negotiated_version = 0;
        let mut handle = HANDLE::default();
        let code = unsafe { WlanOpenHandle(WLAN_API_VERSION_2_0, None, &mut negotiated_version, &mut handle) };
        if code != 0 {
            return Err(Error::Probe(format!("WlanOpenHandle failed with {code}")));
        }

        Ok(Self { handle })
    }

    fn interfaces(&self) -> Result<Vec<WifiInterface>> {
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = null_mut();
        let code = unsafe { WlanEnumInterfaces(self.handle, None, &mut list) };
        if code != 0 {
            return Err(Error::Probe(format!("WlanEnumInterfaces failed with {code}")));
        }
        if list.is_null() {
            return Ok(Vec::new());
        }

        let interfaces = unsafe {
            let count = (*list).dwNumberOfItems as usize;
            let first = addr_of!((*list).InterfaceInfo).cast::<WLAN_INTERFACE_INFO>();
            slice::from_raw_parts(first, count)
                .iter()
                .map(|interface| WifiInterface {
                    guid: interface.InterfaceGuid,
                    description: wide_string(&interface.strInterfaceDescription).unwrap_or_else(|| "WiFi".to_owned()),
                    state: interface.isState,
                })
                .collect::<Vec<_>>()
        };
        unsafe {
            WlanFreeMemory(list.cast::<c_void>());
        }

        Ok(interfaces)
    }

    fn current_connection(&self, interface: &WifiInterface) -> Result<Option<WLAN_CONNECTION_ATTRIBUTES>> {
        let mut data_size = 0;
        let mut data: *mut c_void = null_mut();
        let code = unsafe {
            WlanQueryInterface(self.handle, &interface.guid, wlan_intf_opcode_current_connection, None, &mut data_size, &mut data, None)
        };
        if code != 0 || data.is_null() {
            return Ok(None);
        }

        let attributes = unsafe { *(data.cast::<WLAN_CONNECTION_ATTRIBUTES>()) };
        unsafe {
            WlanFreeMemory(data);
        }

        Ok(Some(attributes))
    }

    fn rssi_dbm(&self, interface: &WifiInterface) -> Option<f64> {
        let mut data_size = 0;
        let mut data: *mut c_void = null_mut();
        let code =
            unsafe { WlanQueryInterface(self.handle, &interface.guid, wlan_intf_opcode_rssi, None, &mut data_size, &mut data, None) };
        if code != 0 || data.is_null() {
            return None;
        }

        let bytes = unsafe { slice::from_raw_parts(data.cast::<u8>(), data_size as usize) };
        let rssi_dbm = WifiLink::rssi_dbm_from_query(bytes);
        unsafe {
            WlanFreeMemory(data);
        }

        rssi_dbm
    }

    fn bss_entries(&self, interface: &WifiInterface, request_scan: bool) -> Result<Vec<WLAN_BSS_ENTRY>> {
        if request_scan {
            let scan = unsafe { WlanScan(self.handle, &interface.guid, None, None, None) };
            if scan != 0 {
                return Err(Error::Probe(format!("WlanScan failed with {scan}")));
            }
            thread::sleep(Duration::from_secs(4));
        }

        let mut list: *mut WLAN_BSS_LIST = null_mut();
        let code = unsafe { WlanGetNetworkBssList(self.handle, &interface.guid, None, dot11_BSS_type_any, false, None, &mut list) };
        if code != 0 {
            return Err(Error::Probe(format!("WlanGetNetworkBssList failed with {code}")));
        }
        if list.is_null() {
            return Ok(Vec::new());
        }

        let entries = unsafe {
            let count = (*list).dwNumberOfItems as usize;
            let first = addr_of!((*list).wlanBssEntries).cast::<WLAN_BSS_ENTRY>();
            slice::from_raw_parts(first, count).to_vec()
        };
        unsafe {
            WlanFreeMemory(list.cast::<c_void>());
        }

        Ok(entries)
    }
}

/* === Public Functions === */

pub fn active_link(options: &HostOptions) -> Result<Option<WifiLink>> {
    let client = WlanClient::open()?;
    let interfaces = client.interfaces()?;
    let Some(interface) = select_interface(&interfaces, options) else {
        return Ok(None);
    };
    let state = Some(interface_state(interface.state).to_owned());
    if interface.state != wlan_interface_state_connected {
        return Ok(Some(WifiLink {
            adapter: Some(interface.description),
            state,
            ssid: None,
            bssid: None,
            profile: None,
            radio: None,
            auth: None,
            cipher: None,
            channel: None,
            band: None,
            signal_percent: None,
            rssi_dbm: None,
            rx_phy_mbps: None,
            tx_phy_mbps: None,
        }));
    }

    let Some(attributes) = client.current_connection(&interface)? else {
        return Ok(None);
    };
    let association = attributes.wlanAssociationAttributes;
    let security = attributes.wlanSecurityAttributes;
    let bssid = mac_string(&association.dot11Bssid);
    // Read The Signal Before The Interface Gives Up Its Description
    let rssi_dbm = client.rssi_dbm(&interface);
    Ok(Some(WifiLink {
        adapter: Some(interface.description),
        state,
        ssid: ssid_string(&association.dot11Ssid),
        bssid: Some(bssid),
        profile: wide_string(&attributes.strProfileName),
        radio: Some(phy_type(association.dot11PhyType).to_owned()),
        auth: Some(auth_algorithm(security.dot11AuthAlgorithm).to_owned()),
        cipher: Some(cipher_algorithm(security.dot11CipherAlgorithm).to_owned()),
        channel: None,
        band: None,
        signal_percent: Some(f64::from(association.wlanSignalQuality)),
        rssi_dbm,
        rx_phy_mbps: Some(association.ulRxRate as f64 / 1_000.0),
        tx_phy_mbps: Some(association.ulTxRate as f64 / 1_000.0),
    }))
}

pub fn scan_channel_occupancy(options: &HostOptions) -> Result<Vec<WifiChannelOccupancy>> {
    let client = WlanClient::open()?;
    let interfaces = client.interfaces()?;
    let Some(interface) = select_interface(&interfaces, options) else {
        return Ok(Vec::new());
    };
    let entries = client.bss_entries(&interface, true)?;
    if entries.is_empty() {
        return Ok(Vec::new());
    }
    let active_channel = active_channel_from_scan(&client, &interface, &entries)?;
    let active_band = active_channel.and_then(wifi_band);

    let mut channels: BTreeMap<u32, WifiChannelOccupancy> = BTreeMap::new();
    for entry in entries {
        let Some(channel) = channel_from_frequency(entry.ulChCenterFrequency) else {
            continue;
        };
        let item = channels.entry(channel).or_insert(WifiChannelOccupancy {
            adapter: Some(interface.description.clone()),
            channel,
            band: wifi_band(channel),
            active_channel,
            active_band: active_band.clone(),
            network_count: 0,
            max_signal_percent: None,
        });
        item.network_count += 1;
        let signal = f64::from(entry.uLinkQuality);
        item.max_signal_percent = Some(item.max_signal_percent.map_or(signal, |max| max.max(signal)));
    }

    Ok(channels.into_values().collect())
}

/* === Private Functions === */

fn select_interface(interfaces: &[WifiInterface], options: &HostOptions) -> Option<WifiInterface> {
    if let Some(adapter) = &options.wifi_adapter_name {
        let adapter = adapter.to_lowercase();
        return interfaces.iter().find(|interface| interface.description.to_lowercase().contains(&adapter)).cloned();
    }

    interfaces.iter().find(|interface| interface.state == wlan_interface_state_connected).or_else(|| interfaces.first()).cloned()
}

fn wide_string(value: &[u16]) -> Option<String> {
    let len = value.iter().position(|item| *item == 0).unwrap_or(value.len());
    if len == 0 {
        return None;
    }

    Some(String::from_utf16_lossy(&value[..len]))
}

fn ssid_string(value: &DOT11_SSID) -> Option<String> {
    let len = usize::try_from(value.uSSIDLength).ok()?.min(value.ucSSID.len());
    if len == 0 {
        return None;
    }

    Some(String::from_utf8_lossy(&value.ucSSID[..len]).to_string())
}

fn mac_string(value: &[u8; 6]) -> String {
    format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", value[0], value[1], value[2], value[3], value[4], value[5])
}

fn channel_from_frequency(value: u32) -> Option<u32> {
    let mhz = if value > 10_000 { value / 1_000 } else { value };
    match mhz {
        2_412..=2_472 => Some((mhz - 2_407) / 5),
        2_484 => Some(14),
        5_005..=5_895 => Some((mhz - 5_000) / 5),
        5_955..=7_115 => Some(1_000 + (mhz - 5_950) / 5),
        _ => None,
    }
}

fn wifi_band(channel: u32) -> Option<String> {
    match channel {
        1..=14 => Some("2.4GHz".to_owned()),
        32..=177 => Some("5GHz".to_owned()),
        1_000..=2_000 => Some("6GHz".to_owned()),
        _ => None,
    }
}

fn active_channel_from_scan(client: &WlanClient, interface: &WifiInterface, entries: &[WLAN_BSS_ENTRY]) -> Result<Option<u32>> {
    if interface.state != wlan_interface_state_connected {
        return Ok(None);
    }
    let Some(attributes) = client.current_connection(interface)? else {
        return Ok(None);
    };
    let association = attributes.wlanAssociationAttributes;

    Ok(entries
        .iter()
        .find(|entry| entry.dot11Bssid == association.dot11Bssid)
        .and_then(|entry| channel_from_frequency(entry.ulChCenterFrequency)))
}

fn interface_state(state: WLAN_INTERFACE_STATE) -> &'static str {
    match state {
        value if value == wlan_interface_state_not_ready => "not_ready",
        value if value == wlan_interface_state_connected => "connected",
        value if value == wlan_interface_state_ad_hoc_network_formed => "ad_hoc_network_formed",
        value if value == wlan_interface_state_disconnecting => "disconnecting",
        value if value == wlan_interface_state_disconnected => "disconnected",
        value if value == wlan_interface_state_associating => "associating",
        value if value == wlan_interface_state_discovering => "discovering",
        value if value == wlan_interface_state_authenticating => "authenticating",
        _ => "unknown",
    }
}

fn phy_type(value: DOT11_PHY_TYPE) -> &'static str {
    match value.0 {
        4 => "802.11g",
        5 => "802.11a",
        6 => "802.11n",
        7 => "802.11ac",
        8 => "802.11ad",
        9 => "802.11ax",
        _ => "unknown",
    }
}

fn auth_algorithm(value: DOT11_AUTH_ALGORITHM) -> &'static str {
    match value {
        value if value == DOT11_AUTH_ALGO_80211_OPEN => "open",
        value if value == DOT11_AUTH_ALGO_80211_SHARED_KEY => "shared_key",
        value if value == DOT11_AUTH_ALGO_WPA => "wpa",
        value if value == DOT11_AUTH_ALGO_WPA_PSK => "wpa_psk",
        value if value == DOT11_AUTH_ALGO_WPA_NONE => "wpa_none",
        value if value == DOT11_AUTH_ALGO_RSNA => "rsna",
        value if value == DOT11_AUTH_ALGO_RSNA_PSK => "rsna_psk",
        value if value == DOT11_AUTH_ALGO_WPA3 => "wpa3",
        value if value == DOT11_AUTH_ALGO_WPA3_SAE => "wpa3_sae",
        value if value == DOT11_AUTH_ALGO_OWE => "owe",
        value if value == DOT11_AUTH_ALGO_WPA3_ENT => "wpa3_enterprise",
        _ => "unknown",
    }
}

fn cipher_algorithm(value: DOT11_CIPHER_ALGORITHM) -> &'static str {
    match value {
        value if value == DOT11_CIPHER_ALGO_NONE => "none",
        value if value == DOT11_CIPHER_ALGO_WEP40 => "wep40",
        value if value == DOT11_CIPHER_ALGO_TKIP => "tkip",
        value if value == DOT11_CIPHER_ALGO_CCMP => "ccmp",
        value if value == DOT11_CIPHER_ALGO_WEP104 => "wep104",
        value if value == DOT11_CIPHER_ALGO_WEP => "wep",
        value if value == DOT11_CIPHER_ALGO_GCMP => "gcmp",
        value if value == DOT11_CIPHER_ALGO_GCMP_256 => "gcmp_256",
        value if value == DOT11_CIPHER_ALGO_CCMP_256 => "ccmp_256",
        _ => "unknown",
    }
}

/* === Tests === */
