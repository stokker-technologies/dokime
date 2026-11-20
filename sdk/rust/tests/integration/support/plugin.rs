/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, path::PathBuf, process::Stdio};

// Third Party Imports
use harmos_protocol::plugin::instance_client::InstanceClient;
use prost::Message;
use harmos_runtime::plugin::grpc::Launches;
use tokio::{
    process::{Child, Command},
    sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

// Local Imports
use super::TestResult;
use harmos_protocol::model::Failure;
use harmos_protocol::plugin::ChildRequest;
use harmos_protocol::plugin::ChildResponse;
use harmos_protocol::plugin::InstanceGetRequest;
use harmos_protocol::plugin::InstanceGetResponse;
use harmos_protocol::plugin::InstanceInvokeRequest;
use harmos_protocol::plugin::InstanceInvokeResponse;
use harmos_protocol::plugin::InstanceSamplesRequest;
use harmos_protocol::plugin::InstanceSamplesResponse;
use harmos_protocol::plugin::InstanceStopRequest;
use harmos_protocol::plugin::Request;
use harmos_protocol::plugin::Response;
use harmos_protocol::plugin::SamplesSelection;
use harmos_protocol::plugin::Window;
use harmos_protocol::plugin::child_response;
use harmos_protocol::plugin::instance_invoke_request;
use harmos_protocol::plugin::instance_invoke_response;
use harmos_protocol::plugin::instance_samples_request;
use harmos_protocol::plugin::instance_samples_response;
use harmos_protocol::plugin::response;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Process {
    pub client: InstanceClient<Channel>,
    pub address: String,
    child: Child,
    placement: Placement,
}

pub struct Invocation {
    send: mpsc::Sender<InstanceInvokeRequest>,
    output: Streaming<InstanceInvokeResponse>,
}

/* === Enums === */

#[derive(Clone, Copy)]
pub enum Placement {
    Managed,
    Remote,
}

/* === Traits === */

/* === Implementations === */

impl Process {
    pub async fn start(name: &str, placement: Placement) -> TestResult<Self> {
        let mut command = Command::from(crate::process::command(binary(name)));
        let launches = Launches::new();
        let mut launch = launches.reserve(name).await?;
        command.envs(launch.environment());
        command.env_remove("DOKIME_REMOTE");
        let child = command.stdout(Stdio::null()).stderr(Stdio::inherit()).kill_on_drop(true).spawn()?;
        let address = format!("http://{}", launch.announced().await?);
        launches.shutdown().await;
        let client = InstanceClient::connect(address.clone()).await?;
        Ok(Self { client, address, child, placement })
    }
    pub async fn describe(&mut self) -> TestResult<InstanceGetResponse> {
        Ok(self
            .client
            .get(InstanceGetRequest {
                revision: 1,
                host: "sdk_story".into(),
                host_version: "1.0.0".into(),
            })
            .await?
            .into_inner())
    }
    pub async fn invoke<Q: Message>(&mut self, operation: &str, scope: &str, request: Q) -> TestResult<Invocation> {
        let (send, receive) = mpsc::channel(8);
        send.send(InstanceInvokeRequest {
            frame: Some(instance_invoke_request::Frame::Request(Request {
                operation: operation.into(),
                payload: request.encode_to_vec(),
                invocation: format!("{scope}:{operation}"),
                attempt: 1,
                scope: scope.into(),
            })),
        })
        .await?;
        let output = self.client.invoke(ReceiverStream::new(receive)).await?.into_inner();
        Ok(Invocation { send, output })
    }
    pub async fn publish(
        &mut self, stream: &str, scope: &str,
    ) -> TestResult<(mpsc::Sender<InstanceSamplesRequest>, Streaming<InstanceSamplesResponse>)> {
        let (send, receive) = mpsc::channel(8);
        send.send(InstanceSamplesRequest {
            frame: Some(instance_samples_request::Frame::Open(SamplesSelection {
                stream: stream.into(),
                scope: scope.into(),
            })),
        })
        .await?;
        let samples = self.client.samples(ReceiverStream::new(receive)).await?.into_inner();
        Ok((send, samples))
    }
    /// Grant One Interval Window and Return the Columns of Its Single Answering Part.
    pub async fn window(
        grants: &mpsc::Sender<InstanceSamplesRequest>, samples: &mut Streaming<InstanceSamplesResponse>, window: Window,
    ) -> TestResult<Vec<u8>> {
        let count = window.count;
        grants
            .send(InstanceSamplesRequest {
                frame: Some(instance_samples_request::Frame::Window(window)),
            })
            .await?;
        let answer = samples.message().await?.ok_or("Missing Window Answer")?;
        let Some(instance_samples_response::Frame::Batch(batch)) = answer.frame else {
            return Err(format!("Expected a Sample Batch, Got {answer:?}").into());
        };
        assert_eq!((batch.offset, batch.count, batch.last), (0, count, true));
        Ok(batch.columns)
    }
    pub async fn stop(mut self) -> TestResult<()> {
        self.client.stop(InstanceStopRequest { reason: "Story Complete".into() }).await?;
        if matches!(self.placement, Placement::Remote) {
            self.child.kill().await?;
        }
        let status = self.child.wait().await?;
        if matches!(self.placement, Placement::Managed) && !status.success() {
            return Err(format!("Plugin Exited: {status}").into());
        }
        Ok(())
    }
}

impl Invocation {
    pub async fn child(&mut self) -> TestResult<ChildRequest> {
        match self.output.message().await?.and_then(|frame| frame.frame) {
            Some(instance_invoke_response::Frame::Child(request)) => Ok(request),
            other => Err(format!("Expected Host Call, Got {other:?}").into()),
        }
    }
    pub async fn reply<R: Message>(&self, call: u64, response: R) -> TestResult<()> {
        self.send
            .send(InstanceInvokeRequest {
                frame: Some(instance_invoke_request::Frame::Child(ChildResponse {
                    call,
                    result: Some(child_response::Result::Payload(response.encode_to_vec())),
                })),
            })
            .await?;
        Ok(())
    }
    pub async fn finish<R: Message + Default>(mut self) -> TestResult<R> {
        let frame = self.output.message().await?.and_then(|frame| frame.frame);
        let Some(instance_invoke_response::Frame::Response(Response {
            result: Some(response::Result::Payload(payload)),
        })) = frame
        else {
            return Err(format!("Expected Successful Result, Got {frame:?}").into());
        };
        let response = R::decode(payload.as_slice())?;
        if self.output.message().await?.is_some() {
            return Err("Frames After Terminal Response".into());
        }
        Ok(response)
    }
    pub async fn failure(mut self) -> TestResult<Failure> {
        let frame = self.output.message().await?.and_then(|frame| frame.frame);
        let Some(instance_invoke_response::Frame::Response(Response {
            result: Some(response::Result::Failure(failure)),
        })) = frame
        else {
            return Err(format!("Expected Failure, Got {frame:?}").into());
        };
        if self.output.message().await?.is_some() {
            return Err("Frames After Terminal Failure".into());
        }
        Ok(failure)
    }
}

/* === Public Functions === */

pub fn binary(name: &str) -> PathBuf {
    env::var_os(format!("NEXTEST_BIN_EXE_{name}"))
        .or_else(|| env::var_os(format!("NEXTEST_BIN_EXE_{}", name.replace('-', "_"))))
        .or_else(|| env::var_os(format!("CARGO_BIN_EXE_{name}")))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let sdk = if manifest.ends_with("tests/consumer") { manifest.join("../..") } else { manifest };
            sdk.join("../../target/debug").join(format!("{name}{}", env::consts::EXE_SUFFIX))
        })
}

/* === Private Functions === */

/* === Tests === */
