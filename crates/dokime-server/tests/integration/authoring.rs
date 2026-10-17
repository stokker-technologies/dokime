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
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports

/* === Constants === */
const HOME_STREAM: &str = "fixture.homed.events";
const OTHER_STREAM: &str = "fixture.other.samples";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
struct HomedRow {
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime_app::StreamSpec for HomedRow {
    type Row = Self;
    const ID: &'static str = HOME_STREAM;
    const VERSION: u32 = 1;
    const WINDOW: usize = 4096;
}

impl dokime_app::telemetry::TelemetryStream for HomedRow {
    fn contract() -> dokime_app::telemetry::TelemetryContract {
        dokime_app::telemetry::TelemetryContract::declared(
            <Self as dokime_app::StreamSpec>::ID,
            "Steady Events",
        )
        .columns(<Self as dokime_app::telemetry::TelemetryRow>::columns())
        .typed::<Self>()
        .strict()
    }
}

impl dokime_app::telemetry::OwnedStream for HomedRow {}

impl dokime_app::telemetry::TelemetryRow for HomedRow {
    fn columns() -> Vec<dokime_app::telemetry::TelemetryColumn> {
        vec![dokime_app::telemetry::TelemetryColumn::declared(
            "value",
            "Value",
            dokime_app::core::ValueType::Number,
            dokime_app::telemetry::TelemetryRole::Value,
        )]
    }

    fn into_row(self) -> BTreeMap<String, dokime_app::core::Value> {
        BTreeMap::from([(
            "value".to_owned(),
            dokime_app::core::Value::Number(self.value),
        )])
    }

    fn from_row(
        values: &BTreeMap<String, dokime_app::core::Value>,
    ) -> Result<Self, dokime_app::telemetry::FieldError> {
        Ok(Self {
            value: dokime_app::telemetry::field(values, "value")?,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
#[test]
fn untyped_artifact_refused() {
    let sink = std::sync::Arc::new(dokime_app::transport::recording::RecordingSink::new());
    let artifacts = dokime_app::artifacts::Artifacts::new("authoring", sink.clone());

    // An artifact name is the one statement the file makes about its content,
    // and there is no typed override left to make that statement a lie.
    let refused = artifacts
        .write("x", b"bytes")
        .expect_err("a name that does not say what the content is is refused");
    // `dokime::Error` is not a name this crate resolves (it depends on
    // `dokime` only through `dokime_app`'s re-exports), so the `io::Error`
    // beneath the refusal is reached through `source()` instead of a match.
    let io_error = std::error::Error::source(&refused)
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("an artifact refusal keeps the io::Error as its source");
    assert_eq!(io_error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(
        refused
            .to_string()
            .contains("does not say what the content is"),
        "{refused}"
    );
    assert!(artifacts.create("evidence/trace").is_err());
    artifacts
        .write("x.bin", b"bytes")
        .expect("opaque bytes are a statement a name can make");
    assert_eq!(sink.chunks().len(), 1, "a refused name sends no piece");
}

#[tokio::test]
async fn declared_row_home() {
    let sink = std::sync::Arc::new(dokime_app::transport::recording::RecordingSink::new());
    let artifacts = dokime_app::artifacts::Artifacts::new("authoring", sink.clone());

    // A row type that declared its stream is published on that one and nowhere
    // else — the contract the host validates it against is that stream's, and
    // the call has no room left to name another. Driven through a step's own
    // grant, which is where an author meets the rule.
    let mut transport = dokime_app::transport::recording::RecordingTransport::new();
    let parameters = BTreeMap::new();
    let settings = dokime::plugin::Configuration::default();
    let environment: dokime_app::execution::Environment<'_, (), ()> =
        dokime_app::execution::Environment::new(
            &(),
            &parameters,
            &settings,
            artifacts,
            dokime_app::telemetry::Streams::new(sink.clone()),
            dokime_app::records::Deliveries::new(sink.clone()),
        );
    let step = environment.step(&mut transport);

    step.stream::<HomedRow>()
        .open()
        .push(HomedRow { value: 1.5 })
        .await;
    assert_eq!(sink.rows(HOME_STREAM).len(), 1);
    assert!(
        sink.rows(OTHER_STREAM).is_empty(),
        "a declared row type has one home and lands on that one"
    );
}
