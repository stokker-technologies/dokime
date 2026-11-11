/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Checked-In Wire Bindings.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fs, path::Path};

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
fn retired_plot_kinds_require_explicit_replacements() {
    use dokime_protocol::v1::PlotKind;
    for (name, number, replacement) in [
        ("GROUPED_BAR", 3, "BAR with arrangement=grouped"),
        ("STACKED_BAR", 4, "BAR with arrangement=stacked"),
        ("DONUT", 6, "PIE with hole=0.5"),
        ("PARETO", 7, "BAR with pareto=true"),
        ("XY_INTERVAL", 9, "XY with interval=sigma"),
        ("HEATMAP", 11, "CONTOUR"),
        ("XY_VALUE_HEATMAP", 12, "CONTOUR"),
        ("WIFI_CHANNEL_OCCUPANCY", 13, "GRID_CHART with declared rows, columns and frames"),
        ("BOX_SUMMARY", 16, "BOX_PLOT"),
        ("AGGREGATE_TABLE", 17, "a document Summary card"),
    ] {
        assert_eq!(PlotKind::parse_name(name), Err(format!("{name}: use {replacement}")));
        assert_eq!(PlotKind::from_declared_number(number), Err(format!("{name}: use {replacement}")));
    }
    for kind in [PlotKind::Contour, PlotKind::GridChart, PlotKind::BoxPlot] {
        assert_eq!(PlotKind::parse_name(kind.as_str_name()), Ok(kind));
        assert_eq!(PlotKind::from_declared_number(kind as i32), Ok(kind));
    }
}

#[test]
fn generated_code_matches_proto() -> Result<(), Box<dyn std::error::Error>> {
    // Regenerate Into Disposable Output.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output_root = root.join("target/generated/rust/dokime-protocol");
    fs::create_dir_all(&output_root)?;
    let output = tempfile::tempdir_in(output_root)?;
    crate::protocol::generate(&root, output.path())?;

    // Compare Both Packages, Retaining Evidence on Failure.
    for (generated, committed) in [("dokime.v1.rs", "src/v1.rs"), ("harmos.v1.rs", "src/harmos/v1.rs")] {
        let actual = fs::read(output.path().join(generated))?;
        let expected = fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(committed))?;
        if actual != expected {
            let output = output.keep();
            panic!("generated Rust differs from {committed}; run mise run generate -- --protocol; evidence: {}", output.display());
        }
    }
    Ok(())
}
