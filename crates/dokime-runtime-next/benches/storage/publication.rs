/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Runtime Publication Benchmark Fixture

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    hint::black_box,
    io::{Read, Seek, Write},
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode};
use harmos_protocol::v1::Sample;
use prost::Message;
use prost_types::Timestamp;
use tempfile::TempDir;
use tokio::runtime::Runtime;

// Local Imports
use super::sample_carrier::Producer;
use dokime::{
    clock::TimeSleepRequest,
    prelude::*,
    protocol::plugin::host::v1::{PromptsCreateRequest, evidence::RecordsAppendRequest},
};
use dokime_protocol::{
    builders::Plot,
    client::{evidence_client::EvidenceClient, files_client::FilesClient},
};
use dokime_protocol::{
    client,
    context::{PromptsCreateRequest as PromptRequest, PromptsCreateResponse},
    v1,
};
use dokime_runtime_next::{ControlledClock, Cycle, Host, HostSettings, PlacementSettings, RenderSettings, services};
use dokime_storage::storage_defaults;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[harmos(package = "storage.benchmark.v1")]
struct Reading {
    timestamp: String,
    value: f64,
}

#[derive(dokime::Row)]
#[harmos(package = "storage.benchmark.v1")]
struct Pulse {
    value: f64,
}

#[derive(dokime::Record)]
#[harmos(package = "storage.benchmark.v1")]
struct Produced {
    rows: u64,
}

#[derive(dokime::Parameters)]
#[harmos(package = "storage.benchmark.v1")]
struct Capture {
    rows: u64,
}

struct Fixture {
    host: Host,
    root: TempDir,
    cycle: Cycle,
    execution: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    async fn new(endpoint: &str, template: &str, rows: usize) -> Self {
        let root = tempfile::tempdir().unwrap();
        let host = Host::open(root.path(), settings()).unwrap();
        let plugin = host.add_remote(endpoint).await.unwrap();
        let parameters = HashMap::from([(
            "rows".into(),
            v1::Value {
                kind: Some(v1::value::Kind::UnsignedInteger(rows as u64)),
            },
        )]);
        let origin = host.materialize(&plugin, template, parameters).await.unwrap();
        let cycle = host.start_cycle(vec![origin]).await.unwrap();
        let execution = cycle.executions[0].clone();
        Self { host, root, cycle, execution }
    }

    async fn prompt(&self, name: &str) -> PromptRequest {
        let mut changed = self.host.changes();
        loop {
            changed.borrow_and_update();
            if let Some((_, prompt)) = self
                .host
                .pending_prompts()
                .unwrap()
                .into_iter()
                .find(|(execution, prompt)| execution == &self.execution && prompt.prompt == name)
            {
                return prompt;
            }
            changed.changed().await.expect("Await Benchmark Prompt: Host Remains Open");
        }
    }

