/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Backend Discovery Precedence, Probed By Re-Running This Binary In A Fixture Layout.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{fs, path::Path};

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
fn discovery_refuses_precedence() {
    const PROBE: &str = "DOKIME_DISCOVERY_TEST_EXPECTED";
    if let Some(expected) = std::env::var_os(PROBE) {
        assert_eq!(dokime_gui::native_session::backend::locate().unwrap(), std::path::PathBuf::from(expected));
        return;
    }
    // Re-exec a hard link beside fixture candidates. Child-only environment
    // avoids races with native sessions and never consults a developer keyring.
    let executable = std::env::current_exe().unwrap();
    let fixture = tempfile::tempdir_in(executable.parent().unwrap()).unwrap();
    let harness = fixture.path().join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    fs::hard_link(&executable, &harness).unwrap();
    let home = fixture.path().join("home");
    let path = fixture.path().join("path");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&path).unwrap();
    let name = format!("dokime-server{}", std::env::consts::EXE_SUFFIX);
    let bundled = fixture.path().join(&name);
    let in_home = home.join(&name);
    let on_path = path.join(&name);
    for candidate in [&bundled, &in_home, &on_path] {
        fs::write(candidate, "backend").unwrap();
    }
    let probe = |expected: &Path| {
        let output = std::process::Command::new(&harness)
            .args(["--exact", "discovery::discovery_refuses_precedence", "--nocapture"])
            .env(PROBE, expected)
            .env("DOKIME_HOME", &home)
            .env("PATH", &path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{stdout}\n{}", String::from_utf8_lossy(&output.stderr));
        // A renamed story would select nothing and pass vacuously.
        assert!(stdout.contains("1 passed"), "the probe ran exactly one story: {stdout}");
    };
    probe(&bundled);
    fs::remove_file(&bundled).unwrap();
    probe(&in_home);
    fs::remove_file(&in_home).unwrap();
    fs::create_dir(&in_home).unwrap();
    probe(&on_path);
    fs::remove_dir(&in_home).unwrap();
    probe(&on_path);
}
