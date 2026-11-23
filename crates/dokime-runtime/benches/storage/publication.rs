/* ***
 *
 * Dokime
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
    env,
    hint::black_box,
    io::{Read, Seek, Write},
    num::NonZeroUsize,
    path::PathBuf,
    process,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode};
use tokio::runtime::Runtime;

// Local Imports
use dokime::prelude::*;
use dokime::templates::Context;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::evidence_client::EvidenceClient;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::{EvidenceArchiveRequest, EvidenceSubject, FilesReadRequest, PromptsCreateResponse, evidence_subject};
use dokime_runtime::{ControlledClock, CredentialNamespace, Cycle, Host, HostSettings, PlacementSettings, PluginProcess, services};
use dokime_storage::storage_defaults;

use dokime_protocol::model::Value;
use dokime_protocol::model::value;

/* === Constants === */

/// The Bench Client's Operation Deadlines.
const OPERATION_TIMEOUT: Duration = Duration::from_secs(30);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(dokime::Row)]
#[dokime(package = "storage.benchmark.v1")]
struct Reading {
    timestamp: String,
    value: f64,
}

#[derive(dokime::Row)]
#[dokime(package = "storage.benchmark.v1")]
struct Pulse {
    value: f64,
}

#[derive(dokime::Record)]
#[dokime(package = "storage.benchmark.v1")]
struct Produced {
    rows: u64,
}

#[derive(dokime::Parameters)]
#[dokime(package = "storage.benchmark.v1")]
struct Capture {
    rows: u64,
}

#[derive(Default)]
struct Observed {
    count: AtomicU64,
}

