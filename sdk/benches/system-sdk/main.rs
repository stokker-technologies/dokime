/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Measure the Shared SDK Scenario

/* === Modules === */

mod memory;
mod scenario;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    env,
    error::Error,
    num::NonZeroUsize,
    path::PathBuf,
    process::{self, Stdio},
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

// Third Party Imports
use harmos_protocol::{
    host::{InvocationsCreateRequest, InvocationsCreateResponse},
    model::PageRequest,
};
use harmos_runtime::plugin::grpc::Launches;
use prost::Message;
use prost_types::Timestamp;
use tokio::{
    process::{Child, Command},
    task::JoinSet,
};
use tonic::{Request, client::Grpc, codegen::http::uri::PathAndQuery, transport::Channel};
use tonic_prost::ProstCodec;

// Local Imports
use dokime::{ConnectionSettings, client::ExecutionsQuery, connect_with};
use dokime_protocol::{
    host::{
        CasesCreateRequest, CasesCreateResponse, CyclesCreateRequest, CyclesCreateResponse, ExecutionsGetRequest, ExecutionsGetResponse,
        PluginEntry, PluginPlacement, PluginRemote, PluginState, PluginValue, PluginsCreateRequest, PluginsDeleteRequest,
        PluginsGetRequest, PluginsWatchRequest, RegisteredSelection, RegisteredSelectionParametersEntry, SelectionPlan, SelectionValues,
        evidence::{SamplesListRequest, SamplesListResponse, sample_row},
        plugin_placement,
        plugins_client::PluginsClient,
    },
    model::{EvidenceScope, Metadata, Value, origin, value},
};
use dokime_runtime::{
    CaptureSettings, CredentialNamespace, HostSettings, INTERVAL_LAG, INTERVAL_WINDOW, PlacementSettings,
    services::{self, ClientSettings, WallClock},
    storage_defaults,
};
use scenario::{Measurement, Scenario};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type BenchResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/* === Structs === */

struct Plugin {
    child: Child,
    entry: PluginEntry,
    ready_ms: f64,
}

