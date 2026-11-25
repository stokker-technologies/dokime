/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bluetooth Radio and Adapter Inventory, Bluetooth Radio Counters and BLE Advertisement Signal Strength

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::mem;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Instant;

// Third Party Imports
use windows::Devices::Bluetooth::Advertisement::{
    BluetoothLEAdvertisementReceivedEventArgs, BluetoothLEAdvertisementWatcher, BluetoothLEAdvertisementWatcherStatus,
    BluetoothLEAdvertisementWatcherStoppedEventArgs, BluetoothLEScanningMode,
};
use windows::Devices::Bluetooth::BluetoothError;
use windows::Foundation::TypedEventHandler;
use windows::Win32::System::Performance::{
    PDH_CSTATUS_INVALID_DATA, PDH_CSTATUS_NEW_DATA, PDH_CSTATUS_NO_INSTANCE, PDH_CSTATUS_NO_OBJECT, PDH_CSTATUS_VALID_DATA,
    PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_INVALID_DATA, PDH_MORE_DATA, PDH_NO_DATA,
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
};
use windows::core::{HSTRING, Ref};

// Local Imports
use super::apartment::Lease;
use crate::collectors::bluetooth::{Heard, LinkOptions, Measure, Radio, Sample, radios};
use crate::controls::Timing;
use crate::domain::controls::{Radio as RadioKind, RadioPower};
use crate::domain::readings::Bluetooth;
use crate::domain::telemetry::BluetoothRssi;
use crate::platform::windows::controls::read_radio;

/* === Constants === */

/// The Bluetooth Radio Counter Set's English Paths: Every Radio's ACL and SCO Bytes per Second and Its ACL Connections.
const COUNTERS: [(Measure, &str); 8] = [
    (Measure::Received, r"\Bluetooth Radio(*)\Classic ACL bytes read/sec"),
    (Measure::Received, r"\Bluetooth Radio(*)\LE ACL bytes read/sec"),
    (Measure::Received, r"\Bluetooth Radio(*)\SCO bytes read/sec"),
    (Measure::Transmitted, r"\Bluetooth Radio(*)\Classic ACL bytes written/sec"),
    (Measure::Transmitted, r"\Bluetooth Radio(*)\LE ACL bytes written/sec"),
    (Measure::Transmitted, r"\Bluetooth Radio(*)\SCO bytes written/sec"),
    (Measure::Connections, r"\Bluetooth Radio(*)\Classic ACL Connections"),
    (Measure::Connections, r"\Bluetooth Radio(*)\LE ACL Connections"),
];

const PDH_SUCCESS: u32 = 0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// The Link Collection's Native State: the Counter Query and the Advertisement Watcher, Each Opened at Its First Read.
#[derive(Default)]
pub struct Link {
    counters: Option<Counters>,
    watcher: Option<Watcher>,
}

/// One Open Counter Query With One Wildcard Counter per Path, Closed on Drop.
struct Counters {
    query: PDH_HQUERY,
    counters: Vec<(Measure, PDH_HCOUNTER)>,
}

/// One Active-Scanning Advertisement Watcher and What Its Handlers Recorded, Stopped on Drop.
struct Watcher {
    watcher: BluetoothLEAdvertisementWatcher,
    received: i64,
    stopped: i64,
    watched: Arc<Mutex<Watched>>,
    _lease: Lease,
}

#[derive(Default)]
struct Watched {
    heard: Heard,
    stopped: Option<BluetoothError>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Link {
    /// Every Radio Instance's Traffic and Connections; a Rate Needs the Query's Second Collection.
    pub fn radios(&mut self, _options: &LinkOptions) -> Result<Vec<Radio>, String> {
        let counters = match &mut self.counters {
            Some(counters) => counters,
            None => self.counters.insert(Counters::open()?),
        };
        counters.read()
    }

    /// The Selected Devices Heard Since the Previous Read; an Aborted Watcher Restarts and Says Why It Stopped.
    pub fn heard(&mut self, devices: &[String]) -> Result<Vec<BluetoothRssi>, String> {
        let watcher = match &self.watcher {
            Some(watcher) => watcher,
            None => self.watcher.insert(Watcher::start()?),
        };
        watcher.drain(devices)
    }
}

impl Counters {
    fn open() -> Result<Self, String> {
        let mut query = PDH_HQUERY::default();
        // SAFETY: PdhOpenQueryW Writes One Query Handle, Which Counters::drop Closes.
        pdh("Open Bluetooth Counters", unsafe { PdhOpenQueryW(None, 0, &mut query) })?;
        let mut counters = Self {
            query,
            counters: Vec::with_capacity(COUNTERS.len()),
        };
        for (measure, path) in COUNTERS {
            let mut counter = PDH_HCOUNTER::default();
            // SAFETY: The Query Is Open and the Path Outlives the Call; the Counter Belongs to the Query.
            pdh("Open Bluetooth Counters", unsafe { PdhAddEnglishCounterW(query, &HSTRING::from(path), 0, &mut counter) })?;
            counters.counters.push((measure, counter));
        }
        Ok(counters)
    }

