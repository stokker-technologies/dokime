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
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime_packaging::{Error, process::run};

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn credential_selection_and_unsigned_packaging() {
    let policy: toml::Value = toml::from_str(include_str!("../../../../tools/project.toml")).unwrap();
    let seconds = policy["conventions"]["testing"]["deadlines"]["acceptance"].as_integer().unwrap();
    let deadline = Duration::from_secs(seconds.try_into().unwrap());
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
        let mut command = dokime::domain::process::command("pwsh");
        for name in names {
            command.env_remove(name);
        }
        command
            .args(["-NoProfile", "-NonInteractive", "-File"])
            .arg(&script)
            .args(["-Operation", operation, "-Path", "absent-file.exe"])
            .envs(credentials);
        let output = run(command, "Signing credentials", Some(deadline), 65536, 65536).await.unwrap();
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
        let mut command = dokime::domain::process::command("pwsh");
        for name in names {
            command.env_remove(name);
        }
        command.args(["-NoProfile", "-NonInteractive", "-File"]).arg(&script).args(["-Operation", "Select"]).envs(credentials);
        let result = run(command, "Signing credentials", Some(deadline), 65536, 65536).await;
        assert!(matches!(result, Err(Error::Process { .. })));
    }

    let mut command = dokime::domain::process::command("pwsh");
    for name in names {
        command.env_remove(name);
    }
    command
        .args(["-NoProfile", "-NonInteractive", "-File"])
        .arg(&script)
        .args(["-Operation", "Sign", "-Path", "absent-file.exe"])
        .env("DOKIME_AUTHENTICODE_PROVIDER", "pfx");
    let result = run(command, "Signing credentials", Some(deadline), 65536, 65536).await;
    assert!(matches!(result, Err(Error::Process { .. })));
}
