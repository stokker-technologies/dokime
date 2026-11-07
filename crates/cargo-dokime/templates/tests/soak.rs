/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Mutex};

// Third Party Imports
use dokime::host::ToolConnectionAcquired;
use dokime::plugin::testing::Harness;

// Local Imports
use {{crate}}::operations::chamber::{ChamberReading, ChamberReleased, HeaterIdentity, TargetSet};
use {{crate}}::operations::soak::EncodedSummary;
use {{crate}}::operations::{EncodeSummary, OpenHeater, ReadChamber, ReleaseChamber, SetTarget};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn bench() -> Harness {
    let temperature = Arc::new(Mutex::new(22.0));
    let measured = Arc::clone(&temperature);
    Harness::template("{{crate}}.soak.hold")
        .answering::<OpenHeater, _>(|operation| HeaterIdentity {
            serial_number: format!("SIM-{}", operation.port),
            firmware_version: "sim-1.0".to_owned(),
        })
        .answering::<SetTarget, _>(move |operation| {
            *temperature.lock().unwrap() = operation.temperature_c;
            TargetSet {}
        })
        .answering::<ReadChamber, _>(move |_| ChamberReading {
            temperature_c: *measured.lock().unwrap(),
            sensor: "simulated sensor".to_owned(),
        })
        .answering::<ReleaseChamber, _>(|_| ChamberReleased {})
        .answering::<EncodeSummary, _>(|operation| EncodedSummary {
            json: serde_json::to_vec_pretty(&operation.summary).expect("summary data serializes"),
        })
}

#[tokio::test]
async fn soak_holds_band() {
    let report = bench()
        .parameter("hold_samples", 3)
        .parameter("attended", true)
        // The record the step waits for, filed the way the host would file it.
        // The type is the host's own declaration, so the test names no id.
        .publish(ToolConnectionAcquired {
            resource_id: "sim-chamber-1".into(),
            resource_kind: "chamber".into(),
            connection_transport: None,
            connection_endpoint: None,
            status_message: None,
        })
        // Without this the harness would answer the checkpoint with the form's
        // own defaults, which is what an unattended run wants; a test that cares
        // what the operator said says so.
        .answer("{{crate}}.soak.checkpoint", "submit", [("note", "chamber steady")])
        .run::<{{crate}}::suites::Suites>()
        .await
        .expect("the soak runs");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(report.traceability("dut").row("firmware_version").text(), "sim-1.0");
    assert_eq!(report.traceability("bench").row("target_c").number(), 60.0);
    // The stream is the row type that declared it, so the test names no id and
    // reads the fields the declaration named.
    let samples = report.samples::<{{crate}}::suites::soak::Row>();
    assert!(samples.len() >= 3);

    let log = String::from_utf8(report.artifact("heater.log").bytes.clone()).expect("the chamber log is text");
    assert_eq!(log.lines().count(), 3, "one line per held sample: {log}");

    // Read back through the same declaration the step published through, so the
    // assertion is about a field rather than about a detail map.
    let held = report.records::<{{crate}}::suites::soak::HoldReached>();
    let [held] = held.as_slice() else {
        panic!("the hold records what it measured, once: {:?}", report.record_types().collect::<Vec<_>>());
    };
    assert_eq!(held.unit.as_deref(), Some("sim-chamber-1"), "the awaited record named the unit this run tested");
    assert_eq!(held.samples, 3);
}

#[tokio::test]
async fn unavailable_target_operation_run() {
    // A refused operation ends the step while cleanup can still release the bench.
    let released = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = Arc::clone(&released);
    let report = Harness::template("{{crate}}.soak.hold")
        .answering::<OpenHeater, _>(|_| HeaterIdentity {
            serial_number: "SIM-1".into(),
            firmware_version: "sim-1.0".into(),
        })
        .answering::<ReadChamber, _>(|_| ChamberReading {
            temperature_c: 22.0,
            sensor: "simulated sensor".into(),
        })
        .answering::<ReleaseChamber, _>(move |_| {
            released.store(true, std::sync::atomic::Ordering::Release);
            ChamberReleased {}
        })
        .run::<{{crate}}::suites::Suites>()
        .await
        .expect("the run reaches a terminal outcome");

    assert!(!report.passed());
    assert!(report.diagnostics().contains("step `reach`"));
    assert!(observed.load(std::sync::atomic::Ordering::Acquire), "cleanup releases the chamber after a refused call");
}

/* === Tests === */

#[test]
fn soak_document_wraps() {
    use dokime::definition::DeclaredTemplate;
    use dokime::document::{DocumentCardBody, DocumentNode};

    let declaration = <{{crate}}::suites::soak::Template as DeclaredTemplate>::declaration();
    let document = declaration.document.as_ref().expect("the soak template authors its document");

    assert_eq!(document.document_id.to_string(), "default");
    assert_eq!(
        document
            .layout
            .nodes
            .iter()
            .filter_map(|node| match node {
                DocumentNode::Section(section) => Some(section.node_id.as_str()),
                DocumentNode::Card(_) => None,
            })
            .collect::<Vec<_>>(),
        ["{{crate}}.soak.signals"]
    );

    let DocumentCardBody::Plot { plot } = &document.cards[0].body else {
        panic!("the soak section holds one plot");
    };
    assert_eq!(plot.unit.as_deref(), Some("°C"));
    assert_eq!(
        plot.marker_record_types,
        ["{{crate}}.soak.held"],
        "the hold marks the axis with the record type it publishes, named by type"
    );
    // The band is the case's own parameters rather than a number the author
    // fixed, so the plot draws the band that case was held to.
    assert!(plot.reference_bands[0].center.is_some());
    assert!(plot.reference_bands[0].tolerance.is_some());
}
