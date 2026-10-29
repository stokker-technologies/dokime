/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Internet Connectivity As NetworkManager Last Determined It.

use std::time::Duration;

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

fn parse(state: &str) -> Option<bool> {
    match state.trim() {
        "full" => Some(true),
        "none" | "portal" | "limited" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_full_connectivity_is_internet() {
        assert_eq!(super::parse("full\n"), Some(true));
        assert_eq!(super::parse("portal"), Some(false));
        assert_eq!(super::parse("unknown"), None);
    }
}
