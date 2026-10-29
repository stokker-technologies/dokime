/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Active Power Plan Read Through The Power Management API, Without Starting powercfg.

use std::ptr::null_mut;

use windows::{
    Win32::{
        Foundation::{ERROR_SUCCESS, HLOCAL, LocalFree},
        System::Power::{PowerGetActiveScheme, PowerReadFriendlyName},
    },
    core::GUID,
};

// The Active Plan's GUID And Friendly Name.
pub fn active_scheme() -> Option<(String, Option<String>)> {
    let mut active: *mut GUID = null_mut();
    // SAFETY: The API writes a LocalAlloc'd GUID into `active`, which is copied and freed once.
    let code = unsafe { PowerGetActiveScheme(None, &mut active) };
    if code != ERROR_SUCCESS || active.is_null() {
        return None;
    }
    // SAFETY: A successful call returned a valid GUID allocation.
    let guid = unsafe { *active };
    let allocation = HLOCAL(active.cast());
    // SAFETY: The allocation came from LocalAlloc and is not used again.
    unsafe { LocalFree(Some(allocation)) };
    Some((guid_string(&guid), friendly_name(&guid)))
}

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

// Lowercase Hyphenated Form, As powercfg Prints It.
fn guid_string(guid: &GUID) -> String {
    let [a, b, c, d, e, f, g, h] = guid.data4;
    format!("{:08x}-{:04x}-{:04x}-{a:02x}{b:02x}-{c:02x}{d:02x}{e:02x}{f:02x}{g:02x}{h:02x}", guid.data1, guid.data2, guid.data3)
}

#[cfg(test)]
mod tests {
    #[test]
    fn guid_prints_like_powercfg() {
        let guid = windows::core::GUID::from_u128(0x381b4222_f694_41f0_9685_ff5bb260df2e);
        assert_eq!(super::guid_string(&guid), "381b4222-f694-41f0-9685-ff5bb260df2e");
    }

    #[test]
    fn active_scheme_reads_without_a_process() {
        let (guid, _) = super::active_scheme().expect("Windows has an active power scheme");
        assert_eq!(guid.len(), 36);
    }
}
