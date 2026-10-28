/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod dokime {
    #[sdk::message]
    #[derive(Clone, PartialEq)]
    pub struct Payload {
        pub text: String,
    }

    #[allow(dead_code)]
    #[sdk::message]
    #[derive(Clone, PartialEq)]
    pub struct Ignored {
        pub text: String,
    }

    pub fn reply(request: Payload) -> Payload {
        request
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use sdk::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
use sdk::harmos::plugin::{Attachment, Declared};
use sdk::harmos::sidecar::{ObserverAttachment, ObserverContext, PublishError, Publisher};

// Local Imports

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(120),
    attempt_timeout: std::time::Duration::from_secs(120),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */
type Observing = ObserverAttachment<Resources, Publisher<Renamed>>;

/* === Structs === */
struct Renamed {
    resources: Arc<Resources>,
    observers: Option<Arc<Observing>>,
}

struct Resources;

#[sdk::row]
#[derive(Clone)]
pub struct Row {
    value: i64,
}

#[sdk::stream(id = "renamed.observations", window = 64, row = Row, label = "Observations")]
pub struct ObservationStream;

#[sdk::stream(id = "renamed.direct", window = 8, row = Row)]
pub struct DirectStream;

#[sdk::record(id = "renamed.observed", label = "Observed")]
pub struct Observed {
    value: i64,
}

#[derive(Default)]
struct Observations;

#[sdk::section(label = "Chamber", description = "One settings section, written the way an author writes one.")]
struct Chamber {
    #[setting(label = "Port", default = "/dev/ttyUSB0")]
    port: String,
}

#[sdk::settings]
struct RenamedSettings {
    chamber: Chamber,
}

#[sdk::operation(id = "renamed.get_echo", description = "Echoes.", policy = POLICY, response = self::dokime::Payload, resources = [Resources])]
#[derive(PartialEq)]
struct GetEcho {
    text: String,
}

#[sdk::operation(id = "renamed.get_status",
    description = "Reports.",
    policy = POLICY,
    response = sdk::domains::StatusResponse)]
#[derive(PartialEq)]
struct GetStatus {}

#[sdk::operation(id = "renamed.get_spare_echo",
    description = "Echoes too.",
    policy = POLICY,
    response = self::dokime::Ignored)]
#[derive(PartialEq)]
struct GetSpareEcho {
    text: String,
}

/* === Enums === */
#[sdk::catalog]
enum Implemented {
    GetEcho(GetEcho),
    GetStatus(GetStatus),
    GetSpareEcho(GetSpareEcho),
}

#[sdk::catalog]
enum Operations {
    #[nested]
    Implemented(Implemented),
}

#[sdk::catalog]
enum Observers {
    Observations(Observations),
}

#[sdk::catalog]
pub enum Streams {
    Observation(ObservationStream),
    Direct(DirectStream),
}

#[sdk::catalog]
pub enum Records {
    Observed(Observed),
}

/* === Traits === */

/* === Implementations === */
impl sdk::Operation for GetEcho {
    type Error = sdk::Refusal;

    async fn run(self, ctx: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<self::dokime::Payload>, sdk::Refusal> {
        let _: Arc<Resources> = ctx.resource::<Resources>();
        Ok(self::dokime::reply(self::dokime::Payload { text: self.text }).into())
    }
}

impl sdk::Operation for GetStatus {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<sdk::domains::StatusResponse>, sdk::Refusal> {
        Ok(sdk::domains::StatusResponse::ready()
            .field("Echo", self::dokime::reply(self::dokime::Payload { text: "author syntax survives".to_owned() }).text)
            .into())
    }
}

impl sdk::Operation for GetSpareEcho {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<self::dokime::Ignored>, sdk::Refusal> {
        Ok(self::dokime::Ignored { text: self.text }.into())
    }
}

#[sdk::observer(interval_ms = 10, streams(ObservationStream), activation = "demand")]
impl Observations {
    async fn observe(&mut self, ctx: &ObserverContext<'_, Resources, Publisher<Renamed>>) -> Result<(), PublishError> {
        let text = self::dokime::reply(self::dokime::Payload { text: "one".to_owned() });

        ctx.publish_on::<ObservationStream>(&Row { value: text.text.len() as i64 }).await
    }
}

impl sdk::harmos::builtins::lifecycle::Policy for Renamed {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "renamed",
    provides = Operations,
    requires(),
    contributions(
        sdk::Records<Records, ()>,
        sdk::Evidence<Streams>,
        sdk::Settings<RenamedSettings>
    ),
    observers = Observers,
    restart = "never",
)]
impl sdk::Plugin for Renamed {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    type Resources = Resources;