#[derive(dokime::Message)]
#[dokime(package = "thermal")]
struct Echo {
    value: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Plugin {
    async fn start(channel: Channel, language: &str) -> BenchResult<Self> {
        let executable = env::var_os(if language == "rust" { "DOKIME_BENCH_REFERENCE" } else { "DOKIME_BENCH_EXECUTABLE" })
            .ok_or("Start SDK Bench: Supply the Built Reference Plugin Executable")?;
        let mut command = Command::from(dokime_runtime::process::command(executable));
        if language == "python" {
            command.args(["-m", "thermal_example.plugin"]);
        }
        let launches = Launches::new();
        let mut launch = launches.reserve("SDK Benchmark").await?;
        command
            .envs(launch.environment())
            .env_remove("DOKIME_REMOTE")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);
        let started = Instant::now();
        let mut child = command.spawn()?;
        let admitted = async {
            let address = launch.announced().await?;
            let mut plugins = PluginsClient::new(channel);
            let response = plugins
                .create(PluginsCreateRequest {
                    request_id: format!("sdk-{language}-{}", child.id().unwrap_or_default()),
                    name: "thermal".into(),
                    placement: Some(PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: format!("http://{address}") })),
                    }),
                    value: Some(PluginValue { enabled: true }),
                    ..Default::default()
                })
                .await?
                .into_inner();
            let mut entry = response.entry.ok_or("Admit Plugin: Return Its Entry")?;
            let mut watch = plugins.watch(PluginsWatchRequest { name: Some(entry.name.clone()), at: None }).await?.into_inner();
            while entry.state != PluginState::Ready as i32 {
                if let Some(failure) = &entry.failure {
                    return Err(format!("Admit Plugin: {}", failure.message).into());
                }
                watch.message().await?.ok_or("Admit Plugin: Keep Its Readiness Watch Open")?;
                entry = plugins
                    .get(PluginsGetRequest { name: entry.name, at: None })
                    .await?
                    .into_inner()
                    .entry
                    .ok_or("Admit Plugin: Keep Its Entry")?;
            }
            Ok::<_, Box<dyn Error + Send + Sync>>(entry)
        }
        .await;
        let ready_ms = started.elapsed().as_secs_f64() * 1000.0;
        launches.shutdown().await;
        match admitted {
            Ok(entry) => Ok(Self { child, entry, ready_ms }),
            Err(error) => {
                child.kill().await?;
                child.wait().await?;
                Err(error)
            }
        }
    }
    async fn close(mut self, channel: Channel) -> BenchResult<()> {
        let removed = PluginsClient::new(channel)
            .delete(PluginsDeleteRequest {
                name: self.entry.name,
                expected_revision: self.entry.revision,
            })
            .await;
        self.child.kill().await?;
        self.child.wait().await?;
        removed?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

#[tokio::main]
async fn main() -> BenchResult<()> {
    let language = env::var("DOKIME_BENCH_LANGUAGE").map_err(|_| "Measure SDK: Set DOKIME_BENCH_LANGUAGE")?;
    let root = PathBuf::from(env::var("DOKIME_BENCH_ROOT")?);
    let namespace = CredentialNamespace::new(format!("sdk-bench-{}", process::id()))?;
    let server = Box::pin(services::serve(
        "127.0.0.1:0".parse()?,
        &root,
        HostSettings {
            installation: None,
            credentials: namespace.clone(),
            storage: storage_defaults(),
            clock: Arc::new(WallClock),
            capture: CaptureSettings::default(),
            placement: PlacementSettings {
                reconnect_delay: Duration::from_millis(25),
                stream_window: NonZeroUsize::new(8).unwrap(),
                interval_window: INTERVAL_WINDOW,
                interval_lag: INTERVAL_LAG,
                message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            },
        },
        ClientSettings {
            read_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(30),
            watch_timeout: Duration::from_secs(3600),
            max_cases: 1000,
            page_items: 1000,
            message_bytes: 4 * 1024 * 1024,
            stream_buffer: 8,
        },
    ))
    .await?;
    let endpoint = format!("http://{}", server.endpoint());
    let channel = Channel::from_shared(endpoint.clone())?.connect().await?;
    let result = Box::pin(run(channel.clone(), &endpoint, &language)).await;
    let closed = server.close().await;
    services::remove_credentials(&root, &namespace)?;
    closed?;
    result
}

async fn run(channel: Channel, endpoint: &str, language: &str) -> BenchResult<()> {
    let reference = Plugin::start(channel.clone(), "rust").await?;
    let reference_catalog = scenario::catalog(
        reference
            .entry
            .description
            .as_ref()
            .and_then(|description| description.declarations.as_ref())
            .ok_or("Inspect Rust: Admit Its Declarations")?,
    )?;
    reference.close(channel.clone()).await?;
    let mut ready = Vec::new();
    for _ in 0..2 {
        let plugin = Plugin::start(channel.clone(), language).await?;
        let checked = scenario::guard(
            &reference_catalog,
            &scenario::catalog(
                plugin
                    .entry
                    .description
                    .as_ref()
                    .and_then(|description| description.declarations.as_ref())
                    .ok_or("Inspect Plugin: Admit Its Declarations")?,
            )?,
            language,
        );
        ready.push(plugin.ready_ms);
        plugin.close(channel.clone()).await?;
        checked?;
    }
    let plugin = Plugin::start(channel.clone(), language).await?;
    ready.push(plugin.ready_ms);
    let measured = async {
        let declarations = plugin
            .entry
            .description
            .as_ref()
            .and_then(|description| description.declarations.as_ref())
            .ok_or("Inspect Plugin: Admit Its Declarations")?;
        scenario::guard(&reference_catalog, &scenario::catalog(declarations)?, language)?;
        let mut measurements = Vec::new();
        measurements.push(Measurement::new(
            language,
            "start_to_ready",
            "Start to Ready [ms]",
            "ms",
            scenario::percentile(&ready, 50),
            None,
        ));
        let pid = plugin.child.id().ok_or("Read Plugin Memory: Keep Its Process Running")?;
        let idle = memory::resident(pid);
        measurements.push(Measurement::new(
            language,
            "memory/idle",
            "Memory [bytes]",
            "bytes",
            idle,
            idle.is_none().then(|| "Read Plugin Memory: Process Resident Memory Is Unavailable on This Platform".into()),
        ));
        let fingerprint = declarations
            .operations
            .iter()
            .find(|operation| operation.id == "thermal.echo")
            .ok_or("Measure Echo: Declare thermal.echo")?
            .fingerprint
            .clone();
        let mut latencies = Vec::new();
        for _ in 0..scenario::CALLS {
            let started = Instant::now();
            echo(channel.clone(), fingerprint.clone()).await?;
            latencies.push(started.elapsed().as_secs_f64() * 1e9);
        }
        for (percentile, case, label) in [
            (50, "round_trip/echo/p50", "Round Trip p50 [ns]"),
            (95, "round_trip/echo/p95", "Round Trip p95 [ns]"),
            (99, "round_trip/echo/p99", "Round Trip p99 [ns]"),
        ] {
            measurements.push(Measurement::new(language, case, label, "ns", scenario::percentile(&latencies, percentile), None));
        }
        let started = Instant::now();
        let mut callers = JoinSet::new();
        for _ in 0..scenario::CONCURRENCY {
            let channel = channel.clone();
            let fingerprint = fingerprint.clone();
            callers.spawn(async move {
                for _ in 0..scenario::CALLS {
                    echo(channel.clone(), fingerprint.clone()).await?;
                }
                Ok::<_, Box<dyn Error + Send + Sync>>(())
            });
        }
        while let Some(result) = callers.join_next().await {
            result??;
        }
        measurements.push(Measurement::new(
            language,
            "round_trip/echo_rate",
            "Calls [calls/s]",
            "calls/s",
            Some((scenario::CALLS * scenario::CONCURRENCY) as f64 / started.elapsed().as_secs_f64()),
            None,
        ));
        let scenario = Scenario::default();
        let (rate, loaded) = throughput(channel.clone(), endpoint, &scenario.channels, pid).await?;
        measurements.push(Measurement::new(
            language,
            "throughput/channels_128_at_1khz",
            "Throughput [samples/s]",
            "samples/s",
            Some(rate),
            None,
        ));
        measurements.push(Measurement::new(
            language,
            "memory/loaded",
            "Memory [bytes]",
            "bytes",
            loaded,
            loaded.is_none().then(|| "Read Plugin Memory: Process Resident Memory Is Unavailable on This Platform".into()),
        ));
        measurements.push(Measurement::new(
            language,
            "throughput/ceiling",
            "Throughput [samples/s]",
            "samples/s",
            None,
            Some(format!(
                "Count {} at {:?} ns: Provide Bounded Receipt-Time Counting; Samples.List Exceeded Its Read Deadline During the Sweep",
                scenario.ceiling, scenario.intervals_ns
            )),
        ));
        Ok::<_, Box<dyn Error + Send + Sync>>(measurements)
    }
    .await;
    plugin.close(channel).await?;
    for measurement in measured? {
        println!(
            "SDK\t{}\t{}\t{}\t{}\t{}",
            measurement.case,
            measurement.unit,
            measurement.value.map(|value| value.to_string()).unwrap_or_else(|| "not_measured".into()),
            measurement.label,
            measurement.reason.unwrap_or_default()
        );
    }
    Ok(())
}

async fn call<Q: Message + Default + Send + Sync + 'static, R: Message + Default + Send + Sync + 'static>(
    channel: Channel, path: &'static str, request: Q,
) -> BenchResult<R> {
    let mut client = Grpc::new(channel);
    client.ready().await?;
    Ok(client.unary(Request::new(request), PathAndQuery::from_static(path), ProstCodec::default()).await?.into_inner())
}

async fn echo(channel: Channel, fingerprint: Vec<u8>) -> BenchResult<()> {
    let request = Echo { value: scenario::PAYLOAD.to_vec() };
    let response: InvocationsCreateResponse = call(
        channel,
        "/dokime.host.v1.Invocations/Create",
        InvocationsCreateRequest {
            operation: "thermal.echo".into(),
            fingerprint,
            payload: request.encode_to_vec(),
        },
    )
    .await?;
    if Echo::decode(response.payload.as_slice())?.value != scenario::PAYLOAD {
        return Err("Measure Echo: Return the Request Payload Unchanged".into());
    }
    Ok(())
}

async fn throughput(channel: Channel, endpoint: &str, streams: &[String], pid: u32) -> BenchResult<(f64, Option<f64>)> {
    let identity = format!("load-{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_nanos());
    let cases: CasesCreateResponse = call(
        channel.clone(),
        "/dokime.host.v1.Cases/Create",
        CasesCreateRequest {
            request_id: identity.clone(),
            plan: Some(SelectionPlan {
                selections: vec![RegisteredSelection {
                    suite_id: "thermal".into(),
                    template_id: "thermal.load".into(),
                    parameters: vec![RegisteredSelectionParametersEntry {
                        key: "burst".into(),
                        value: Some(SelectionValues {
                            values: vec![Value { kind: Some(value::Kind::Boolean(false)) }],
                        }),
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }),
            validate_only: false,
        },
    )
    .await?;
    let case = cases.cases.first().ok_or("Measure Throughput: Materialize Its Case")?;
    let frozen = case
        .origin
        .as_ref()
        .and_then(|origin| match origin.kind.as_ref() {
            Some(origin::Kind::PluginRun(run)) => run.test_case.as_ref(),
            _ => None,
        })
        .ok_or("Measure Throughput: Freeze the Plugin Case Before Running")?;
    if frozen.streams.iter().collect::<BTreeSet<_>>() != streams.iter().collect::<BTreeSet<_>>() {
        return Err("Measure Throughput: Materialize the Shared Scenario's Exact Stream Selection".into());
    }
    let cycle: CyclesCreateResponse = call(
        channel.clone(),
        "/dokime.host.v1.Cycles/Create",
        CyclesCreateRequest {
            origins: vec![case.origin.clone().ok_or("Measure Throughput: Retain Its Frozen Origin")?],
            request_id: format!("cycle-{identity}"),
            metadata: Some(Metadata {
                label: "Thermal Cycle".into(),
                ..Default::default()
            }),
            ..Default::default()
        },
    )
    .await?;
    let cycle = cycle.cycle.ok_or("Measure Throughput: Create Its Cycle")?.cycle_id;
    let client = connect_with(ConnectionSettings {
        target: Some(endpoint.into()),
        ..Default::default()
    })
    .await?;
    let execution_handle = client
        .executions()
        .list_with(ExecutionsQuery { cycle, ..Default::default() })
        .await?
        .into_iter()
        .next()
        .ok_or("Measure Throughput: Admit Its Execution With the Frozen Origin")?;
    let execution = execution_handle.identifier().to_owned();
    execution_handle.start().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let start = Timestamp::from(SystemTime::now());
    let started = Instant::now();
    let mut loaded = memory::resident(pid);
    for _ in 0..10 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Some(value) = memory::resident(pid) {
            loaded = Some(loaded.unwrap_or_default().max(value));
        }
    }
    let elapsed = started.elapsed().as_secs_f64();
    let end = Timestamp::from(SystemTime::now());
    let current: ExecutionsGetResponse = call(
        channel.clone(),
        "/dokime.host.v1.Executions/Get",
        ExecutionsGetRequest {
            name: execution.clone(),
            ..Default::default()
        },
    )
    .await?;
    if current.execution.as_ref().and_then(|execution| execution.controls.as_ref()).is_some_and(|controls| controls.can_cancel) {
        execution_handle.cancel().await?;
    }
    let mut watch = execution_handle.watch();
    while watch.next().await?.is_some() {}
    let mut count = 0;
    let mut request = SamplesListRequest {
        scope: Some(EvidenceScope {
            execution_id: Some(execution),
            ..Default::default()
        }),
        streams: streams.to_vec(),
        ..Default::default()
    };
    loop {
        let page: SamplesListResponse = call(channel.clone(), "/dokime.host.v1.evidence.Samples/List", request.clone()).await?;
        for row in page.samples {
            let Some(sample_row::Kind::Raw(sample)) = row.kind else {
                return Err("Measure Throughput: Return Raw Accepted Samples".into());
            };
            let received = sample.received_at.ok_or("Measure Throughput: Retain Host Receipt Timestamps")?;
            count += usize::from(scenario::accepted(received, start, end));
        }
        if page.next_page_token.is_empty() {
            break;
        }
        request.at = page.resolved_at;
        request.page = Some(PageRequest {
            page_token: page.next_page_token,
            ..Default::default()
        });
    }
    client.close().await?;
    Ok((count as f64 / elapsed, loaded))
}

/* === Tests === */
