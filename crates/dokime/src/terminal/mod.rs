/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod error;
mod output;

/* === Re-Exports === */

pub use error::{Error, Result};
pub use output::{ColorMode, Progress, Terminal, color_mode, set_color_mode};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// A byte count the way a person reads one: binary units, one decimal from
// kibibytes upwards, whole bytes below.
pub fn size_label(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{value:.1} {}", UNITS[unit]) }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::size_label;

    #[test]
    fn readable_size_units() {
        assert_eq!(size_label(0), "0 B");
        assert_eq!(size_label(1023), "1023 B");
        assert_eq!(size_label(1024), "1.0 KiB");
        assert_eq!(size_label(45_298_176), "43.2 MiB");
        assert_eq!(size_label(134_217_728), "128.0 MiB");
        assert_eq!(size_label(5 * 1024 * 1024 * 1024), "5.0 GiB");
    }
}
