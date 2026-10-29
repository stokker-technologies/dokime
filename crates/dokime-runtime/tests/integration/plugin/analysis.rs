/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, mpsc};
use std::time::Duration;

use harmos::plugin::Contribution;

use dokime_runtime::Value;
use dokime_runtime::analysis::{Analysis, Axis, Definition, Limits, Operation, Output, Request, Response, ResultData, Samples, Values};

static RELEASE: Mutex<Option<mpsc::Receiver<()>>> = Mutex::new(None);
static STARTED: tokio::sync::Notify = tokio::sync::Notify::const_new();
static CALLS: AtomicUsize = AtomicUsize::new(0);
static FINISHED: AtomicUsize = AtomicUsize::new(0);

struct Host;
struct Scale;
struct Held;
struct Scaled {
    axis: Vec<f64>,
    values: Vec<f64>,
}

#[dokime_runtime::parameters]
struct Parameters {
    #[parameter(label = "Factor", default = 2.0, min = 0.0, max = 10.0)]
    factor: f64,
}

#[harmos::catalog]
enum Analyses {
    Scale(Operation<Scale>),
    Held(Operation<Held>),
}

#[harmos::lanes]
enum Lanes {
    #[default]
    Open,
}

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
    const ID: &'static str = "analysis_fixture.scale";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Scale Signal";
    const DESCRIPTION: &'static str = "Multiply Original Samples By A Typed Factor";
    const TIMEOUT: Duration = Duration::from_secs(120);
    const LIMITS: Limits = Limits { max_samples: 64, max_output_values: 64 };

    fn evaluate(samples: &Samples, parameters: &Parameters) -> Result<Scaled, Self::Error> {
        Ok(Scaled {
            axis: samples.axis.clone(),
            values: samples.values.iter().map(|value| value * parameters.factor).collect(),
        })
    }
}

impl Analysis for Held {
    type Parameters = ();
    type Output = Scaled;
    type Error = std::sync::mpsc::RecvError;
    const ID: &'static str = "analysis_fixture.held";
    const VERSION: u32 = 1;
    const LABEL: &'static str = "Held Signal";
    const DESCRIPTION: &'static str = "Retained Worker Fixture";
    const TIMEOUT: Duration = Duration::from_secs(120);
    const LIMITS: Limits = Scale::LIMITS;

    fn evaluate(samples: &Samples, _: &()) -> Result<Scaled, Self::Error> {
        CALLS.fetch_add(1, Ordering::SeqCst);
        let release = RELEASE.lock().unwrap().take().unwrap();
        STARTED.notify_one();
        release.recv()?;
        FINISHED.fetch_add(1, Ordering::SeqCst);
        Ok(Scaled {
            axis: samples.axis.clone(),
            values: samples.values.clone(),
        })
    }
}

impl harmos::Application for Host {
    type Lanes = Lanes;
    type State = ();
    type Transaction = harmos::state::Never;
    type Record = ();
    type Streams = ();
}

impl harmos::Lifecycle for Host {
    type Config = ();
    type Error = harmos::Error;
    type Context = harmos::Startup<Self>;
    async fn prepare(_: (), setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
        setup.settings().operation_settings(harmos::operations::Settings {
            preparation_bytes: Some(NonZeroUsize::new(4096).unwrap()),
            ..Default::default()
        });
        setup.operations::<Analyses>(Lanes::Open);
        Ok(Self)
    }
    async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
        Ok(())
    }
    async fn cleanup(&mut self) -> harmos::Result<()> {
        Ok(())
    }
}

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
    harmos::Runtime::<Host>::launch((), ()).await.unwrap()
}

async fn call<A: Analysis>(operations: &harmos::operations::OperationManager<Lanes>, request: Request) -> harmos::Result<Response> {
    operations.call::<Operation<A>>(Operation::new(request)).await.map(|response| response.into_owned())
}

#[test]
fn named_analysis_params() {
    let mut registrar = harmos::plugin::Registrar::new();
    dokime_runtime::Evidence::<(), Analyses>::register(&mut registrar).unwrap();
    let registrations = registrar.finish();
    let declaration =
        registrations.iter().find(|entry| entry.namespace == dokime_runtime::plugin::registration::ANALYSES_NAMESPACE).unwrap();
    let definitions: Vec<Definition> = serde_json::from_slice(&declaration.payload).unwrap();
    assert_eq!(definitions.len(), 2);
    assert_eq!(definitions[0].id, Scale::ID);
    assert_eq!(definitions[0].version, 1);
    assert_eq!(definitions[0].parameters[0].metadata.identifier.as_str(), "factor");
    assert_eq!(definitions[0].outputs[0].axis.as_ref().unwrap().key, "time");
}

#[tokio::test]
async fn analysis_original_lifecycle() {
    let runtime = serve().await;
    let response = call::<Scale>(&runtime.operations, request(BTreeMap::new())).await.unwrap();
    assert_eq!(response.outputs[0].axis, [0.0, 0.25, 0.5]);
    assert_eq!(response.outputs[0].values, [4.0, -2.0, 6.0]);
    let invalid = call::<Scale>(&runtime.operations, request(BTreeMap::from([("factor".into(), Value::Number(11.0))]))).await;
    assert!(invalid.is_err(), "typed parameter bounds must survive the native carrier");
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn cancelled_analysis_shutdown() {
    let (release, receiver) = mpsc::channel();
    *RELEASE.lock().unwrap() = Some(receiver);
    let runtime = serve().await;
    let operations = runtime.operations.clone();
    let pending = tokio::spawn(async move { call::<Held>(&operations, request(BTreeMap::new())).await });
    STARTED.notified().await;
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    let shutdown = runtime.shutdown();
    tokio::pin!(shutdown);
    assert!(
        std::future::poll_fn(|cx| std::task::Poll::Ready(std::future::Future::poll(shutdown.as_mut(), cx).is_pending())).await,
        "shutdown retains the original numerical worker until its release"
    );
    assert_eq!(FINISHED.load(Ordering::SeqCst), 0);
    release.send(()).unwrap();
    shutdown.await.unwrap();
    assert_eq!(CALLS.load(Ordering::SeqCst), 1, "cancelled work must not replay");
    assert_eq!(FINISHED.load(Ordering::SeqCst), 1);
}
