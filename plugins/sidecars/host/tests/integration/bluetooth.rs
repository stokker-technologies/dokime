/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The Bluetooth collector against a scripted `bluetoothctl` on `PATH`. Each
// story changes `PATH` for its own process: the native runner (nextest) runs
// every story in a process of its own. The fixture's `time.sleep(30)` is the
// lifetime of the stalled child the collector must time out and reap.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::os::unix::fs::PermissionsExt;

// Third Party Imports
use model::values::Value;

// Local Imports
use dokime_domain::model;
use dokime_host::HostStreamSample;
use dokime_host::collectors::bluetooth::BluetoothCollector;

/* === Constants === */

const FIXTURE: &str = r#"#!/usr/bin/env python3
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
"#;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn collect_case(mode: &str) -> HostStreamSample {
    // Scripted Probe On PATH
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("bluetoothctl");
    std::fs::write(&executable, FIXTURE).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path =
        std::env::join_paths(std::iter::once(root.path().to_owned()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())))
            .unwrap();
    // All environment changes precede the collector's worker thread, and the
    // story owns its process.
    unsafe {
        std::env::set_var("PATH", path);
        std::env::set_var("DOKIME_PROBE_PID", root.path().join("pid"));
        std::env::set_var("DOKIME_PROBE_CASE", mode);
    }

    // Collection And Reaped Child
    let sample = BluetoothCollector::new(dokime_host::HostOptions::default()).collect().unwrap();
    let pid = std::fs::read_to_string(root.path().join("pid")).unwrap();
    assert!(!std::path::Path::new("/proc").join(pid).exists(), "probe must be reaped before collection returns");
    sample
}

/* === Tests === */

#[test]
fn probe_child_succeeds() {
    let sample = collect_case("success");
    assert_eq!(sample.values["probe_status"], Value::from("ok"));
    assert_eq!(sample.values["bluetooth_adapter"], Value::from("fixture"));
}

#[test]
fn probe_child_fails() {
    let sample = collect_case("failure");
    assert_eq!(sample.values["probe_status"], Value::from("error"));
    assert_eq!(sample.values["probe_error"], Value::from("e".repeat(512 * 1024)));
}

#[test]
fn probe_child_reaped() {
    let sample = collect_case("timeout");
    assert_eq!(sample.values["probe_status"], Value::from("error"));
    assert_eq!(sample.values["probe_error"], Value::from("bluetoothctl could not be started: command timed out"));
}
