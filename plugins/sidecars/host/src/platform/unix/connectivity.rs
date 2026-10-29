/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Internet Connectivity As NetworkManager Last Determined It.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

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

// `nmcli networking connectivity` Reports The Cached State Without Checking Again.
pub fn internet(timeout: Duration) -> Option<bool> {
    let mut command = dokime_domain::process::command("nmcli");
    command.args(["networking", "connectivity"]);
    let output = crate::collectors::run_command_bounded(command, timeout).ok()?;
    if !output.status.success() {
        return None;
    }
    parse(&String::from_utf8_lossy(&output.stdout))
}

/* === Private Functions === */

fn parse(state: &str) -> Option<bool> {
    match state.trim() {
        "full" => Some(true),
        "none" | "portal" | "limited" => Some(false),
        _ => None,
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    #[test]
    fn only_full_connectivity_is_internet() {
        assert_eq!(super::parse("full\n"), Some(true));
        assert_eq!(super::parse("portal"), Some(false));
        assert_eq!(super::parse("unknown"), None);
    }
}
