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
use sdk::harmos::operations::{
    OperationPolicy, RestartPolicy, RetryPolicy, ServingError, WireTable,
};
use sdk::harmos::plugin::{Attachment, Declared};
use sdk::harmos::sidecar::{ObserverAttachment, ObserverContext, PublishError, Publisher};

// Local Imports

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
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

#[sdk::section(
    label = "Chamber",
    description = "One settings section, written the way an author writes one."
)]
struct Chamber {
    #[setting(label = "Port", default = "/dev/ttyUSB0")]
    port: String,
}

#[sdk::settings]
struct RenamedSettings {
    chamber: Chamber,
}

struct GetEcho;

struct GetStatus;

struct GetSpareEcho;

struct ObserverControl;

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
    Observe(ObserverControl),
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
#[sdk::operation(id = "renamed.get_echo", version = 1, description = "Echoes.", policy = POLICY)]
impl GetEcho {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        request: &self::dokime_runtime::Payload,
    ) -> Result<self::dokime_runtime::Payload, sdk::Refusal> {
        Ok(self::dokime_runtime::reply(request.clone()))
    }
}

#[sdk::operation(id = "renamed.get_status", version = 1, description = "Reports.", policy = POLICY)]
impl GetStatus {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        _: &(),
    ) -> Result<sdk::domains::StatusResponse, sdk::Refusal> {
        Ok(sdk::domains::StatusResponse::ready().field(
            "Echo",
            self::dokime_runtime::reply(self::dokime_runtime::Payload {
                text: "author syntax survives".to_owned(),
            })
            .text,
        ))
    }
}

#[sdk::operation(id = "renamed.get_spare_echo", version = 1, description = "Echoes too.", policy = POLICY)]
impl GetSpareEcho {
    pub async fn initialize(_: Arc<Resources>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(
        &mut self,
        _: &sdk::Supervisor,
        request: &self::dokime_runtime::Ignored,
    ) -> Result<self::dokime_runtime::Ignored, sdk::Refusal> {
        Ok(request.clone())
    }
}

impl sdk::harmos::operations::OperationDeclaration for ObserverControl {
    type Request = sdk::harmos::operations::observers::Control;
    type Response = ();
    const ID: &'static str = "renamed.observers.observe";
    const VISIBILITY: sdk::harmos::operations::Visibility =
        sdk::harmos::operations::Visibility::Host;
    const POLICY: OperationPolicy = sdk::harmos::operations::observers::POLICY;
}

impl sdk::harmos::operations::WireOperation for ObserverControl {
    const VERSION: u32 = 1;
}

#[sdk::observer(interval_ms = 10, streams(ObservationStream), activation = "demand")]
impl Observations {
    async fn observe(
        &mut self,
        ctx: &ObserverContext<'_, Resources, Publisher<Renamed>>,
    ) -> Result<(), PublishError> {
        let text = self::dokime_runtime::reply(self::dokime_runtime::Payload {
            text: "one".to_owned(),
        });

        ctx.publish_on::<ObservationStream>(&Row {
            value: text.text.len() as i64,
        })
        .await
    }
}

impl sdk::harmos::operations::lifecycle::Policy for Renamed {
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
    observer_control = ObserverControl,
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

        Ok(Self {
            resources: Arc::new(Resources),
            observers: None,
        })
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        self.observers = Some(ObserverAttachment::from_catalog::<Observers>(
            self.resources(),
            <Self as Declared>::STREAMS,
        )?);

        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<sdk::Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Implemented, _>(self.resources())?;
        self.observers
            .as_ref()
            .ok_or(ServingError::Catalog)?
            .bind::<ObserverControl>(table)
    }

    fn attachment(&self) -> Option<Arc<dyn Attachment<sdk::Supervisor>>> {
        self.observers
            .as_ref()
            .map(|observers| Arc::clone(observers) as Arc<dyn Attachment<sdk::Supervisor>>)
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
    let _ = <Renamed as Declared>::markers();
}

#[cfg(test)]
fn operation_ids(
    nodes: &[sdk::catalog::Node<sdk::harmos::operations::WireDeclaration>],
) -> Vec<&'static str> {
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
    let encoded = sdk::Message::encode_to_vec(&record);
    let recovered = <Observed as sdk::Message>::decode(&encoded).unwrap();
    assert_eq!(recovered.value, 19);
    assert_eq!(
        <Observed as sdk::records::DeclaredRecord>::RECORD_TYPE,
        "renamed.observed"
    );
    let row = Row { value: 23 };
    let encoded = sdk::Message::encode_to_vec(&row);
    assert_eq!(<Row as sdk::Message>::decode(&encoded).unwrap().value, 23);
    assert_eq!(Row::column_value().field(), "value");
    assert_eq!(<Renamed as Declared>::ID, "renamed");
    let mut served = operation_ids(<Renamed as Declared>::PROVIDES);
    served.sort_unstable();
    assert_eq!(
        served,
        [
            "renamed.get_echo",
            "renamed.get_spare_echo",
            "renamed.get_status",
            "renamed.observers.observe",
        ]
    );
    let streams = <Renamed as Declared>::STREAMS
        .iter()
        .map(|stream| stream.id)
        .collect::<Vec<_>>();
    assert_eq!(streams, ["renamed.observations", "renamed.direct"]);
    assert_eq!(<Renamed as Declared>::OBSERVERS.len(), 1);
    let namespaces = <Renamed as Declared>::CONTRIBUTIONS
        .iter()
        .map(|section| section.namespace.as_ref())
        .collect::<Vec<_>>();
    assert!(namespaces.contains(&"dokime.settings"), "{namespaces:?}");
}
