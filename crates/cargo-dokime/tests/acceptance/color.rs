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
// Standard Library Imports
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{fs, process::Command};
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

/* === Private Functions === */

/* === Tests === */
#[test]
fn build_color_policy() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"color-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\n",
    )
    .unwrap();
    fs::write(project.path().join("src/main.rs"), "fn main() { compile_error!(\"color-fixture-diagnostic\"); }\n").unwrap();
    for (mode, inherited, colored) in [
        ("always", Some("never"), true),
        ("never", Some("always"), false),
        ("auto", Some("always"), true),
        ("auto", Some("never"), false),
        ("auto", None, false),
    ] {
        let mut command = Command::new(super::support::tool());
        #[cfg(windows)]
        command.creation_flags(0x0800_0000);
        command
            .current_dir(project.path())
            .args(["dokime", "build", "--offline", "--color", mode])
            .env_remove("CARGO_TERM_COLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("CLICOLOR")
            .env_remove("NO_COLOR");
        if let Some(inherited) = inherited {
            command.env("CARGO_TERM_COLOR", inherited);
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        let diagnostic = stderr
            .lines()
            .find(|line| line.contains("color-fixture-diagnostic") && !line.contains("compile_error!"))
            .unwrap_or_else(|| panic!("compiler diagnostic missing: {stderr}"));
        assert_eq!(diagnostic.contains('\u{1b}'), colored, "--color {mode}, CARGO_TERM_COLOR={inherited:?}: {stderr}");
        if !colored {
            assert!(!stderr.contains('\u{1b}'), "unexpected ANSI: {stderr}");
        }
        assert!(output.stdout.is_empty(), "human diagnostics belong on stderr");
    }
}
