/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg(target_os = "linux")]

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
use dokime_contract::core::Value;
use dokime_host::collectors::bluetooth::BluetoothCollector;
use std::{
    os::unix::fs::PermissionsExt,
    process::Command,
    time::{Duration, Instant},
};

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

/* === Private Functions === */

/* === Tests === */

pub(crate) fn public_probe_child() {
    let Ok(mode) = std::env::var("DOKIME_PROBE_CASE") else {
        for mode in ["success", "failure", "timeout"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "domain_contract", "--nocapture"])
                .env("DOKIME_PROBE_CASE", mode)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    };
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("bluetoothctl");
    std::fs::write(
        &executable,
        r#"#!/usr/bin/env python3
import os, sys, time
from pathlib import Path
if sys.argv[1] != "show":
    sys.exit(0)
Path(os.environ["DOKIME_PROBE_PID"]).write_text(str(os.getpid()))
if os.environ["DOKIME_PROBE_CASE"] == "timeout":
    time.sleep(30)
sys.stdout.write("o" * (512 * 1024) + "\nController fixture\nPowered: yes\n")
sys.stdout.flush()
sys.stderr.write("e" * (512 * 1024))
sys.stderr.flush()
sys.exit(7 if os.environ["DOKIME_PROBE_CASE"] == "failure" else 0)
"#,
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(root.path().to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    // This exact test runs alone in the child; all environment changes precede
    // the collector's worker thread and do not affect the parent test process.
    unsafe {
        std::env::set_var("PATH", path);
        std::env::set_var("DOKIME_PROBE_PID", root.path().join("pid"));
    }
    let start = Instant::now();
    let sample = BluetoothCollector::default().collect().unwrap();
    match mode.as_str() {
        "success" => {
            assert_eq!(sample.values["probe_status"], Value::from("ok"));
            assert_eq!(sample.values["bluetooth_adapter"], Value::from("fixture"));
        }
        "failure" => {
            assert_eq!(sample.values["probe_status"], Value::from("error"));
            assert_eq!(
                sample.values["probe_error"],
                Value::from("e".repeat(512 * 1024))
            );
        }
        "timeout" => {
            assert_eq!(sample.values["probe_status"], Value::from("error"));
            assert_eq!(
                sample.values["probe_error"],
                Value::from("bluetoothctl could not be started: command timed out")
            );
            assert!(start.elapsed() < Duration::from_secs(10));
        }
        _ => panic!("unknown probe case"),
    }
    let pid = std::fs::read_to_string(root.path().join("pid")).unwrap();
    assert!(
        !std::path::Path::new("/proc").join(pid).exists(),
        "probe must be reaped before collection returns"
    );
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: public_probe_child");
        public_probe_child();
    }
}
