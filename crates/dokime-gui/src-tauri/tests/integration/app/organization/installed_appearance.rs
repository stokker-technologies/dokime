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

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

// Local Imports

use super::*;

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
    if std::env::var_os(CHILD).is_some() {
        assert!(!std::env::temp_dir().exists());
        let pack = dokime_gui::testing::resolve_directory(None).unwrap();
        let loaded = implementation::inspect_resolved(pack).unwrap();
        assert!(!loaded["logos"].as_object().unwrap().is_empty());
        assert!(!loaded["css"].as_str().unwrap().is_empty());
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("nonexistent");
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", std::thread::current().name().unwrap(), "--nocapture"])
        .env(CHILD, "1")
        .env("TMP", &missing)
        .env("TEMP", &missing)
        .env("TMPDIR", &missing)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(!missing.exists());
}

#[test]
fn memory_css_matches_the_disk_fixture() {
    let root = tempfile::tempdir().unwrap();
    let css = "/* fixture */@font-face{src:url('fonts/test.woff2')}a{background:url(logos/test.svg)}b{background:url(images/test.png)}";
    let fixtures: [(&str, &[u8], &str); 3] = [
        ("fonts/test.woff2", &[0, 1, 2, 255], "font/woff2"),
        ("logos/test.svg", LOGO, "image/svg+xml"),
        ("images/test.png", &[137, 80, 78, 71], "image/png"),
    ];
    let mut pack = dokime_gui::testing::resolve_directory(None).unwrap();
    let mut expected = css.replace("/* fixture */", "");
    for (relative, bytes, mime) in fixtures {
        let path = root.path().join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let disk = std::fs::read(path).unwrap();
        let url = format!("url(\"data:{mime};base64,{}\")", BASE64.encode(disk));
        expected = expected.replace(&format!("url('{relative}')"), &url).replace(&format!("url({relative})"), &url);
        pack.files.insert(relative.into(), bytes.to_vec());
    }
    assert_eq!(implementation::inspect_css(&pack, css).unwrap().as_bytes(), expected.as_bytes());
    assert!(implementation::inspect_css(&pack, "a{background:url(../outside.svg)}").is_err());
    assert!(implementation::inspect_css(&pack, "a{background:url(missing.svg)}").is_err());
}
