/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Organization Pack Lifecycle Stories

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, thread};

// Third Party Imports

// Local Imports
use dokime_gui::testing;
use dokime_runtime::process;

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
fn bundled_pack_loads_without_a_system_temp_directory() {
    const CHILD: &str = "DOKIME_TEST_MISSING_TEMP";
    if env::var_os(CHILD).is_some() {
        assert!(!env::temp_dir().exists());
        let loaded = testing::inspect_bundled().unwrap();
        assert!(!loaded["logos"].as_object().unwrap().is_empty());
        assert!(!loaded["css"].as_str().unwrap().is_empty());
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("nonexistent");
    let status = process::command(env::current_exe().unwrap())
        .args(["--exact", thread::current().name().unwrap(), "--nocapture"])
        .env(CHILD, "1")
        .env("TMP", &missing)
        .env("TEMP", &missing)
        .env("TMPDIR", &missing)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(!missing.exists());
}
