/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Multithreaded Apartment the Host's COM and WinRT Probes Share

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::ffi::c_void;

// Third Party Imports
use windows::Win32::System::Com::{CO_MTA_USAGE_COOKIE, CoDecrementMTAUsage, CoIncrementMTAUsage};

// Local Imports
use dokime::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One MTA Usage Lease, Released After Its Collection or Probe Drops Its COM Objects.
pub struct Lease {
    cookie: usize,
    release: fn(usize),
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Lease {
    pub fn acquire() -> Result<Self> {
        // SAFETY: Incrementing MTA Usage Has No Preconditions; Drop Releases This Cookie Once.
        let cookie = unsafe { CoIncrementMTAUsage() }
            .map_err(|error| Error::Invalid(format!("Join the Host COM Apartment: {error}; Restart the Plugin")))?;
        Ok(Self { cookie: cookie.0 as usize, release })
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        (self.release)(self.cookie);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn release(cookie: usize) {
    // SAFETY: The Cookie Came From CoIncrementMTAUsage and Is Released Exactly Once.
    let _ = unsafe { CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE(cookie as *mut c_void)) };
}

/* === Tests === */

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::Lease;

    pub(crate) fn fake(releases: Arc<AtomicUsize>) -> Lease {
        Lease {
            cookie: Arc::into_raw(releases) as usize,
            release: release_fake,
        }
    }

    fn release_fake(cookie: usize) {
        // SAFETY: fake Transfers One Arc Ownership to the Lease, Which Releases It Once.
        let releases = unsafe { Arc::from_raw(cookie as *const AtomicUsize) };
        releases.fetch_add(1, Ordering::SeqCst);
    }
}