    async fn release(&self, name: &str) {
        let prompt = self.prompt(name).await;
        self.host
            .answer(
                &self.execution,
                PromptsCreateResponse {
                    prompt: prompt.prompt,
                    action: "continue".into(),
                    accepted: true,
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

/* === Public Functions === */

pub fn serve() -> Result<()> {
    let stream = Stream::<Reading>::new("readings", "Readings");
    let observer = Observer::new(&stream, async |context: &Context| {
        let at = context.now().await?;
        Ok(Reading {
            timestamp: at.to_string(),
            value: at.nanos as f64 / 1_000_000.0,
        })
    })
    .every_ms(1);
    let template = Template::new("capture", "Capture")
        .parameters(Capture { rows: 1024 })
        .step(Step::new("Capture", capture))
        .stream(stream)
        .observer(observer);
    let pulse = Stream::<Pulse>::new("pulse", "Pulse");
    let observer = Observer::new(&pulse, async |_: &Context| Ok(Pulse { value: 0.0 })).every_ms(1);
    let export = Template::new("export", "Export")
        .parameters(Capture { rows: 16384 })
        .step(Step::new("Capture", capture))
        .stream(pulse)
        .observer(observer);
    Plugin::new("storage.benchmark", "1.0.0")
        .suite(Suite::new("storage.benchmark", "Storage Benchmark").template(template).template(export))
        .stream(Stream::<Pulse>::new("spectrum", "Spectrum"))
        .record(Record::<Produced>::new("produced", "Produced"))
        .prompt(Prompt::new("continue", "Continue", "Continue the Measurement?").action("continue", "Continue"))
        .requires_host::<PromptsCreateRequest>()
        .requires_host::<RecordsAppendRequest>()
        .requires::<TimeSleepRequest>()
        .remote()
        .serve()
}

pub fn benchmark(criterion: &mut Criterion, executor: &Runtime) {
    let mut group = criterion.benchmark_group("runtime");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let mut plugin = None;
    super::boundary(
        "Runtime Observer Publication and Evidence Barrier, 1,024 Rows",
        "warm host, fresh execution; release ready step, tick-driven producer encoding of 1,024 samples, plugin transport and batch admission, append produced-count record (durable evidence barrier), await captured prompt; execution end and shutdown excluded",
    );
    group.bench_function("observer_publication_and_barrier_durable/1024", |b| {
        let plugin = plugin.get_or_insert_with(|| executor.block_on(Producer::start(&settings().placement, &[])).unwrap());
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::new(plugin.endpoint(), "capture", 1024).await;
                    fixture.prompt("ready").await;
                    let started = Instant::now();
                    fixture.release("ready").await;
                    fixture.prompt("captured").await;
                    elapsed += started.elapsed();
                    fixture.release("captured").await;
                    let completed = fixture.cycle.wait().await.unwrap();
                    assert_eq!(completed.executions[0].samples.len(), 1024, "Prepare Runtime Publication: Retain Every Sample");
                    fixture.host.shutdown().await.unwrap();
                }
                super::report(elapsed, iterations)
            })
        });
    });
    super::boundary(
        "Execution End and Host Shutdown, 65,536 Rows",
        "warm populated host; answer final prompt, execution cleanup, stream finalization, seal, final snapshot join and shutdown; durable; capture excluded",
    );
    group.bench_function("execution_end_and_shutdown/65536", |b| {
        let plugin = plugin.get_or_insert_with(|| executor.block_on(Producer::start(&settings().placement, &[])).unwrap());
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::new(plugin.endpoint(), "capture", 65536).await;
                    fixture.release("ready").await;
                    fixture.prompt("captured").await;
                    let started = Instant::now();
                    fixture.release("captured").await;
                    let completed = fixture.cycle.wait().await.unwrap();
                    fixture.host.shutdown().await.unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(completed.executions[0].samples.len(), 65536, "Prepare Runtime Shutdown: Retain Every Sample");
                }
                super::report(elapsed, iterations)
            })
        });
    });
    let mut export = None;
    super::boundary(
        "Execution Archive on Disk, 16,384 Rows",
        "warm host; Evidence.Archive and Files.Read over gRPC, including sealed metadata/records/report and unchanged Arrow files; write received ZIP to disk, sync_all, rewind and drain; durable; no raw SQLite database snapshot",
    );
    group.bench_function("execution_archive_disk_synced_drained/16384", |b| {
        let (server, _, execution, evidence, files) = export.get_or_insert_with(|| {
            executor.block_on(async {
                if plugin.is_none() {
                    plugin = Some(Producer::start(&settings().placement, &[]).await.unwrap());
                }
                let plugin = plugin.as_ref().unwrap();
                let fixture = Fixture::new(plugin.endpoint(), "export", 16384).await;
                fixture.release("ready").await;
                fixture.release("captured").await;
                let Fixture { host, root, cycle, execution } = fixture;
                let completed = cycle.wait().await.unwrap();
                assert_eq!(completed.executions[0].samples.len(), 16384, "Prepare Runtime Archive: Retain Every Sample");
                let snapshot = host.snapshot(&execution).unwrap();
                let mut pulse = Plot::time_series(v1::FieldReference::new("pulse").field("value")).message;
                pulse.display_points = Some(400);
                let mut spectrum = Plot::frequency_series(v1::FieldReference::new("spectrum").field("value")).message;
                spectrum.sample_rate_hz = Some(100.0);
                let document = v1::Document::new("Report").section(
                    v1::Section::new("Readings")
                        .card(v1::Card {
                            body: Some(v1::card::Body::Plot(pulse)),
                            ..Default::default()
                        })
                        .card(v1::Card {
                            body: Some(v1::card::Body::Plot(spectrum)),
                            ..Default::default()
                        }),
                );
                let report = host
                    .render_document(
                        &snapshot,
                        document,
                        RenderSettings {
                            page_size: NonZeroUsize::new(256).unwrap(),
                            display_points: NonZeroUsize::new(1500).unwrap(),
                        },
                    )
                    .unwrap();
                host.publish_report(&report).unwrap();
                host.shutdown().await.unwrap();
                let bounds = services::ClientSettings {
                    read_timeout: Duration::from_secs(30),
                    mutation_timeout: Duration::from_secs(30),
                    watch_timeout: Duration::from_secs(3600),
                    max_cases: 1000,
                    page_items: 1000,
                    message_bytes: 4 * 1024 * 1024,
                    stream_buffer: 8,
                };
                let server = services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings(), bounds).await.unwrap();
                let endpoint = format!("http://{}", server.endpoint());
                let evidence = EvidenceClient::connect(endpoint.clone()).await.unwrap();
                let files = FilesClient::connect(endpoint).await.unwrap();
                (server, root, execution, evidence, files)
            })
        });
        let _ = server;
        b.iter(|| {
            executor.block_on(async {
                let started = Instant::now();
                let mut archive = tempfile::tempfile().unwrap();
                let response = evidence
                    .archive(client::EvidenceArchiveRequest {
                        subject: Some(client::EvidenceSubject {
                            kind: Some(client::evidence_subject::Kind::Execution(execution.clone())),
                        }),
                        at: None,
                    })
                    .await
                    .unwrap()
                    .into_inner();
                let reference = response.file.unwrap();
                let mut offset = 0;
                while offset < reference.length {
                    let response = files
                        .read(client::FilesReadRequest {
                            file: Some(reference.clone()),
                            offset,
                            max_bytes: 2 * 1024 * 1024,
                            at: None,
                        })
                        .await
                        .unwrap()
                        .into_inner();
                    assert!(!response.data.is_empty());
                    offset += response.data.len() as u64;
                    archive.write_all(&response.data).unwrap();
                }
                archive.sync_all().unwrap();
                archive.rewind().unwrap();
                let mut bytes = vec![0; 2 * 1024 * 1024];
                let mut length = 0;
                loop {
                    let count = archive.read(&mut bytes).unwrap();
                    if count == 0 {
                        break;
                    }
                    black_box(&bytes[..count]);
                    length += count;
                }
                assert!(length > 0);
                super::report(started.elapsed(), 1);
            })
        });
    });
    if let Some((server, root, _, _, _)) = export {
        executor.block_on(server.close()).unwrap();
        services::remove_credentials(root.path()).unwrap();
    }
    group.finish();
    if let Some(mut plugin) = plugin {
        executor.block_on(plugin.close()).unwrap();
    }
}

