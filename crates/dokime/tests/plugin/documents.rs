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

// Local Imports
use std::sync::atomic::{AtomicBool, Ordering};

use dokime::document::DocumentNode;
use dokime::plugin::testing::Harness;
use dokime::prelude::*;

/* === Constants === */
const SUITE: &str = "documents.checks";
const TEMPLATE: &str = "documents.checks.probe";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct Probe;

#[derive(Default)]
struct ProbeResources {
    reported: AtomicBool,
    observed: AtomicBool,
}

#[dokime::stream(
    id = "documents.checks.samples",
    window = 64,
    label = "Samples",
    description = "One measurement of whatever the bench turned out to have.\nWhat the bench answered with, once the step found it."
)]
struct SampleRow {
    #[row(unit = "V", format = fixed(2))]
    voltage_v: f64,
}

#[dokime::parameters]
struct ProbeParameters {
    #[parameter(label = "Add the same section twice", default = false)]
    repeat: bool,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),
}

/* === Traits === */

#[dokime::catalog]
enum ObserveStreams {
    SampleRow(SampleRow),
}

/* === Implementations === */
#[dokime::catalog]
enum OwnerSuites {
    Checks(Checks),
}

#[dokime::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[dokime::suite(id = SUITE, label = "Documents", templates = ChecksTemplates)]
impl Checks {}

#[dokime::template(
    id = TEMPLATE,
    label = "Probe",
    parameters = ProbeParameters)]
impl Probe {
    fn document() -> Document {
        let mut voltage = Plot::time_series("Voltage");
        voltage.id("documents.checks.voltage");
        voltage.unit("V");
        voltage.y(SampleRow::column_voltage_v());

        let mut declared = Section::new("Declared");
        declared.id("documents.checks.declared");
        declared.plot(voltage);

        let mut document = Document::new("default", "Probe");
        document.section(declared);

        document
    }

    async fn initialize(
        _: &ProbeParameters,
        _ctx: &mut InitializationContext<'_>,
    ) -> Result<(Self, ProbeResources), Error> {
        let template = Self;
        let shared = ProbeResources::default();
        Ok((template, shared))
    }

    #[dokime::step(
        label = "Report What It Found",
        description = "Says what it found, as the document a reader opens rather than as a log line a card would have to reassemble."
    )]
    async fn report(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let mut discovered = Section::new("Discovered");
        discovered.id("documents.checks.discovered");
        discovered.note("The bench answered on one stream.");
        ctx.document().section(discovered);
        ctx.document()
            .note("documents.checks.declared", "Measured once.");

        if ctx.parameters().repeat {
            let mut replaced = Section::new("Replaced");
            replaced.id("documents.checks.declared");
            ctx.document().section(replaced);
        }

        ctx.resources().reported.store(true, Ordering::Release);

        Ok(())
    }

    #[dokime::step(
        label = "Wait For The Observer",
        description = "Waits until the observer has had its turn at the shared index.\nWithout it the run could end before any tick happened, and the claim this file makes about two tasks sharing one index would be asserted by a test that never exercised it."
    )]
    async fn settle(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        while !ctx.resources().observed.load(Ordering::Acquire) {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }

        Ok(())
    }

    #[dokime::observer(interval_ms = 5, streams = ObserveStreams)]
    async fn observe(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        if !ctx.resources().reported.load(Ordering::Acquire) {
            return Ok(());
        }

        let mut observed = Section::new("From The Observer");
        observed.id("documents.checks.discovered");
        ctx.document().section(observed);
        ctx.resources().observed.store(true, Ordering::Release);
        ctx.publish(SampleRow { voltage_v: 1.0 }).await;

        Ok(())
    }

    async fn cleanup(&mut self, ctx: &mut CleanupContext<'_>) -> Result<(), Error> {
        ctx.document()
            .note("documents.checks.discovered", "The bench was released.");

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn sections(document: &dokime::document::DocumentDefinition) -> Vec<&str> {
    document
        .layout
        .nodes
        .iter()
        .filter_map(|node| match node {
            DocumentNode::Section(section) => Some(section.node_id.as_str()),
            DocumentNode::Card(_) => None,
        })
        .collect()
}

/* === Tests === */

pub(crate) async fn step_adds_declared() {
    let report = Harness::template(TEMPLATE)
        .run::<OwnerSuites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(
        sections(report.document()),
        ["documents.checks.declared", "documents.checks.discovered"],
        "an addition lands behind the declared skeleton, never in front of it"
    );
    // The note the step wrote onto the declared section sits behind the plot
    // that section was declared with, and the one the added section carries
    // travelled with it.
    let cards = report
        .document()
        .cards
        .iter()
        .map(|card| card.metadata.identifier.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        cards,
        [
            "documents.checks.voltage",
            "documents.checks.discovered.note.0",
            "documents.checks.declared.note.1",
            // Cleanup runs inside the attempt, so what it adds rides the same
            // drain the steps' additions did.
            "documents.checks.discovered.note.1",
        ]
    );
}

pub(crate) async fn observer_shares_write() {
    let report = Harness::template(TEMPLATE)
        .run::<OwnerSuites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(
        report
            .document()
            .layout
            .nodes
            .iter()
            .filter_map(|node| match node {
                DocumentNode::Section(section)
                    if section.node_id.as_str() == "documents.checks.discovered" =>
                    Some(section.metadata.label.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        ["Discovered"],
        "the step's section is the one that stands; the observer added nothing"
    );
}

pub(crate) async fn declared_section_so() {
    let report = Harness::template(TEMPLATE)
        .parameter("repeat", true)
        .run::<OwnerSuites>()
        .await
        .expect("the run completes");

    assert!(report.passed(), "{}", report.diagnostics());
    assert_eq!(
        sections(report.document()),
        ["documents.checks.declared", "documents.checks.discovered"],
        "a declared section cannot be replaced from a step"
    );
    assert_eq!(
        report
            .document()
            .layout
            .nodes
            .iter()
            .filter_map(|node| match node {
                DocumentNode::Section(section)
                    if section.node_id.as_str() == "documents.checks.declared" =>
                    Some(section.metadata.label.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        ["Declared"],
        "the declared label is the one that survives"
    );
    assert!(
        report.diagnostics().contains("documents.checks.declared"),
        "an operator cannot act on {:?}",
        report.diagnostics()
    );
}