    fn read(&mut self) -> Result<Vec<Radio>, String> {
        // SAFETY: The Query Is Open Until Drop.
        match unsafe { PdhCollectQueryData(self.query) } {
            PDH_NO_DATA => return Ok(Vec::new()),
            status => pdh("Read Bluetooth Counters", status)?,
        }
        let mut samples = Vec::new();
        for (measure, counter) in &self.counters {
            values(*measure, *counter, &mut samples)?;
        }
        Ok(radios(samples))
    }
}

impl Watcher {
    fn start() -> Result<Self, String> {
        let lease = Lease::acquire().map_err(|error| error.to_string())?;
        let watcher = BluetoothLEAdvertisementWatcher::new().map_err(watch)?;
        watcher.SetScanningMode(BluetoothLEScanningMode::Active).map_err(watch)?;
        let watched = Arc::new(Mutex::new(Watched::default()));
        let received = watcher
            .Received(&TypedEventHandler::new({
                let watched = Arc::clone(&watched);
                move |_, args: Ref<BluetoothLEAdvertisementReceivedEventArgs>| {
                    let args = args.ok()?;
                    let name = args.Advertisement()?.LocalName()?.to_string_lossy();
                    let rssi_dbm = f64::from(args.RawSignalStrengthInDBm()?);
                    watched.lock().unwrap_or_else(PoisonError::into_inner).heard.record(&name, rssi_dbm, Instant::now());
                    Ok(())
                }
            }))
            .map_err(watch)?;
        let stopped = watcher
            .Stopped(&TypedEventHandler::new({
                let watched = Arc::clone(&watched);
                move |_, args: Ref<BluetoothLEAdvertisementWatcherStoppedEventArgs>| {
                    watched.lock().unwrap_or_else(PoisonError::into_inner).stopped = Some(args.ok()?.Error()?);
                    Ok(())
                }
            }))
            .map_err(watch)?;
        let watcher = Self {
            watcher,
            received,
            stopped,
            watched,
            _lease: lease,
        };
        watcher.watcher.Start().map_err(watch)?;
        Ok(watcher)
    }