pub fn samples(stream: &str, first: u64, _at: &Timestamp) -> Vec<Sample> {
    (first..first + 1024)
        .map(|index| {
            let source_ns = 1_788_220_800_000_000_000 + index as i64 * 1_000_000;
            let at = Timestamp {
                seconds: source_ns / 1_000_000_000,
                nanos: (source_ns % 1_000_000_000) as i32,
            };
            Sample {
                sequence: index + 1,
                at: Some(at),
                row: if stream == "pulse" {
                    Pulse { value: index as f64 }.encode_to_vec()
                } else {
                    Reading {
                        timestamp: at.to_string(),
                        value: (index % 1024) as f64,
                    }
                    .encode_to_vec()
                },
            }
        })
        .collect()
}

/* === Private Functions === */

fn settings() -> HostSettings {
    HostSettings {
        clock: Arc::new(ControlledClock::new(1_788_220_800_000_000_000)),
        storage: storage_defaults(),
        placement: PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_millis(25),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
    }
}

async fn capture(context: &Context) -> Result<()> {
    context
        .prompt(PromptsCreateRequest {
            prompt: "ready".into(),
            prompt_type: "continue".into(),
            ..Default::default()
        })
        .await?;
    let rows = context.parameters::<Capture>()?.rows;
    context.sleep(Duration::from_millis(rows.div_ceil(1024) - 1)).await?;
    context.record(&Record::<Produced>::new("produced", "Produced"), Produced { rows }).await?;
    context
        .prompt(PromptsCreateRequest {
            prompt: "captured".into(),
            prompt_type: "continue".into(),
            ..Default::default()
        })
        .await?;
    Ok(())
}

/* === Tests === */
