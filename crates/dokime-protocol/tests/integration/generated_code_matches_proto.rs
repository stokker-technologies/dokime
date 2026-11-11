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
        ("GROUPED_BAR", 3, "BAR"),
        ("STACKED_BAR", 4, "BAR"),
        ("DONUT", 6, "PIE"),
        ("PARETO", 7, "BAR"),
        ("XY_INTERVAL", 9, "XY"),
        ("HEATMAP", 11, "CONTOUR"),
        ("XY_VALUE_HEATMAP", 12, "CONTOUR"),
        ("WIFI_CHANNEL_OCCUPANCY", 13, "GRID_CHART"),
        ("BOX_SUMMARY", 16, "BOX_PLOT"),
        ("AGGREGATE_TABLE", 17, "Summary"),
    ] {
        for result in [PlotKind::parse_name(name), PlotKind::from_declared_number(number)] {
            let error = result.unwrap_err();
            let identifiers = error.split(|character: char| !character.is_ascii_alphanumeric() && character != '_').collect::<Vec<_>>();
            assert!(identifiers.contains(&name));
            assert!(identifiers.contains(&replacement));
            let option = match name {
                "GROUPED_BAR" => Some("arrangement=grouped"),
                "STACKED_BAR" => Some("arrangement=stacked"),
                "DONUT" => Some("hole=0.5"),
                "PARETO" => Some("pareto=true"),
                "XY_INTERVAL" => Some("interval=sigma"),
                _ => None,
            };
            if let Some(option) = option {
                assert!(error.contains(option));
            }
        }
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
    for (generated, committed) in [
        ("python/documents.pb", "sdk/python/python/dokime/v1/documents.pb"),
        ("csharp/Documents.cs", "sdk/csharp/Dokime/V1/Documents.cs"),
        ("csharp/Values.cs", "sdk/csharp/Dokime/V1/Values.cs"),
        ("cpp/dokime/v1/documents.pb.h", "sdk/cpp/include/dokime/v1/documents.pb.h"),
        ("cpp/dokime/v1/values.pb.h", "sdk/cpp/include/dokime/v1/values.pb.h"),
        ("cpp/dokime/v1/documents.pb.cc", "sdk/cpp/src/dokime/v1/documents.pb.cc"),
        ("cpp/dokime/v1/values.pb.cc", "sdk/cpp/src/dokime/v1/values.pb.cc"),
    ] {
        if fs::read(output.path().join(generated))? != fs::read(root.join(committed))? {
            let output = output.keep();
            panic!("generated SDK binding differs from {committed}; run mise run generate -- --protocol; evidence: {}", output.display());
        }
    }
    Ok(())
}

#[test]
fn typed_plot_builders_produce_canonical_document_messages() {
    use dokime_protocol::builders::Plot;
    use dokime_protocol::v1::*;
    use prost::Message;
    let scan = FieldReference::new("scan");
    let surface = Plot::contour(scan.field("temperature").scale(1.0))
        .x(scan.field("position_x"))
        .y(scan.field("position_y"))
        .label("Temperature Map")
        .unit("°C")
        .reduce(Reduction::Mean)
        .render(ContourRender::Bands);
    assert_eq!(surface.kind, PlotKind::Contour as i32);
    assert_eq!(surface.y.len(), 1);
    let data_expression::Kind::Transform(transform) = surface.z.as_ref().unwrap().kind.as_ref().unwrap() else { panic!("typed scale") };
    assert_eq!(transform.kind, Some(transform_expression::Kind::Scale(1.0)));
    let grid = Plot::grid_chart(scan.field("value"))
        .row(scan.field("row_key"))
        .column(scan.field("column_key"))
        .frame(scan.field("timestamp"))
        .rows([GridAxisEntry {
            key: "sensor".into(),
            label: "Sensor".into(),
            order: 0,
            group: None,
        }])
        .columns([GridAxisEntry {
            key: "site".into(),
            label: "Site".into(),
            order: 0,
            group: None,
        }])
        .reduce(Reduction::Max)
        .label("Sensor Grid");
    assert!(grid.grid.as_ref().unwrap().frame.is_some());
    let document = Document::new("Scan").section(Section::new("Map").plot(surface).plot(grid));
    assert_eq!(Document::decode(document.encode_to_vec().as_slice()).unwrap(), document);
    let variants = [
        Plot::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Grouped),
        Plot::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Stacked),
        Plot::bar(scan.field("value")).x(scan.field("category")).pareto(true),
        Plot::pie(scan.field("value")).x(scan.field("category")).hole(0.5),
        Plot::xy(scan.field("value")).x(scan.field("position")).interval(XyInterval::Sigma),
    ];
    assert_eq!(variants[0].arrangement, Some(BarArrangement::Grouped as i32));
    assert_eq!(variants[1].arrangement, Some(BarArrangement::Stacked as i32));
    assert!(variants[2].pareto);
    assert_eq!(variants[3].hole, Some(0.5));
    assert_eq!(variants[4].interval, Some(XyInterval::Sigma as i32));
    assert_eq!(Plot::contour(scan.field("value")).id("stable").label("Temperature Map").metadata.as_ref().unwrap().identifier, "stable");
}
