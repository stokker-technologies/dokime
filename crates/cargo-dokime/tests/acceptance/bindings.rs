/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export The Built-In Operation Catalog Through The Authoring Command.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

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

#[test]
fn proto_catalog_export() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("proto");
    let nextest_tool = std::env::var_os("NEXTEST_BIN_EXE_cargo-dokime");
    let cargo_tool = std::env::var_os("CARGO_BIN_EXE_cargo-dokime");
    let tool = nextest_tool.or(cargo_tool);
    let tool = tool.expect("the test runner supplies cargo-dokime");
    let output = dokime::domain::process::command(tool)
        .args(["dokime", "bindings", "proto", "--output"])
        .arg(&destination)
        .current_dir(root.path())
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    let manifest = std::fs::read_to_string(destination.join("manifest.txt")).unwrap();
    assert!(manifest.lines().any(|name| name.ends_with(".proto")));
    for name in manifest.lines() {
        assert!(destination.join(name).is_file(), "missing exported {name}");
    }
    assert!(destination.join("descriptors.bin").metadata().unwrap().len() > 0);
    let artifacts = harmos_packaging::proto::Artifacts::read(&destination).unwrap();
    let actual = artifacts.operation_contracts().unwrap();
    let mut expected = dokime::harmos::operations::Contracts::of::<dokime::operations::Operations>().unwrap();
    expected.sort_unstable_by_key(|contract| {
        let operation = contract.operation();
        operation.id.to_string()
    });
    assert!(!actual.is_empty());
    assert_eq!(serde_json::to_value(actual).unwrap(), serde_json::to_value(expected).unwrap());
}