    fn drain(&self, devices: &[String]) -> Result<Vec<BluetoothRssi>, String> {
        let aborted = self.watcher.Status().map_err(watch)? == BluetoothLEAdvertisementWatcherStatus::Aborted;
        let (heard, stopped) = {
            let mut watched = self.watched.lock().unwrap_or_else(PoisonError::into_inner);
            (watched.heard.drain(devices), watched.stopped.take())
        };
        if !aborted {
            return Ok(heard);
        }
        let restart = self.watcher.Start().err().map(|error| format!(" and Did Not Restart: {error}")).unwrap_or_default();
        let reason = match stopped {
            Some(BluetoothError::RadioNotAvailable) => "No Bluetooth Radio Is Available",
            Some(BluetoothError::DisabledByUser) => "Bluetooth Is Turned Off",
            Some(BluetoothError::DisabledByPolicy) => "Bluetooth Is Disabled by Policy",
            Some(BluetoothError::ResourceInUse) => "the Radio Is Busy",
            _ => "the Watcher Stopped",
        };
        Err(format!("Watch Bluetooth Advertisements: {reason}{restart}; Connect or Turn On a Bluetooth Radio"))
    }
}

impl Drop for Counters {
    fn drop(&mut self) {
        // SAFETY: The Query Came From PdhOpenQueryW and Is Closed Once; Closing It Also Frees Its Counters.
        let _ = unsafe { PdhCloseQuery(self.query) };
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        let _ = self.watcher.RemoveReceived(self.received);
        let _ = self.watcher.RemoveStopped(self.stopped);
        let _ = self.watcher.Stop();
    }
}

// SAFETY: PDH Query and Counter Handles Are Process-Wide, Not Bound to Their Creating Thread; the Link Mutex Serializes Every Use.
unsafe impl Send for Counters {}

/* === Public Functions === */

/// The Inventory Columns: Present Bluetooth Devices Matching the Configured Adapter and the Bluetooth Radios' Power.
pub fn snapshot(adapter: Option<&str>, timing: &Timing) -> Result<Bluetooth, String> {
    let names = super::devices::bluetooth_presence().map_err(|error| error.to_string())?;
    let filter = adapter.map(str::to_lowercase);
    let device_names =
        names.into_iter().filter(|name| filter.as_ref().is_none_or(|filter| name.to_lowercase().contains(filter))).collect::<Vec<_>>();
    // An Unreadable Radio Leaves the Radio Columns Empty.
    let radios = read_radio(RadioKind::Bluetooth, timing).ok().map(|state| state.radios.iter().map(|radio| radio.on).collect::<Vec<_>>());
    let adapter_count = radios.as_ref().map(Vec::len);
    let radio_state = match radios.as_deref() {
        Some(radios) if radios.iter().any(|on| *on) => Some(RadioPower::On),
        Some(radios) if !radios.is_empty() => Some(RadioPower::Off),
        _ => None,
    };
    Ok(Bluetooth {
        bluetooth_adapter: device_names.first().cloned(),
        bluetooth_adapter_count: adapter_count.map(count),
        bluetooth_present_device_count: Some(count(device_names.len())),
        bluetooth_radio_state: Some(radio_state.map_or("unavailable", RadioPower::as_str).into()),
        bluetooth_device_names: device_names,
        ..Bluetooth::default()
    })
}

/* === Private Functions === */

fn count(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Append Every Instance's Valid Value of One Wildcard Counter; No Instance and a Rate Before Its Second Collection Add None.
fn values(measure: Measure, counter: PDH_HCOUNTER, samples: &mut Vec<Sample>) -> Result<(), String> {
    let mut items = Vec::<PDH_FMT_COUNTERVALUE_ITEM_W>::new();
    let mut size = 0;
    let mut filled = 0;
    loop {
        let buffer = (!items.is_empty()).then_some(items.as_mut_ptr());
        // SAFETY: The Buffer Holds at Least `size` Bytes, Aligned for Its Items, Whose Names Point Into It.
        match unsafe { PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut filled, buffer) } {
            PDH_SUCCESS => break,
            PDH_MORE_DATA => {
                items =
                    vec![PDH_FMT_COUNTERVALUE_ITEM_W::default(); (size as usize).div_ceil(mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>())]
            }
            PDH_NO_DATA | PDH_INVALID_DATA | PDH_CSTATUS_INVALID_DATA | PDH_CSTATUS_NO_INSTANCE => return Ok(()),
            status => return pdh("Read Bluetooth Counters", status),
        }
    }
    for item in items.iter().take(filled as usize) {
        if !matches!(item.FmtValue.CStatus, PDH_CSTATUS_VALID_DATA | PDH_CSTATUS_NEW_DATA) {
            continue;
        }
        // SAFETY: A Valid Item Formatted as Double Holds a Double and a Name Terminated Inside the Buffer.
        let (instance, value) = unsafe { (item.szName.to_string().unwrap_or_default(), item.FmtValue.Anonymous.doubleValue) };
        samples.push(Sample { instance, measure, value });
    }
    Ok(())
}

fn pdh(action: &str, status: u32) -> Result<(), String> {
    match status {
        PDH_SUCCESS => Ok(()),
        PDH_CSTATUS_NO_OBJECT => Err(format!("{action}: Windows Has No Bluetooth Radio Counter Set; Update Windows")),
        status => Err(format!("{action}: PDH Status {status:#010x}; Restart the Plugin")),
    }
}

fn watch(error: windows::core::Error) -> String {
    format!("Watch Bluetooth Advertisements: {error}; Restart the Plugin")
}

/* === Tests === */