    fn prepare(config: sdk::plugin::Configuration) -> Result<Self, sdk::Refusal> {
        let settings = sdk::plugin::refusal(config.settings::<RenamedSettings>())?;
        let _ = settings.chamber.port;

        Ok(Self { resources: Arc::new(Resources), observers: None })
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<Observers>(self.resources(), <Self as Declared>::STREAMS)?);

        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable) -> Result<(), ServingError> {
        let mut dependencies = sdk::harmos::Dependencies::new();
        dependencies.insert(self.resources()).map_err(|_| ServingError::Catalog)?;
        table.register(Implemented::registration(dependencies))?;
        self.observers.as_ref().ok_or(ServingError::Catalog)?.bind(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<sdk::Supervisor>>> {
        self.observers.as_ref().map(|observers| Arc::clone(observers) as Arc<dyn Attachment<sdk::Supervisor>>)
    }

    async fn stop(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), sdk::Refusal> {
        self.observers.take();

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[cfg(not(test))]
fn main() {
    std::hint::black_box(<Renamed as Declared>::markers()).unwrap();
}

#[cfg(test)]
fn operation_ids(nodes: &[sdk::catalog::Node<sdk::harmos::operations::WireDeclaration>]) -> Vec<&'static str> {
    nodes
        .iter()
        .flat_map(|node| match node {
            sdk::catalog::Node::Leaf(declaration) => vec![declaration.operation.id],
            sdk::catalog::Node::Nested(nested) => operation_ids(nested),
        })
        .collect()
}

/* === Tests === */

#[cfg(test)]
#[test]
fn alias_compiles_operations() {
    // Renamed Facade Uses One Shared Codec For Every Domain Shape.
    let record = Observed { value: 19 };
    let encoded = sdk::Message::encode(&record);
    let recovered = <Observed as sdk::Message>::decode(&encoded).unwrap();
    assert_eq!(recovered.value, 19);
    assert_eq!(<Observed as sdk::records::DeclaredRecord>::RECORD_TYPE, "renamed.observed");
    let row = Row { value: 23 };
    let encoded = sdk::Message::encode(&row);
    assert_eq!(<Row as sdk::Message>::decode(&encoded).unwrap().value, 23);
    assert_eq!(Row::column_value().field(), "value");
    assert_eq!(<Renamed as Declared>::ID, "renamed");
    let mut served = operation_ids(<Renamed as Declared>::PROVIDES);
    served.sort_unstable();
    assert_eq!(served, ["renamed.get_echo", "renamed.get_spare_echo", "renamed.get_status"]);
    let streams = <Renamed as Declared>::STREAMS.iter().map(|stream| stream.id).collect::<Vec<_>>();
    assert_eq!(streams, ["renamed.observations", "renamed.direct"]);
    assert_eq!(<Renamed as Declared>::OBSERVERS.len(), 1);
    assert_eq!(sdk::harmos::operations::observers::contract().unwrap().operation().id.as_str(), "harmos_observe");
    let namespaces = <Renamed as Declared>::CONTRIBUTIONS.iter().map(|section| section.namespace.as_ref()).collect::<Vec<_>>();
    assert!(namespaces.contains(&"dokime.settings"), "{namespaces:?}");
}
