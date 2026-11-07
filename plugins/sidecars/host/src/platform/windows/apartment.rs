/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Multithreaded Apartment The Host's COM And WinRT Probes Share.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::OnceLock;

// Third Party Imports
use windows::Win32::System::Com::CoIncrementMTAUsage;

// Local Imports
use crate::{Error, Result};

/* === Constants === */

static RETAINED: OnceLock<std::result::Result<(), String>> = OnceLock::new();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Keeps The Apartment Alive Until The Process Exits. When Its Last Thread Leaves, COM Unloads Every
// In-Process Server, Yet System Worker Threads Still Run Inside Them And windows-rs Keeps Their
// Activation Factories Cached, So Either One Faults On Unmapped Code.
pub fn retain() -> Result<()> {
    // SAFETY: Incrementing the usage has no preconditions; the usage is deliberately never released.
    let retained = RETAINED.get_or_init(|| unsafe { CoIncrementMTAUsage() }.map(drop).map_err(|error| error.to_string()));
    retained.clone().map_err(Error::Probe)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use crate::domain::controls::Radio;
    use windows::Win32::System::Com::{APTTYPE, APTTYPE_MTA, APTTYPEQUALIFIER, APTTYPEQUALIFIER_IMPLICIT_MTA, CoGetApartmentType};

    #[test]
    fn apartment_outlives_probe_threads() {
        let timing = crate::HostOptions::default().timing;
        for _ in 0..2 {
            // Whether This Machine Has The Radio Is Irrelevant; Only The Apartment Is Under Test.
            std::thread::scope(|scope| scope.spawn(|| drop(crate::host::controls::read_radio(Radio::Bluetooth, &timing))).join().unwrap());
        }
        let apartment = std::thread::spawn(|| {
            let (mut kind, mut qualifier) = (APTTYPE::default(), APTTYPEQUALIFIER::default());
            // SAFETY: Both outputs are valid for this call, on a thread that never initialized COM.
            unsafe { CoGetApartmentType(&mut kind, &mut qualifier) }.map(|()| (kind, qualifier))
        });
        assert_eq!(apartment.join().unwrap().unwrap(), (APTTYPE_MTA, APTTYPEQUALIFIER_IMPLICIT_MTA));
    }
}
