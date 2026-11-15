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
use prost::Message as _;
use prost_reflect::DescriptorPool;

// Local Imports
use dokime::{Aggregate, DataExpression, Declarations, Document, PlotKind, Suites, TestCaseTemplate, Visibility, card, data_expression};
use plugin_obc::device::{Charger, Fault, Power, Supply};
use plugin_obc::suites::{bench, startup};

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
fn charger_inrush_settles_and_protection_latches_until_cleared() {
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
    charger.clear();
    charger.set_load(100.0, 0.4).unwrap();
    charger.start(0.4).unwrap();
    assert_eq!(charger.power, Power::Running);

    // A Power Loss Removes the Supply Voltage Until It Is Cleared
    charger.inject(Fault::PowerLoss);
    assert_eq!(charger.reading(0.5).voltage_rms_v, 0.0);
    assert!(charger.restart_blocked());
}

#[test]
fn catalog_binds_every_document_to_declared_rows() {
    // Every Suite Composes Into One Admitted Catalog
    let declarations = plugin_obc::declarations().unwrap();
    let templates = templates(&declarations);
    let identifiers = templates.iter().map(|template| template.metadata.as_ref().unwrap().identifier.clone()).collect::<BTreeSet<_>>();
    assert_eq!(
        identifiers,
        BTreeSet::from(
            [
                "obc.checks.pulse",
                "obc.charger.startup.sequence",
                "obc.charger.steady.measurement",
                "obc.charger.protection.overcurrent",
                "obc.charger.recovery.power",
                "obc.bench.module_array"
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
    let report = document(&templates, startup::TEMPLATE);
    let kinds = plots(&report).iter().map(|(_, plot)| plot.kind).collect::<BTreeSet<_>>();
    assert_eq!(kinds.len(), 10);
    let maps = plots(&report)
        .into_iter()
        .filter(|(section, _)| section == "obc.charger.startup.sequence.maps")
        .map(|(_, plot)| (plot.kind, plot.metadata.unwrap().label))
        .collect::<BTreeSet<_>>();
    assert!(maps.contains(&(PlotKind::Trajectory as i32, "Electrical Trajectory".into())));
    assert!(maps.contains(&(PlotKind::GridChart as i32, "Channel Survey".into())));
    assert!(maps.iter().any(|(kind, _)| *kind == PlotKind::Contour as i32));

    // The Bench Report Holds One Hundred Time Series and Eleven Exact Register Tiles
    let report = document(&templates, bench::TEMPLATE);
    let series = plots(&report);
    assert_eq!(series.len(), bench::MODULES * bench::CHANNELS.len());
    assert!(series.iter().all(|(_, plot)| plot.kind == PlotKind::TimeSeries as i32));
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
        assert_eq!(fields, [(bench::METER.to_owned(), "energy_register".to_owned())]);
    }
}
