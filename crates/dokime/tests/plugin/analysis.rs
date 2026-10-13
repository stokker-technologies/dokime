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
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

// Third Party Imports
use harmos::plugin::contract::Contribution;

// Local Imports
use dokime::analysis::{
    Analysis, Axis, Definition, Error, Handler, Limits, Operation, Output, Request, Response,
    ResultData, Samples, Values,
};
use dokime::core::Value;

/* === Constants === */

static RELEASE: Mutex<Option<mpsc::Receiver<()>>> = Mutex::new(None);
static STARTED: tokio::sync::Notify = tokio::sync::Notify::const_new();
static CALLS: AtomicUsize = AtomicUsize::new(0);
static FINISHED: AtomicUsize = AtomicUsize::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Host;
struct Scale;
struct Held;
struct Scaled {
    axis: Vec<f64>,
    values: Vec<f64>,
}

#[dokime::parameters]
struct Parameters {
    #[parameter(label = "Factor", default = 2.0, min = 0.0, max = 10.0)]
    factor: f64,
}

/* === Enums === */
#[dokime::catalog]
enum Analyses {
    Scale(Operation<Scale>),
    Held(Operation<Held>),
}

#[dokime::catalog]
enum Handlers {
    Scale(Handler<Scale>),
    Held(Handler<Held>),
}

#[harmos::lanes]
enum Lanes {
    #[default]
    Open,
}

/* === Traits === */

/* === Implementations === */
impl ResultData for Scaled {
    fn outputs() -> Vec<Output> {
        vec![Output {
            key: "scaled".into(),
            label: "Scaled Signal".into(),
            unit: Some("V".into()),
            axis: Some(Axis {
                key: "time".into(),
                unit: Some("s".into()),
            }),
        }]
    }
    fn into_response(self) -> Response {
        Response {
            outputs: vec![Values {
                key: "scaled".into(),
                unit: "V".into(),
                axis_unit: "s".into(),
                axis: self.axis,
                values: self.values,
            }],
        }
    }
}

impl Analysis for Scale {
    type Parameters = Parameters;
    type Output = Scaled;
    type Error = std::convert::Infallible;
    const ID: &'static str = "analysis-fixture.scale";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Scale Signal";
    const DESCRIPTION: &'static str = "Multiply Original Samples By A Typed Factor";
    const TIMEOUT: Duration = Duration::from_secs(5);
    const LIMITS: Limits = Limits {
        max_samples: 64,
        max_output_values: 64,
    };

    fn evaluate(samples: &Samples, parameters: &Parameters) -> Result<Scaled, Self::Error> {
        Ok(Scaled {
            axis: samples.axis.clone(),
            values: samples
                .values
                .iter()
                .map(|value| value * parameters.factor)
                .collect(),
        })
    }
}

impl Analysis for Held {
    type Parameters = ();
    type Output = Scaled;
    type Error = std::sync::mpsc::RecvTimeoutError;
    const ID: &'static str = "analysis-fixture.held";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Held Signal";
    const DESCRIPTION: &'static str = "Retained Worker Fixture";
    const TIMEOUT: Duration = Duration::from_secs(5);
    const LIMITS: Limits = Scale::LIMITS;

    fn evaluate(samples: &Samples, _: &()) -> Result<Scaled, Self::Error> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        let release = RELEASE.lock().unwrap().take().unwrap();
        STARTED.notify_one();
        release.recv_timeout(Duration::from_secs(5))?;
        FINISHED.fetch_add(1, Ordering::SeqCst);
        Ok(Scaled {
            axis: samples.axis.clone(),
            values: samples.values.clone(),
        })
    }
}

impl harmos::Application for Host {
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = ();
    type Streams = ();
    type Resources = ();
}

/* === Public Functions === */

/* === Private Functions === */
fn request(parameters: BTreeMap<String, Value>) -> Request {
    Request {
        samples: Some(Samples {
            axis: vec![0.0, 0.25, 0.5],
            values: vec![2.0, -1.0, 3.0],
            axis_unit: "s".into(),
            unit: "V".into(),
        }),
        parameters: serde_json::to_vec(&parameters).unwrap(),
    }
}

