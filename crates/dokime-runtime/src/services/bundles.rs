/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Plugin Target

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn native_target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        _ => "unsupported",
    }
}
fn portable(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 100 || name.eq_ignore_ascii_case("manifest.json") {
        return Err(invalid("Use Portable Relative Bundle Paths of 1 to 100 Bytes"));
    }
    for part in name.split('/') {
        let stem = part.split('.').next().unwrap_or_default().to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with('.')
            || !part.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
            || matches!(
                stem.as_str(),
                "CON"
                    | "PRN"
                    | "AUX"
                    | "NUL"
                    | "COM1"
                    | "COM2"
                    | "COM3"
                    | "COM4"
                    | "COM5"
                    | "COM6"
                    | "COM7"
                    | "COM8"
                    | "COM9"
                    | "LPT1"
                    | "LPT2"
                    | "LPT3"
                    | "LPT4"
                    | "LPT5"
                    | "LPT6"
                    | "LPT7"
                    | "LPT8"
                    | "LPT9"
            )
        {
            return Err(invalid("Remove Unsafe or Nonportable Bundle Paths"));
        }
    }
    Ok(())
}

/* === Tests === */

/* === Private Functions === */

/* === Tests === */
