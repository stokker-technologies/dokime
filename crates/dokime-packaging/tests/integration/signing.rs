/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */
// Exercise Credential Selection and the No-Certificate Path Without Network Access.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Command;

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
fn credential_selection_and_unsigned_packaging() {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/installer/authenticode.ps1");
    let names = [
        "AZURE_TENANT_ID",
        "AZURE_CLIENT_ID",
        "AZURE_CLIENT_SECRET",
        "DOKIME_SIGNING_ENDPOINT",
        "DOKIME_SIGNING_ACCOUNT",
        "DOKIME_SIGNING_PROFILE",
        "DOKIME_SIGNING_PFX",
        "DOKIME_SIGNING_PFX_PASSWORD",
        "DOKIME_AUTHENTICODE_PROVIDER",
    ];
    for (operation, credentials, expected) in [
        ("Select", vec![], Some("none")),
        ("Sign", vec![], None),
        ("Select", vec![("DOKIME_SIGNING_PFX", "fixture"), ("DOKIME_SIGNING_PFX_PASSWORD", "password")], Some("pfx")),
        ("Select", names[..6].iter().map(|name| (*name, "fixture")).collect(), Some("azure")),
    ] {
        let mut command = Command::new("pwsh");
        for name in names {
            command.env_remove(name);
        }
        let output = command
            .args(["-NoProfile", "-NonInteractive", "-File"])
            .arg(&script)
            .args(["-Operation", operation, "-Path", "absent-file.exe"])
            .envs(credentials)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        if let Some(expected) = expected {
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
        }
    }

    for credentials in [
        vec![("AZURE_CLIENT_SECRET", "incomplete")],
        vec![("DOKIME_SIGNING_PFX", "incomplete")],
        vec![("DOKIME_SIGNING_PFX_PASSWORD", "incomplete")],
        vec![("AZURE_CLIENT_SECRET", "conflict"), ("DOKIME_SIGNING_PFX", "conflict")],
    ] {
        let mut command = Command::new("pwsh");
        for name in names {
            command.env_remove(name);
        }
        let output = command
            .args(["-NoProfile", "-NonInteractive", "-File"])
            .arg(&script)
            .args(["-Operation", "Select"])
            .envs(credentials)
            .output()
            .unwrap();
        assert!(!output.status.success());
    }

    let mut command = Command::new("pwsh");
    for name in names {
        command.env_remove(name);
    }
    let output = command
        .args(["-NoProfile", "-NonInteractive", "-File"])
        .arg(&script)
        .args(["-Operation", "Sign", "-Path", "absent-file.exe"])
        .env("DOKIME_AUTHENTICODE_PROVIDER", "pfx")
        .output()
        .unwrap();
    assert!(!output.status.success());
}
