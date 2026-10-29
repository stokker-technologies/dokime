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

use crate::{Error, Result};
use windows::{
    Win32::{
        Devices::DeviceAndDriverInstallation::*,
        Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
        System::Registry::{KEY_QUERY_VALUE, RegCloseKey, RegQueryValueExW},
    },
    core::{GUID, w},
};

// Standard Library Imports

// Third Party Imports

// Local Imports

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

pub fn adapter_enabled(id: &str) -> Result<bool> {
    let devices = open(&GUID_DEVCLASS_NET)?;
    let device = adapter(&devices, id)?;
    let mut status = CM_DEVNODE_STATUS_FLAGS::default();
    let mut problem = CM_PROB::default();
    let code = unsafe { CM_Get_DevNode_Status(&mut status, &mut problem, device.DevInst, 0) };
    if code != CR_SUCCESS {
        return Err(Error::Probe(format!("CM_Get_DevNode_Status: {}", code.0)));
    }
    Ok(problem != CM_PROB_DISABLED)
}

pub fn set_adapter(id: &str, enabled: bool) -> Result<()> {
    let devices = open(&GUID_DEVCLASS_NET)?;
    let device = adapter(&devices, id)?;
    let mut install = SP_DEVINSTALL_PARAMS_W {
        cbSize: std::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as u32,
        ..Default::default()
    };
    unsafe { SetupDiGetDeviceInstallParamsW(devices.0, Some(&device), &mut install) }.map_err(probe)?;
    install.Flags |= DI_QUIETINSTALL;
    unsafe { SetupDiSetDeviceInstallParamsW(devices.0, Some(&device), &install) }.map_err(probe)?;
    let change = SP_PROPCHANGE_PARAMS {
        ClassInstallHeader: SP_CLASSINSTALL_HEADER {
            cbSize: std::mem::size_of::<SP_CLASSINSTALL_HEADER>() as u32,
            InstallFunction: DIF_PROPERTYCHANGE,
        },
        StateChange: if enabled { DICS_ENABLE } else { DICS_DISABLE },
        Scope: DICS_FLAG_GLOBAL,
        HwProfile: 0,
    };
    unsafe {
        SetupDiSetClassInstallParamsW(devices.0, Some(&device), Some(&change.ClassInstallHeader), std::mem::size_of_val(&change) as u32)
    }
    .map_err(probe)?;
    unsafe { SetupDiCallClassInstaller(DIF_PROPERTYCHANGE, devices.0, Some(&device)) }.map_err(probe)?;
    if adapter_enabled(id)? != enabled {
        return Err(Error::Probe("adapter state did not settle to the requested state; a restart may be required".into()));
    }
    Ok(())
}

/* === Private Functions === */

fn probe(error: impl std::fmt::Display) -> Error {
    Error::Probe(error.to_string())
}
fn open(class: &GUID) -> Result<Devices> {
    unsafe { SetupDiGetClassDevsW(Some(class), None, None, DIGCF_PRESENT) }.map(Devices).map_err(probe)
}
fn entry(devices: &Devices, index: u32) -> Result<Option<SP_DEVINFO_DATA>> {
    let mut device = SP_DEVINFO_DATA {
        cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };
    match unsafe { SetupDiEnumDeviceInfo(devices.0, index, &mut device) } {
        Ok(()) => Ok(Some(device)),
        Err(error) if error.code() == windows::core::HRESULT::from_win32(ERROR_NO_MORE_ITEMS.0) => Ok(None),
        Err(error) => Err(probe(error)),
    }
}
fn adapter(devices: &Devices, id: &str) -> Result<SP_DEVINFO_DATA> {
    let mut index = 0;
    while let Some(device) = entry(devices, index)? {
        index += 1;
        let Ok(key) = (unsafe { SetupDiOpenDevRegKey(devices.0, &device, DICS_FLAG_GLOBAL.0, 0, DIREG_DRV, KEY_QUERY_VALUE.0) }) else {
            continue;
        };
        let mut size = 0;
        let code = unsafe { RegQueryValueExW(key, w!("NetCfgInstanceId"), None, None, None, Some(&mut size)) };
        if code != ERROR_SUCCESS || size == 0 {
            let _ = unsafe { RegCloseKey(key) };
            continue;
        }
        let mut value = vec![0u16; (size as usize).div_ceil(2)];
        let code = unsafe { RegQueryValueExW(key, w!("NetCfgInstanceId"), None, None, Some(value.as_mut_ptr().cast()), Some(&mut size)) };
        let _ = unsafe { RegCloseKey(key) };
        if code == ERROR_SUCCESS {
            let end = value.iter().position(|unit| *unit == 0).unwrap_or(value.len());
            if String::from_utf16_lossy(&value[..end]).eq_ignore_ascii_case(id) {
                return Ok(device);
            }
        }
    }
    Err(Error::Probe(format!("adapter {id} was not found")))
}
/* === Tests === */
