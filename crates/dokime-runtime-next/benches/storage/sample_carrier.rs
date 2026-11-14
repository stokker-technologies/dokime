/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Tick-Driven Benchmark Sample Carrier

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error, io::Write};

// Third Party Imports
use harmos_protocol::{
    plugin::{
        InstanceGetRequest, InstanceGetResponse, InstanceInvokeRequest, InstanceInvokeResponse, InstanceSamplesRequest,
        InstanceSamplesResponse, InstanceStopRequest, InstanceStopResponse,
        instance_client::InstanceClient,
        instance_samples_request::Frame,
        instance_server::{Instance, InstanceServer},
    },
    v1::Sample,
};
use prost_types::Timestamp;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_stream::{
    StreamExt,
    wrappers::{ReceiverStream, TcpListenerStream},
};
use tonic::{
    Request, Response, Status, Streaming,
    transport::{Channel, Server},
};

// Local Imports
use dokime_runtime_next::{PlacementSettings, PluginProcess};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Generator = fn(&str, u64, &Timestamp) -> Vec<Sample>;

/* === Structs === */

#[derive(Clone)]
struct Carrier {
    client: InstanceClient<Channel>,
    generate: Generator,
}

pub struct Producer {
    carrier: PluginProcess,
    sdk: PluginProcess,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Producer {
    pub async fn start(settings: &PlacementSettings, environment: &[(String, String)]) -> Result<Self, Box<dyn Error>> {
        let executable = std::env::current_exe()?;
        let mut sdk_environment = environment.to_vec();
        sdk_environment.push(("DOKIME_BENCH_SDK".into(), "1".into()));
        let sdk = PluginProcess::start_with_environment(&executable, true, settings, &sdk_environment).await?;
        let mut carrier_environment = environment.to_vec();
        carrier_environment.push(("DOKIME_BENCH_SDK_URL".into(), sdk.endpoint().into()));
        let carrier = PluginProcess::start_with_environment(&executable, true, settings, &carrier_environment).await?;
        Ok(Self { carrier, sdk })
    }
    pub fn endpoint(&self) -> &str {
        self.carrier.endpoint()
    }
    pub async fn close(&mut self) -> Result<(), Box<dyn Error>> {
        let carrier = self.carrier.close().await;
        let sdk = self.sdk.close().await;
        carrier?;
        sdk?;
        Ok(())
    }
}

#[tonic::async_trait]
impl Instance for Carrier {
    async fn get(&self, request: Request<InstanceGetRequest>) -> Result<Response<InstanceGetResponse>, Status> {
        self.client.clone().get(request).await
    }

    type InvokeStream = Streaming<InstanceInvokeResponse>;
    async fn invoke(&self, request: Request<Streaming<InstanceInvokeRequest>>) -> Result<Response<Self::InvokeStream>, Status> {
        let mut input = request.into_inner();
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(async move {
            loop {
                let frame = input.message().await.expect("Forward Benchmark Invocation: Read Host Frame");
                let Some(frame) = frame else { break };
                if sender.send(frame).await.is_err() {
                    break;
                }
            }
        });
        self.client.clone().invoke(ReceiverStream::new(receiver)).await
    }

    type SamplesStream = ReceiverStream<Result<InstanceSamplesResponse, Status>>;
    async fn samples(&self, request: Request<Streaming<InstanceSamplesRequest>>) -> Result<Response<Self::SamplesStream>, Status> {
        let mut input = request.into_inner();
        let generate = self.generate;
        let (sender, receiver) = mpsc::channel(8);
        tokio::spawn(async move {
            let mut sequence = 0;
            let mut stream = String::new();
            while let Some(request) = input.next().await {
                let response = match request {
                    Ok(InstanceSamplesRequest { frame: Some(Frame::Open(selection)) }) => {
                        stream = selection.stream;
                        continue;
                    }
                    Ok(InstanceSamplesRequest { frame: Some(Frame::Tick(tick)) }) => {
                        let Some(at) = tick.at else { break };
                        let samples = generate(&stream, sequence, &at);
                        sequence += samples.len() as u64;
                        Ok(InstanceSamplesResponse { samples })
                    }
                    Ok(_) => Err(Status::invalid_argument("Sample Benchmark: Open a Stream and Supply Timestamped Ticks")),
                    Err(error) => Err(error),
                };
                if sender.send(response).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(receiver)))
    }

    async fn stop(&self, _request: Request<InstanceStopRequest>) -> Result<Response<InstanceStopResponse>, Status> {
        Ok(Response::new(InstanceStopResponse {}))
    }
}

/* === Public Functions === */

// Only The Benchmark Producer Replaces Its Samples Carrier. Host Admission,
// Clock Callbacks, Validation, Storage And Durability Are Product Code.
pub async fn serve(generate: Generator) -> Result<(), Box<dyn Error>> {
    let endpoint = std::env::var("DOKIME_BENCH_SDK_URL")?;
    let client = InstanceClient::connect(endpoint).await?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    println!("harmos listening {}", listener.local_addr()?);
    std::io::stdout().flush()?;
    Server::builder()
        .add_service(InstanceServer::new(Carrier { client, generate }))
        .serve_with_incoming(TcpListenerStream::new(listener))
        .await?;
    // The Parent Owns Both Processes and Closes Their Protocols Before Reaping.
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
