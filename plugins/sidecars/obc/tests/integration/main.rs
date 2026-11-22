/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// OBC Plugin Stories: The Simulated Charger and the Catalog the Host Admits

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;

// Third Party Imports
use prost::Message;
use prost_reflect::DescriptorPool;

// Local Imports
use dokime::{Aggregate, DataExpression, Declarations, Document, Suites, TestCaseTemplate, Visibility, card, data_expression};
use plugin_obc::device::{Charger, Fault, Power, Supply};
use plugin_obc::suites::benchmark;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn templates(declarations: &Declarations) -> Vec<TestCaseTemplate> {
    let section = declarations.sections.iter().find(|section| section.r#type == "dokime.model.v1.Suites").unwrap();
    Suites::decode(section.payload.as_slice()).unwrap().suites.into_iter().flat_map(|suite| suite.templates).collect()
}

fn document(templates: &[TestCaseTemplate], id: &str) -> Document {
    let template = templates.iter().find(|template| template.metadata.as_ref().unwrap().identifier == id).unwrap();
    template.documents[0].clone()
}

fn plots(document: &Document) -> Vec<(String, dokime::Plot)> {
    document
        .sections
        .iter()
        .flat_map(|section| section.cards.iter().map(move |card| (section.metadata.as_ref().unwrap().identifier.clone(), card)))
        .filter_map(|(section, value)| match &value.body {
            Some(card::Body::Plot(plot)) => Some((section, plot.clone())),
            _ => None,
        })
        .collect()
}

fn stream_fields(expression: &DataExpression, fields: &mut Vec<(String, String)>) {
    match expression.kind.as_ref() {
        Some(data_expression::Kind::Stream(reference)) => fields.push((reference.identifier.clone(), reference.field.clone())),
        Some(data_expression::Kind::Aggregate(aggregate)) => stream_fields(aggregate.input.as_ref().unwrap(), fields),
        Some(data_expression::Kind::Transform(transform)) => stream_fields(transform.input.as_ref().unwrap(), fields),
        _ => {}
    }
}

/* === Tests === */

#[test]
fn charger_inrush_settles_and_protection_latches() {
    // Energize the Default Charger: Inrush Starts Half Above the Settled Current
    let mut charger = Charger::new(Supply::default()).unwrap();
    charger.start(0.0).unwrap();
    let inrush = charger.sample(0.0);
    let settled = charger.sample(0.2);
    assert!((inrush.current_rms_a - 3.45).abs() < 1e-9);
    assert!((settled.current_rms_a - 2.3).abs() < 1e-3);
    assert_eq!(settled.dc_voltage_v, 400.0);

    // A Heavy Load Trips the Latch at Once; a Latched Charger Refuses to Restart
    charger.set_load(10.0, 0.3).unwrap();
    assert_eq!((charger.power, charger.fault), (Power::Faulted, Some(Fault::Overcurrent)));
    assert_eq!(charger.reading(0.3).current_rms_a, 0.0);
    assert!(charger.start(0.4).is_err());
}

#[test]
fn catalog_binds_every_document_to_declared_rows() {
    // Every Suite Composes Into One Admitted Catalog
    let declarations = plugin_obc::declarations().unwrap();
    let prefix = format!("{}.", plugin_obc::IDENTIFIER);
    assert!(declarations.operations.iter().all(|operation| operation.id.starts_with(&prefix)));
    assert!(declarations.streams.iter().all(|stream| stream.id.starts_with(&prefix)));
    let templates = templates(&declarations);
    let identifiers = templates.iter().map(|template| template.metadata.as_ref().unwrap().identifier.clone()).collect::<BTreeSet<_>>();
    assert_eq!(
        identifiers,
        BTreeSet::from(
            [
                "obc.steady_state.single_phase",
                "obc.steady_state.three_phase",
                "obc.transient.startup",
                "obc.transient.overcurrent",
                "obc.transient.overvoltage",
                "obc.benchmark.streams",
                "obc.benchmark.plots",
                "obc.benchmark.observers",
                "obc.benchmark.exact_values",
            ]
            .map(String::from)
        )
    );
    let public = declarations.operations.iter().filter(|operation| operation.visibility == Visibility::Public as i32);
    assert_eq!(
        public.map(|operation| operation.id.as_str()).collect::<BTreeSet<_>>(),
        BTreeSet::from(["obc.device.read", "obc.device.set_power"])
    );

    // Every Plot Field Exists in Its Stream's Frozen Row
    let pool = DescriptorPool::from_file_descriptor_set(declarations.types.clone().unwrap()).unwrap();
    for template in &templates {
        for document in &template.documents {
            for (_, plot) in plots(document) {
                let mut fields = Vec::new();
                for expression in plot.x.iter().chain(&plot.y).chain(&plot.z).chain(&plot.series_by) {
                    stream_fields(expression, &mut fields);
                }
                if let Some(grid) = &plot.grid {
                    for expression in grid.row.iter().chain(&grid.column).chain(&grid.frame) {
                        stream_fields(expression, &mut fields);
                    }
                }
                for (stream, field) in fields {
                    let declaration = declarations.streams.iter().find(|declaration| declaration.id == stream).unwrap();
                    let row = pool.get_message_by_name(&declaration.row).unwrap();
                    assert!(row.get_field_by_name(&field).is_some(), "{stream}.{field}");
                }
            }
        }
    }

    // The Start-Up Report Draws All Ten Plot Kinds and Keeps Its Signal Maps
    let report = document(&templates, benchmark::plots::TEMPLATE);
    let kinds = plots(&report).iter().map(|(_, plot)| plot.kind).collect::<BTreeSet<_>>();
    assert_eq!(kinds.len(), 10);
    assert!(plots(&report).len() > 100);

    // The Bench Report Holds One Hundred Time Series and Eleven Exact Register Tiles
    let report = document(&templates, benchmark::exact_values::TEMPLATE);
    assert!(plots(&report).is_empty());
    let tiles = report
        .sections
        .iter()
        .flat_map(|section| &section.cards)
        .find_map(|value| match &value.body {
            Some(card::Body::Summary(summary)) => Some(summary.tiles.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(tiles.len(), 11);
    for tile in tiles {
        let Some(data_expression::Kind::Aggregate(aggregate)) = tile.value.unwrap().kind else { panic!("Aggregate Tile") };
        assert_ne!(aggregate.kind, Aggregate::Unspecified as i32);
        let mut fields = Vec::new();
        stream_fields(aggregate.input.as_ref().unwrap(), &mut fields);
        assert_eq!(fields, [(benchmark::exact_values::METER.to_owned(), "energy_register".to_owned())]);
    }
}

#[test]
fn impedance_and_balanced_three_phase_power_are_consistent() {
    let supply = Supply {
        load_ohm: 80.0,
        reactance_ohm: 60.0,
        current_limit_a: 10.0,
        ..Supply::default()
    };
    let mut single = Charger::new(supply).unwrap();
    let mut three = Charger::new(Supply { phases: 3, ..supply }).unwrap();
    single.start(0.0).unwrap();
    three.start(0.0).unwrap();
    let single = single.sample(1.0);
    let three = three.sample(1.0);
    assert!((single.current_rms_a - 2.3).abs() < 1e-9);
    assert!((single.input_power_w / (single.voltage_rms_v * single.current_rms_a) - 0.8).abs() < 1e-9);
    assert!((three.input_power_w - 3.0 * single.input_power_w).abs() < 1e-9);
    assert!((three.dc_current_a - 3.0 * single.dc_current_a).abs() < 1e-9);
    assert!((three.dc_voltage_v * three.dc_current_a / three.input_power_w - 0.94).abs() < 1e-9);
}

#[test]
fn voltage_protection_disconnects_and_blocks_restart() {
    let mut charger = Charger::new(Supply::default()).unwrap();
    charger.start(0.0).unwrap();
    charger.set_voltage(260.0, 0.2).unwrap();
    assert_eq!(charger.power, Power::Running);
    charger.set_voltage(290.0, 0.3).unwrap();
    assert_eq!((charger.power, charger.fault), (Power::Faulted, Some(Fault::Overvoltage)));
    assert_eq!(charger.reading(0.3).dc_current_a, 0.0);
    assert!(charger.start(0.4).is_err());
    assert!(Charger::new(Supply { phases: 2, ..Supply::default() }).is_err());
    assert!(
        Charger::new(Supply {
            reactance_ohm: f64::NAN,
            ..Supply::default()
        })
        .is_err()
    );
}
