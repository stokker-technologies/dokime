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
use std::process::Stdio;

// Local Imports
use crate::support::{
    bridge::{Harness, Local, Process},
    server::Fixture,
};

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
fn desktop_runs_matrix() {
    dokime_gui::test_support::run(async {
        // Admit the SDK Fixture Through the Same Host Adapter as the Running Shell
        let fixture = Fixture::new().await;
        let mut plugin = fixture.thermal().await;
        let harness = Harness::new(fixture.session.clone());
        let mut failure = None;
        for scenario in ["complete", "refuse", "cancel"] {
            // Drive the Canonical Frontend Client Through Real Tauri IPC
            let mut child = Process(
                dokime_runtime_next::process::command("node")
                    .env("DOKIME_TEST_GENERATION", fixture.session.status().generation)
                    .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-execution-matrix.mjs"))
                    .arg(scenario)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .unwrap(),
            );
            let status = harness.drive(&mut child.0, |_, _| Local::Dispatch, |line| panic!("Unexpected Observation: {line}"));
            if !status.success() {
                failure = Some(scenario);
                break;
            }
        }
        fixture.close().await;
        plugin.close().await.unwrap();
        assert!(failure.is_none(), "Desktop Execution Scenario Failed: {failure:?}");
    })
}
