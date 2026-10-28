/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ExitStatus},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use base64::Engine;
use serde_json::{Value, json};

use dokime_gui::{
    connection::ConnectionSession,
    rpc::{Bridge, FILE_DESCRIPTOR_SET},
};
use sha2::{Digest, Sha256};

pub type Window = tauri::WebviewWindow<tauri::test::MockRuntime>;
type ChannelFrames = HashMap<u32, Vec<Vec<u8>>>;

pub struct Harness {
    pub app: tauri::App<tauri::test::MockRuntime>,
    pub window: Window,
    writer: Writer,
    frames: Frames,
    channels: AtomicU32,
}

#[derive(Clone, Default)]
struct Writer(Arc<Mutex<Option<ChildStdin>>>);

#[derive(Clone, Default)]
struct Frames(Arc<(Mutex<ChannelFrames>, Condvar)>);

pub enum Local {
    Dispatch,
    Reply(Value),
}

impl Writer {
    fn line(&self, value: &Value) {
        let mut guard = self.0.lock().expect("harness writer poisoned");
        if let Some(input) = guard.as_mut() {
            // A finished child closes its input; late frames have no reader.
            let _ = writeln!(input, "{value}").and_then(|()| input.flush());
        }
    }
}

impl Frames {
    fn push(&self, channel: u32, bytes: Vec<u8>) {
        let (frames, ready) = &*self.0;
        frames.lock().expect("harness frames poisoned").entry(channel).or_default().push(bytes);
        ready.notify_all();
    }
}

impl Harness {
    pub fn new(session: ConnectionSession) -> Self {
        let writer = Writer::default();
        let lines = writer.clone();
        let frames = Frames::default();
        let recorded = frames.clone();
        let app = tauri::test::mock_builder()
            .manage(session)
            .manage(Bridge::default())
            .invoke_handler(dokime_gui::rpc::handler())
            .channel_interceptor(move |_webview, channel, index, body| {
                let bytes = match body {
                    tauri::ipc::InvokeResponseBody::Raw(bytes) => bytes.clone(),
                    tauri::ipc::InvokeResponseBody::Json(json) => json.clone().into_bytes(),
                };
                recorded.push(channel.0, bytes.clone());
                lines.line(&json!({
                    "channel": channel.0,
                    "index": index,
                    "bytes": encode(&bytes),
                }));
                true
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        Self {
            app,
            window,
            writer,
            frames,
            channels: AtomicU32::new(1),
        }
    }

    pub fn open(&self, generation: u64, path: &str, opening: &[u8], keep_input: bool) -> Result<(u64, u32), Value> {
        let channel = self.channels.fetch_add(1, Ordering::Relaxed);
        let reply = invoke(
            &self.window,
            "rpc_stream_open",
            fenced(generation, opening),
            &json!({
                "dokime-rpc-path": path,
                "dokime-rpc-channel": format!("__CHANNEL__:{channel}"),
                "dokime-rpc-input": if keep_input { "open" } else { "closed" },
            }),
        )?;
        let tauri::ipc::InvokeResponseBody::Raw(reply) = reply else {
            panic!("a stream opens with a raw handle");
        };
        assert_eq!(reply.len(), 16);
        assert_eq!(u64::from_le_bytes(reply[..8].try_into().unwrap()), generation);
        Ok((u64::from_le_bytes(reply[8..].try_into().unwrap()), channel))
    }

    pub fn control(
        &self, command: &str, generation: u64, stream: u64, payload: &[u8], extra: Value,
    ) -> Result<tauri::ipc::InvokeResponseBody, Value> {
        let mut headers = json!({ "dokime-rpc-stream": stream.to_string() });
        if let (Some(headers), Some(extra)) = (headers.as_object_mut(), extra.as_object()) {
            headers.extend(extra.clone());
        }
        invoke(&self.window, command, fenced(generation, payload), &headers)
    }

    pub fn settle(&self, channel: u32) -> Vec<Vec<u8>> {
        let (frames, ready) = &*self.frames.0;
        let guard = frames.lock().expect("harness frames poisoned");
        let guard = ready
            .wait_while(guard, |frames| !frames.get(&channel).is_some_and(|frames| frames.iter().any(|frame| frame[8] != 0)))
            .expect("harness frames poisoned");
        guard[&channel].clone()
    }

    pub fn drive(&self, child: &mut Child, mut local: impl FnMut(&str, &[u8]) -> Local, mut observe: impl FnMut(&Value)) -> ExitStatus {
        *self.writer.0.lock().expect("harness writer poisoned") = child.stdin.take();
        let output = BufReader::new(child.stdout.take().expect("piped child output"));
        let mut calls = Vec::new();
        for line in output.lines() {
            let request: Value = serde_json::from_str(&line.unwrap()).unwrap();
            let Some(id) = request.get("id").cloned() else {
                observe(&request);
                continue;
            };
            let command = request["cmd"].as_str().unwrap().to_owned();
            let bytes = decode(request["bytes"].as_str().unwrap_or_default());
            match local(&command, &bytes) {
                Local::Reply(mut reply) => {
                    reply["id"] = id;
                    self.writer.line(&reply);
                }
                Local::Dispatch => {
                    let window = self.window.clone();
                    let writer = self.writer.clone();
                    let headers = request["headers"].clone();
                    calls.push(std::thread::spawn(move || {
                        let mut reply = match invoke(&window, &command, bytes, &headers) {
                            Ok(tauri::ipc::InvokeResponseBody::Raw(bytes)) => {
                                json!({ "bytes": encode(&bytes) })
                            }
                            Ok(tauri::ipc::InvokeResponseBody::Json(json)) => {
                                json!({ "value": serde_json::from_str::<Value>(&json).unwrap() })
                            }
                            Err(error) => json!({ "error": error }),
                        };
                        reply["id"] = id;
                        writer.line(&reply);
                    }));
                }
            }
        }
        let status = child.wait().unwrap();
        self.writer.0.lock().expect("harness writer poisoned").take();
        for call in calls {
            call.join().unwrap();
        }
        status
    }
}

pub fn invoke(window: &Window, cmd: &str, bytes: Vec<u8>, headers: &Value) -> Result<tauri::ipc::InvokeResponseBody, Value> {
    let headers = headers
        .as_object()
        .into_iter()
        .flatten()
        .map(|(name, value)| {
            (
                tauri::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                tauri::http::HeaderValue::from_str(value.as_str().unwrap()).unwrap(),
            )
        })
        .collect();
    tauri::test::get_ipc_response(
        window,
        tauri::webview::InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: if cfg!(windows) { "http://tauri.localhost" } else { "tauri://localhost" }.parse().unwrap(),
            body: tauri::ipc::InvokeBody::Raw(bytes),
            headers,
            invoke_key: tauri::test::INVOKE_KEY.into(),
        },
    )
}

pub fn fenced(generation: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = generation.to_le_bytes().to_vec();
    bytes.extend(Sha256::digest(FILE_DESCRIPTOR_SET));
    bytes.extend_from_slice(payload);
    bytes
}

fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn decode(text: &str) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD.decode(text).expect("the harness sends base64 bytes")
}
