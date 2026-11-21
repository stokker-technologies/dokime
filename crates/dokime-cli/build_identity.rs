/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Format Human Build Identity

/* === Modules === */

/* === Re-Exports === */

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

pub fn version(version: &str, revision: &str, channel: &str) -> String {
    let commit = revision.strip_suffix("-dirty").unwrap_or(revision);
    let valid = (7..=64).contains(&commit.len()) && commit.bytes().all(|byte| byte.is_ascii_hexdigit());
    let revision = if valid {
        let short = &commit[..commit.len().min(8)];
        let dirty = if revision.ends_with("-dirty") { "-dirty" } else { "" };
        format!("{short}{dirty}")
    } else {
        "unknown".to_owned()
    };
    let channel = if channel.is_empty() { "local" } else { channel };
    format!("{version} ({revision}, {channel})")
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn format_build_identity() {
        for (package, revision, channel, expected) in [
            ("1.2.3", "0123456789abcdef", "nightly", "1.2.3 (01234567, nightly)"),
            ("2.3.4", "abcdef0123456789-dirty", "", "2.3.4 (abcdef01-dirty, local)"),
            ("3.4.5", "unknown", "release", "3.4.5 (unknown, release)"),
            ("4.5.6", "0123456789abcdef", "", "4.5.6 (01234567, local)"),
            ("5.6.7", "", "", "5.6.7 (unknown, local)"),
            ("6.7.8", "not-a-commit", "nightly", "6.7.8 (unknown, nightly)"),
        ] {
            assert_eq!(version(package, revision, channel), expected);
        }
    }
}
