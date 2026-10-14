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
use std::sync::Arc;
// Third Party Imports
use dokime::harmos::operations::{ServingError, WireTable};
use dokime::harmos::plugin::contract::Declared;
// Local Imports
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::row]
#[derive(Clone, PartialEq)]
struct DirectReading {
    value: u32,
}

#[dokime::stream(id = "sdk-streams.inlet", window = 8, row = DirectReading)]
struct Inlet;

#[dokime::stream(id = "sdk-streams.outlet", window = 8, row = DirectReading)]
struct Outlet;

struct DirectStreams {
    host: Option<Supervisor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime::harmos::operations::lifecycle::Policy for DirectStreams {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy =
        super::native::POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime::plugin(
    id = "sdk-streams",
    provides = (),
    requires(dokime::harmos::operations::host::Publish),
    streams(Inlet, Outlet),
    restart = "never", description = "Publishes through Harmos streams.")]
impl dokime::Plugin for DirectStreams {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(_: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        Ok(Self { host: None })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        let publisher = host.publisher::<Self>();
        dokime::plugin::refusal(
            publisher
                .publish_on::<Inlet>(&DirectReading { value: 7 })
                .await,
        )?;
        dokime::plugin::refusal(
            publisher
                .publish_on::<Outlet>(&DirectReading { value: 8 })
                .await,
        )?;
        self.host = Some(host.clone());
        Ok(())
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, _: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        let host = self
            .host
            .take()
            .expect("initialization kept its supervisor");
        assert!(
            host.publisher::<Self>()
                .publish_on::<Inlet>(&DirectReading { value: 9 })
                .await
                .is_err(),
            "a retired plugin publishes nothing"
        );
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) async fn stream_alias_publication() {
    use dokime::telemetry::TelemetryStream;

    assert_eq!(
        <DirectStreams as Declared>::STREAMS
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        ["sdk-streams.inlet", "sdk-streams.outlet"],
        "plugin streams are declared without any observers",
    );
    assert!(Inlet::contract().interval.is_none());
    assert!(Outlet::contract().interval.is_none());

    let mut native = super::native::Native::initialize(
        super::native::attach::<DirectStreams>(),
        <DirectStreams as Declared>::VERSION,
        serde_json::to_vec(&dokime::plugin::Configuration::default()).unwrap(),
        |_, _| {},
        vec![],
    )
    .await;
    let inlet = native
        .runtime
        .streams
        .publication("sdk-streams.inlet", 1)
        .unwrap();
    let outlet = native
        .runtime
        .streams
        .publication("sdk-streams.outlet", 1)
        .unwrap();

    // Original Read Handles Preserve Lifecycle Evidence After Producer Retirement.
    native.retire().await;
    for (name, expected, stream) in [("inlet", vec![7], inlet), ("outlet", vec![8], outlet)] {
        assert_eq!(stream.definition().id(), format!("sdk-streams.{name}"));
        assert_eq!(
            stream.definition().schema().unwrap(),
            &harmos::schema::graph::Schema::from_declaration(
                <DirectReading as harmos::schema::Described>::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        );
        let values: Vec<_> = stream
            .query(..)
            .await
            .unwrap()
            .map(|row| {
                let decoded: DirectReading = harmos::Message::decode(&row.value.payload).unwrap();
                decoded.value
            })
            .collect();
        assert_eq!(values, expected);
    }
    native.shutdown().await;
}