pub(super) struct Fixture {
    pub host: Host,
    pub root: PathBuf,
    pub cycle: Cycle,
    pub execution: String,
    template: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Fixture {
    pub(super) async fn new(settings: HostSettings, endpoint: &str, template: &str, rows: usize) -> Self {
        let root = tempfile::tempdir().unwrap().keep();
        let operations = services::ClientSettings {
            read_timeout: OPERATION_TIMEOUT,
            mutation_timeout: OPERATION_TIMEOUT,
            watch_timeout: OPERATION_TIMEOUT,
            max_cases: settings.placement.stream_window.get(),
            page_items: settings.placement.stream_window.get(),
            message_bytes: settings.placement.message_bytes.get(),
            stream_buffer: settings.placement.stream_window.get(),
        };
        let host = Host::open(root.as_path(), settings, operations).await.unwrap();
        let plugin = host.add_remote("storage.benchmark", endpoint).await.unwrap();
        let parameters = HashMap::from([(
            "rows".into(),
            Value {
                kind: Some(value::Kind::UnsignedInteger(rows as u64)),
            },
        )]);
        let origin = host.materialize(&plugin, template, parameters).await.unwrap();
        let cycle = host.start_cycle(vec![origin]).await.unwrap();
        let execution = cycle.executions[0].clone();
        Self {
            host,
            root,
            cycle,
            execution,
            template: template.into(),
        }
    }

    pub(super) async fn prompt(&self, name: &str) -> PromptsCreateRequest {
        let mut changed = self.host.changes();
        loop {
            changed.borrow_and_update();
            if let Some((_, prompt)) = self
                .host
                .pending_prompts()
                .unwrap()
                .into_iter()
                .find(|(execution, prompt)| execution == &self.execution && prompt.prompt == format!("{}.{name}", self.template))
            {
                return prompt;
            }
            changed.changed().await.expect("Await Benchmark Prompt: Host Remains Open");
        }
    }

    pub(super) async fn release(&self, name: &str) {
        let prompt = self.prompt(name).await;
        self.host
            .answer(
                &self.execution,
                PromptsCreateResponse {
                    prompt: prompt.prompt,
                    action: "0".into(),
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
        let index = if context.test_case().is_ok_and(|case| case.streams.iter().any(|stream| stream == "storage.benchmark.readings")) {
            let observed = context.get::<Observed>()?;
            observed.count.fetch_add(1, Ordering::SeqCst)
        } else {
            0
        };
        Ok(Reading {
            timestamp: at.to_string(),
            value: (index % 1024) as f64,
        })
    })
    .every(Duration::from_millis(1));
    let template = Template::of::<()>()
        .identifier("capture")
        .label("Capture")
        .record(Record::<Produced>::new("produced", "Produced"))
        .prompt(Prompt::new("ready", "Ready", "Continue the Measurement?").action("0", "Continue"))
        .prompt(Prompt::new("captured", "Captured", "Continue the Measurement?").action("0", "Continue"))
        .parameters(Capture { rows: 1024 })
        .initialize(async |context: &Context| {
            context.set(Observed::default());
            Ok(())
        })
        .step(Step::new("Capture", capture))
        .stream(stream)
        .observer(observer);
    let pulse = Stream::<Pulse>::new("pulse", "Pulse");
    let observer = Observer::new(&pulse, async |context: &Context| {
        let index = if context.test_case().is_ok_and(|case| case.streams.iter().any(|stream| stream == "storage.benchmark.pulse")) {
            let observed = context.get::<Observed>()?;
            observed.count.fetch_add(1, Ordering::SeqCst)
        } else {
            0
        };
        Ok(Pulse { value: index as f64 })
    })
    .every(Duration::from_millis(1));
    let mut export = Template::of::<()>()
        .identifier("export")
        .label("Export")
        .record(Record::<Produced>::new("produced", "Produced"))
        .prompt(Prompt::new("ready", "Ready", "Continue the Measurement?").action("0", "Continue"))
        .prompt(Prompt::new("captured", "Captured", "Continue the Measurement?").action("0", "Continue"))
        .parameters(Capture { rows: 16384 })
        .initialize(async |context: &Context| {
            context.set(Observed::default());
            Ok(())
        })
        .step(Step::new("Capture", capture))
        .stream(pulse)
        .observer(observer);
    // The Spectrum's First Scheduled Sample Is Outside These Short Captures.
    let spectrum = Stream::<Pulse>::new("spectrum", "Spectrum");
    let spectrum_observer = Observer::new(&spectrum, async |_: &Context| Ok(Pulse { value: 0.0 })).every(Duration::from_secs(86_400));
    export = export.stream(spectrum).observer(spectrum_observer);
    Plugin::of::<()>("1.0.0")
        .identifier("storage.benchmark")
        .suite(Suite::of::<()>().identifier("storage.benchmark").label("Storage Benchmark").template(template).template(export))
        .run()
}

pub fn benchmark(criterion: &mut Criterion, executor: &Runtime) {
    let mut group = criterion.benchmark_group("runtime");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    let mut plugin = None;
    super::boundary(
        "Runtime Observer Publication and Evidence Barrier, 1,024 Rows",
        "warm host, fresh execution; release ready step, host windows answered by a one-sample producer encoding 1,024 samples, plugin transport and batch admission, append produced-count record (durable evidence barrier), await captured prompt; execution end and shutdown excluded",
    );
    group.bench_function("observer_publication_and_barrier_durable/1024", |b| {
        let plugin = plugin.get_or_insert_with(|| {
            executor.block_on(PluginProcess::start(&env::current_exe().unwrap(), true, &settings().placement)).unwrap()
        });
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::new(settings(), plugin.endpoint(), "storage.benchmark.capture", 1024).await;
                    fixture.prompt("ready").await;
                    let started = Instant::now();
                    fixture.release("ready").await;
                    fixture.prompt("captured").await;
                    elapsed += started.elapsed();
                    fixture.release("captured").await;
                    let completed = fixture.cycle.wait().await.unwrap();
                    assert_eq!(completed.executions[0].samples.len(), 1024, "Prepare Runtime Publication: Retain Every Sample");
                    fixture.host.shutdown().await.unwrap();
                    services::remove_credentials(fixture.root.as_path(), &credentials()).unwrap();
                }
                super::report(elapsed, iterations)
            })
        });
    });
    super::boundary(
        "Execution End and Host Shutdown, 65,536 Rows",
        "warm populated host; answer final prompt, execution cleanup, stream finalization, seal, report and evidence archive freeze, final snapshot join and shutdown; durable; capture excluded",
    );
    group.bench_function("execution_end_and_shutdown/65536", |b| {
        let plugin = plugin.get_or_insert_with(|| {
            executor.block_on(PluginProcess::start(&env::current_exe().unwrap(), true, &settings().placement)).unwrap()
        });
        b.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::new(settings(), plugin.endpoint(), "storage.benchmark.capture", 65536).await;
                    fixture.release("ready").await;
                    fixture.prompt("captured").await;
                    let started = Instant::now();
                    fixture.release("captured").await;
                    let completed = fixture.cycle.wait().await.unwrap();
                    fixture.host.shutdown().await.unwrap();
                    elapsed += started.elapsed();
                    services::remove_credentials(fixture.root.as_path(), &credentials()).unwrap();
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
                    plugin = Some(PluginProcess::start(&env::current_exe().unwrap(), true, &settings().placement).await.unwrap());
                }
                let plugin = plugin.as_ref().unwrap();
                let fixture = Fixture::new(settings(), plugin.endpoint(), "storage.benchmark.export", 16384).await;
                fixture.release("ready").await;
                fixture.release("captured").await;
                let Fixture { host, root, cycle, execution, .. } = fixture;
                let completed = cycle.wait().await.unwrap();
                assert_eq!(completed.executions[0].samples.len(), 16384, "Prepare Runtime Archive: Retain Every Sample");
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
                let server = services::serve(([127, 0, 0, 1], 0).into(), root.as_path(), settings(), bounds).await.unwrap();
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
                    .archive(EvidenceArchiveRequest {
                        subject: Some(EvidenceSubject {
                            kind: Some(evidence_subject::Kind::Execution(execution.clone())),
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
                        .read(FilesReadRequest {
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
        services::remove_credentials(root.as_path(), &credentials()).unwrap();
    }
    group.finish();
    if let Some(mut plugin) = plugin {
        executor.block_on(plugin.close()).unwrap();
    }
}

/* === Private Functions === */

/// This Bench Process's Own Credential Namespace, So Benches Never Touch the Operator's Credentials.
pub(super) fn credentials() -> CredentialNamespace {
    static NAMESPACE: OnceLock<CredentialNamespace> = OnceLock::new();
    NAMESPACE
        .get_or_init(|| {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
            CredentialNamespace::new(format!("dokime-bench-{}-{unique}", process::id())).expect("Name the Bench Credential Namespace")
        })
        .clone()
}

pub(super) fn settings() -> HostSettings {
    HostSettings {
        installation: None,
        credentials: credentials(),
        clock: Arc::new(ControlledClock::new(1_788_220_800_000_000_000)),
        storage: storage_defaults(),
        placement: PlacementSettings {
            reconnect_delay: Duration::from_millis(25),
            stream_window: NonZeroUsize::new(8).unwrap(),
            interval_window: dokime_runtime::INTERVAL_WINDOW,
            interval_lag: dokime_runtime::INTERVAL_LAG,
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        capture: dokime_runtime::CaptureSettings::default(),
    }
}

async fn capture(context: &Context) -> Result<()> {
    context.prompt(Prompt::new("ready", "Ready", "Continue the Measurement?").action("0", "Continue"), &[]).await?;
    let rows = context.parameters::<Capture>()?.rows;
    let observed = context.get::<Observed>()?;
    // A Controlled Sleep Settles After Every Window Due by Its End.
    context.sleep(Duration::from_millis(rows.saturating_sub(observed.count.load(Ordering::SeqCst)))).await?;
    assert_eq!(observed.count.load(Ordering::SeqCst), rows, "Publish Runtime Samples: Retain the Requested Row Count");
    context.record(&Record::<Produced>::new("produced", "Produced"), Produced { rows }).await?;
    context.prompt(Prompt::new("captured", "Captured", "Continue the Measurement?").action("0", "Continue"), &[]).await?;
    Ok(())
}

/* === Tests === */
