/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use dokime_runtime::harmos::plugin::Declared;

use dokime_runtime::{Refusal, Supervisor};

#[dokime_runtime::row]
#[derive(Clone, PartialEq)]
struct DirectReading {
    value: u32,
}

#[dokime_runtime::stream(id = "sdk_streams.inlet", window = 8, row = DirectReading)]
struct Inlet;

#[dokime_runtime::stream(id = "sdk_streams.outlet", window = 8, row = DirectReading)]
struct Outlet;

struct DirectStreams {
    host: Option<Supervisor>,
}

#[dokime_runtime::catalog]
enum Streams {
    Inlet(Inlet),
    Outlet(Outlet),
}

impl dokime_runtime::harmos::builtins::lifecycle::Policy for DirectStreams {
    const INITIALIZATION_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: dokime_runtime::harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(
    id = "sdk_streams",
    requires(dokime_runtime::harmos::builtins::streams::Publish),
    contributions(dokime_runtime::Evidence<Streams>),
    restart = "never", description = "Publishes through Harmos streams.")]
impl dokime_runtime::Plugin for DirectStreams {
    const CATALOGS: &'static [dokime_runtime::harmos::Catalog] = &[];
}

impl dokime_runtime::harmos::Lifecycle for DirectStreams {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    async fn prepare(_: dokime_runtime::plugin::Configuration, setup: &mut dokime_runtime::harmos::Setup<Self>) -> Result<Self, Refusal> {
        let owner = { Self { host: None } };
        setup.resource::<()>(owner.resources());
        Ok(owner)
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        let publisher = host.publisher::<Self>();
        dokime_runtime::plugin::refusal(publisher.publish_on::<Inlet>(&DirectReading { value: 7 }).await)?;
        dokime_runtime::plugin::refusal(publisher.publish_on::<Outlet>(&DirectReading { value: 8 }).await)?;
        self.host = Some(host.clone());
        Ok(())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        let host = self.host.take().expect("initialization kept its supervisor");
        assert!(
            host.publisher::<Self>().publish_on::<Inlet>(&DirectReading { value: 9 }).await.is_err(),
            "a retired plugin publishes nothing"
        );
        Ok(())
    }
}

impl DirectStreams {
    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }
}

#[tokio::test]
async fn stream_alias_publication() {
    use dokime_runtime::telemetry::TelemetryStream;

    assert_eq!(
        <DirectStreams as Declared>::STREAMS.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        ["sdk_streams.inlet", "sdk_streams.outlet"],
        "plugin streams are declared without any observers",
    );
    assert!(Inlet::contract().interval.is_none());
    assert!(Outlet::contract().interval.is_none());

    let mut native = super::native::Native::initialize(
        super::native::attach::<DirectStreams>(),
        <DirectStreams as Declared>::VERSION,
        serde_json::to_vec(&dokime_runtime::plugin::Configuration::default()).unwrap(),
        |_, _| {},
        vec![],
    )
    .await;
    let inlet = native.runtime.streams.publication("sdk_streams.inlet", 1).unwrap();
    let outlet = native.runtime.streams.publication("sdk_streams.outlet", 1).unwrap();

    // Original Read Handles Preserve Lifecycle Evidence After Producer Retirement.
    native.retire().await;
    for (name, expected, stream) in [("inlet", vec![7], inlet), ("outlet", vec![8], outlet)] {
        assert_eq!(stream.definition().id(), format!("sdk_streams.{name}"));
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
