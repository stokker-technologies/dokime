/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Power Plan and Power Mode Reads Through the Power Management API, Without Starting powercfg

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::ptr::null_mut;

// Third Party Imports
use windows::Win32::Foundation::{ERROR_SUCCESS, HLOCAL, LocalFree};
use windows::Win32::System::Power::{PowerGetActiveScheme, PowerReadFriendlyName};
use windows::core::GUID;

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The Active Power Plan's Identifier and Friendly Name.
pub fn active_plan() -> Result<(String, Option<String>)> {
    let mut active: *mut GUID = null_mut();
    // SAFETY: The API writes a LocalAlloc'd GUID into `active`, which is copied and freed once.
    let code = unsafe { PowerGetActiveScheme(None, &mut active) };
    if code != ERROR_SUCCESS || active.is_null() {
        return Err(Error::Invalid(format!(
            "Read the Active Power Plan: PowerGetActiveScheme Returned {}; Check the Windows Power Service",
            code.0
        )));
    }
    // SAFETY: A successful call returned a valid GUID allocation.
    let guid = unsafe { *active };
    // SAFETY: The allocation came from LocalAlloc and is not used again.
    unsafe { LocalFree(Some(HLOCAL(active.cast()))) };
    Ok((guid_string(&guid), friendly_name(&guid)))
}

/// The Power Mode a Plan Name Selects.
pub fn mode(name: &str) -> &'static str {
    let normalized = name.to_ascii_lowercase();
    if normalized.contains("ultimate") || normalized.contains("high performance") {
        "best_performance"
    } else if normalized.contains("power saver") || normalized.contains("energy saver") {
        "best_efficiency"
    } else if normalized.contains("balanced") {
        "balanced"
    } else {
        "unknown"
    }
}

/// The Performance Bias of a Power Mode, From Zero (Efficiency) to 100 (Performance).
pub fn performance_bias(mode: &str) -> i64 {
    match mode {
        "best_performance" => 100,
        "best_efficiency" => 0,
        _ => 50,
    }
}

/* === Private Functions === */

fn friendly_name(scheme: &GUID) -> Option<String> {
    let mut size = 0u32;
    // SAFETY: A null buffer asks only for the required byte size, so the status is not the answer.
    let _ = unsafe { PowerReadFriendlyName(None, Some(scheme), None, None, None, &mut size) };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u16; (size as usize).div_ceil(2)];
    let output = buffer.as_mut_ptr().cast::<u8>();
    // SAFETY: The buffer holds at least `size` bytes.
    let code = unsafe { PowerReadFriendlyName(None, Some(scheme), None, None, Some(output), &mut size) };
    if code != ERROR_SUCCESS {
        return None;
    }
    let end = buffer.iter().position(|unit| *unit == 0).unwrap_or(buffer.len());
    let name = String::from_utf16_lossy(&buffer[..end]).trim().to_owned();
    (!name.is_empty()).then_some(name)
}

/// Lowercase Hyphenated Form, as powercfg Prints It.
fn guid_string(guid: &GUID) -> String {
    let [a, b, c, d, e, f, g, h] = guid.data4;
    format!("{:08x}-{:04x}-{:04x}-{a:02x}{b:02x}-{c:02x}{d:02x}{e:02x}{f:02x}{g:02x}{h:02x}", guid.data1, guid.data2, guid.data3)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn guid_prints_like_powercfg() {
        let guid = windows::core::GUID::from_u128(0x381b4222_f694_41f0_9685_ff5bb260df2e);
        assert_eq!(super::guid_string(&guid), "381b4222-f694-41f0-9685-ff5bb260df2e");
    }

    #[test]
    fn plan_names_select_modes_and_biases() {
        for (name, mode, bias) in [
            ("Ultimate Performance", "best_performance", 100),
            ("High performance", "best_performance", 100),
            ("Power saver", "best_efficiency", 0),
            ("Balanced", "balanced", 50),
            ("Custom", "unknown", 50),
        ] {
            assert_eq!(super::mode(name), mode);
            assert_eq!(super::performance_bias(mode), bias);
        }
    }
}
