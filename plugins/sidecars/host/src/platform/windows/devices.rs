/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Device Inventory Through SetupAPI and Windows Devices

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::fmt::Display;
use std::mem;

// Third Party Imports
use windows::Win32::Devices::DeviceAndDriverInstallation::*;
use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;
use windows::core::{GUID, HRESULT};

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Devices(HDEVINFO);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Devices {
    fn drop(&mut self) {
        let _ = unsafe { SetupDiDestroyDeviceInfoList(self.0) };
    }
}

/* === Public Functions === */

pub fn bluetooth_presence() -> Result<Vec<String>> {
    let devices = open(&GUID_DEVCLASS_BLUETOOTH)?;
    let mut names = Vec::new();
    let mut index = 0;
    while let Some(device) = entry(&devices, index)? {
        index += 1;
        let name = property(&devices, &device, SPDRP_FRIENDLYNAME).or_else(|_| property(&devices, &device, SPDRP_DEVICEDESC))?;
        names.push(name);
    }
    Ok(names)
}

/* === Private Functions === */

fn probe(error: impl Display) -> Error {
    Error::Invalid(format!("Read Device Inventory: {error}"))
}
fn open(class: &GUID) -> Result<Devices> {
    unsafe { SetupDiGetClassDevsW(Some(class), None, None, DIGCF_PRESENT) }.map(Devices).map_err(probe)
}
fn entry(devices: &Devices, index: u32) -> Result<Option<SP_DEVINFO_DATA>> {
    let mut device = SP_DEVINFO_DATA {
        cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };
    match unsafe { SetupDiEnumDeviceInfo(devices.0, index, &mut device) } {
        Ok(()) => Ok(Some(device)),
        Err(error) if error.code() == HRESULT::from_win32(ERROR_NO_MORE_ITEMS.0) => Ok(None),
        Err(error) => Err(probe(error)),
    }
}
fn property(devices: &Devices, device: &SP_DEVINFO_DATA, property: SETUP_DI_REGISTRY_PROPERTY) -> Result<String> {
    let mut size = 0;
    let _ = unsafe { SetupDiGetDeviceRegistryPropertyW(devices.0, device, property, None, None, Some(&mut size)) };
    if size == 0 {
        return Err(Error::Invalid("Read Device Name: the Device Reports No Name".into()));
    }
    let mut bytes = vec![0u8; size as usize];
    unsafe { SetupDiGetDeviceRegistryPropertyW(devices.0, device, property, None, Some(&mut bytes), None) }.map_err(probe)?;
    let units = bytes.as_chunks::<2>().0.iter().map(|chunk| u16::from_ne_bytes(*chunk)).take_while(|unit| *unit != 0).collect::<Vec<_>>();
    Ok(String::from_utf16_lossy(&units))
}

/* === Tests === */
