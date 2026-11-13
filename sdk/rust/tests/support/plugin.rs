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
use std::{path::PathBuf, process::Stdio};

// Third Party Imports
use harmos_protocol::{plugin::instance_client::InstanceClient, v1 as h};
use prost::Message;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Streaming, transport::Channel};

// Local Imports
use super::TestResult;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Process {
    pub client: InstanceClient<Channel>,
    pub address: String,
    child: Child,
    drain: tokio::task::JoinHandle<()>,
}

pub struct Invocation {
    send: mpsc::Sender<harmos_protocol::plugin::InstanceInvokeRequest>,
    output: Streaming<harmos_protocol::plugin::InstanceInvokeResponse>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Process {
    pub async fn start(name: &str, remote: bool) -> TestResult<Self> {
        let binary = std::env::var_os(format!("NEXTEST_BIN_EXE_{name}"))
            .or_else(|| std::env::var_os(format!("NEXTEST_BIN_EXE_{}", name.replace('-', "_"))))
            .or_else(|| std::env::var_os(format!("CARGO_BIN_EXE_{name}")))
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                let sdk = if manifest.ends_with("tests/consumer") { manifest.join("../..") } else { manifest };
                sdk.join("../../target/debug").join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
            });
        let mut command = Command::from(crate::process::command(binary));
        if remote {
            command.arg("--remote");
        }
        let mut child =
            command.env("HARMOS_ADDRESS", "127.0.0.1:0").stdout(Stdio::piped()).stderr(Stdio::inherit()).kill_on_drop(true).spawn()?;
        let mut lines = BufReader::new(child.stdout.take().ok_or("Missing Plugin Output")?).lines();
        let line = lines.next_line().await?.ok_or("Missing Plugin Readiness")?;
        let endpoint = line.strip_prefix("harmos listening ").ok_or("Invalid Plugin Readiness")?;
        let address = format!("http://{endpoint}");
        let client = InstanceClient::connect(address.clone()).await?;
        let drain = tokio::spawn(async move { while lines.next_line().await.ok().flatten().is_some() {} });
        Ok(Self { client, address, child, drain })
    }
    pub async fn describe(&mut self) -> TestResult<harmos_protocol::plugin::InstanceGetResponse> {
        Ok(self
            .client
            .get(harmos_protocol::plugin::InstanceGetRequest {
                revision: 1,
                host: "sdk_story".into(),
                host_version: "1.0.0".into(),
            })
            .await?
            .into_inner())
    }
    pub async fn invoke<Q: Message>(&mut self, operation: &str, scope: &str, request: Q) -> TestResult<Invocation> {
        let (send, receive) = mpsc::channel(8);
        send.send(harmos_protocol::plugin::InstanceInvokeRequest {
            frame: Some(harmos_protocol::plugin::instance_invoke_request::Frame::Request(h::Request {
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
    ) -> TestResult<(
        mpsc::Sender<harmos_protocol::plugin::InstanceSamplesRequest>,
        Streaming<harmos_protocol::plugin::InstanceSamplesResponse>,
    )> {
        let (send, receive) = mpsc::channel(8);
        send.send(harmos_protocol::plugin::InstanceSamplesRequest {
            frame: Some(harmos_protocol::plugin::instance_samples_request::Frame::Open(h::SamplesSelection {
                stream: stream.into(),
                scope: scope.into(),
            })),
        })
        .await?;
        let samples = self.client.samples(ReceiverStream::new(receive)).await?.into_inner();
        Ok((send, samples))
    }
    pub async fn stop(mut self, remote: bool) -> TestResult<()> {
        self.client.stop(harmos_protocol::plugin::InstanceStopRequest { reason: "Story Complete".into() }).await?;
        if remote {
            self.child.kill().await?;
        }
        let status = self.child.wait().await?;
        if !remote && !status.success() {
            return Err(format!("Plugin Exited: {status}").into());
        }
        self.drain.await?;
        Ok(())
    }
}

impl Invocation {
    pub async fn child(&mut self) -> TestResult<h::ChildRequest> {
        match self.output.message().await?.and_then(|frame| frame.frame) {
            Some(harmos_protocol::plugin::instance_invoke_response::Frame::Child(request)) => Ok(request),
            other => Err(format!("Expected Host Call, Got {other:?}").into()),
        }
    }
    pub async fn reply<R: Message>(&self, call: u64, response: R) -> TestResult<()> {
        self.send
            .send(harmos_protocol::plugin::InstanceInvokeRequest {
                frame: Some(harmos_protocol::plugin::instance_invoke_request::Frame::Child(h::ChildResponse {
                    call,
                    result: Some(h::child_response::Result::Payload(response.encode_to_vec())),
                })),
            })
            .await?;
        Ok(())
    }
    pub async fn finish<R: Message + Default>(mut self) -> TestResult<R> {
        let frame = self.output.message().await?.and_then(|frame| frame.frame);
        let Some(harmos_protocol::plugin::instance_invoke_response::Frame::Response(h::Response {
            result: Some(h::response::Result::Payload(payload)),
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
    pub async fn failure(mut self) -> TestResult<h::Failure> {
        let frame = self.output.message().await?.and_then(|frame| frame.frame);
        let Some(harmos_protocol::plugin::instance_invoke_response::Frame::Response(h::Response {
            result: Some(h::response::Result::Failure(failure)),
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

/* === Private Functions === */

/* === Tests === */