async fn serve() -> harmos::Runtime<Host> {
    let runtime = harmos::Runtime::<Host>::builder((), ())
        .operation_settings(harmos::operations::Settings {
            preparation_bytes: Some(NonZeroUsize::new(4096).unwrap()),
            ..Default::default()
        })
        .register(Lanes::catalog())
        .runtime()
        .await
        .unwrap();
    runtime
        .operations
        .register_wire_operations::<Handlers, ()>("analysis-fixture", Lanes::Open, Arc::new(()))
        .unwrap();
    runtime
}

async fn call<A: Analysis>(
    operations: &harmos::operations::OperationManager,
    request: Request,
) -> harmos::Result<Response> {
    let payload = operations
        .call_encoded(A::ID, harmos::Message::encode_to_vec(&request))
        .await?;
    <Response as harmos::Message>::decode(&payload).map_err(|source| {
        harmos::Error::backend(
            "decode analysis answer",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })
}

/* === Tests === */

pub(crate) fn named_analysis_params() {
    let mut registrar = harmos::plugin::Registrar::new();
    dokime::Evidence::<(), Analyses>::register(&mut registrar).unwrap();
    let registrations = registrar.finish();
    let declaration = registrations
        .iter()
        .find(|entry| entry.namespace == dokime::plugin::registration::ANALYSES_NAMESPACE)
        .unwrap();
    let definitions: Vec<Definition> = serde_json::from_slice(&declaration.payload).unwrap();
    assert_eq!(definitions.len(), 2);
    assert_eq!(definitions[0].id, Scale::ID);
    assert_eq!(definitions[0].version, 1);
    assert_eq!(
        definitions[0].parameters[0].metadata.identifier.as_str(),
        "factor"
    );
    assert_eq!(definitions[0].outputs[0].axis.as_ref().unwrap().key, "time");
}

pub(crate) async fn analysis_original_lifecycle() {
    let runtime = serve().await;
    let response = call::<Scale>(&runtime.operations, request(BTreeMap::new()))
        .await
        .unwrap();
    assert_eq!(response.outputs[0].axis, [0.0, 0.25, 0.5]);
    assert_eq!(response.outputs[0].values, [4.0, -2.0, 6.0]);
    let invalid = call::<Scale>(
        &runtime.operations,
        request(BTreeMap::from([("factor".into(), Value::Number(11.0))])),
    )
    .await;
    assert!(
        invalid.is_err(),
        "typed parameter bounds must survive the native carrier"
    );
    runtime.stop().await.unwrap();
}

pub(crate) async fn cancelled_analysis_shutdown() {
    let (release, receiver) = mpsc::channel();
    *RELEASE.lock().unwrap() = Some(receiver);
    let runtime = serve().await;
    let operations = runtime.operations.clone();
    let pending =
        tokio::spawn(async move { call::<Held>(&operations, request(BTreeMap::new())).await });
    tokio::time::timeout(Duration::from_secs(5), STARTED.notified())
        .await
        .unwrap();
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    let shutdown = runtime.stop();
    tokio::pin!(shutdown);
    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut shutdown)
            .await
            .is_err(),
        "shutdown must retain the original numerical worker"
    );
    assert_eq!(FINISHED.load(Ordering::SeqCst), 0);
    release.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), shutdown)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        CALLS.load(Ordering::SeqCst),
        1,
        "cancelled work must not replay"
    );
    assert_eq!(FINISHED.load(Ordering::SeqCst), 1);
}

pub(crate) fn analysis_refuses_them() {
    let mut definition = Operation::<Scale>::definition().unwrap();
    definition.parameters[0].expression = dokime::definition::ParameterExpr::Linspace {
        start: 0.0,
        stop: 1.0,
        count: usize::MAX,
    };
    assert!(definition.check().unwrap_err().contains("literal defaults"));
}

pub(crate) fn result_units_output() {
    let definition = Operation::<Scale>::definition().unwrap();
    let mut response = Operation::<Scale>::evaluate(request(BTreeMap::new())).unwrap();
    response.outputs[0].axis_unit = "Hz".into();
    assert!(matches!(response.check(&definition), Err(Error::Output(_))));
    response.outputs[0].axis_unit = "s".into();
    response.outputs[0].unit = "A".into();
    assert!(matches!(response.check(&definition), Err(Error::Output(_))));
}
