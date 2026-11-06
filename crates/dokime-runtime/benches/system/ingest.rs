/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Sidecar Stream Ingestion

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeSet,
    io::{Seek, SeekFrom},
    path::Path,
    time::{Duration, Instant},
};

// Third Party Imports
use arrow_array::{StringArray, StructArray};
use criterion::{BenchmarkId, Criterion, SamplingMode, Throughput};
use futures_util::FutureExt;
use rusqlite::Connection;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tracing_subscriber::{
    Layer,
    layer::{Context, SubscriberExt},
    registry::LookupSpan,
};

// Local Imports
use dokime_runtime::{Runtime, RuntimeConfig, Value, plugins::PluginBundle, runtime::command::CommandContext};

/* === Constants === */
static PROFILE: std::sync::OnceLock<Timings> = std::sync::OnceLock::new();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
struct Timings(std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<&'static str, u128>>>);

#[derive(Default)]
struct Active(std::collections::HashMap<std::thread::ThreadId, Vec<Instant>>);

#[derive(Default)]
struct Elapsed(Option<u64>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<S: tracing::Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Timings {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        let name = event.metadata().name();
        if name.starts_with("ingest.") {
            let mut elapsed = Elapsed::default();
            event.record(&mut elapsed);
            if let Some(ns) = elapsed.0 {
                *self.0.lock().unwrap().entry(name).or_default() += u128::from(ns);
            }
        }
    }

    fn on_new_span(&self, _: &tracing::span::Attributes<'_>, id: &tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        if span.name() == "ingest.writer" {
            *self.0.lock().unwrap().entry("telemetry_flush_count").or_default() += 1;
        }
        if span.name() == "ingest.ipc.sync" {
            *self.0.lock().unwrap().entry("arrow_sync_count").or_default() += 1;
        }
    }

    fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        if span.name().starts_with("ingest.") {
            let mut extensions = span.extensions_mut();
            if extensions.get_mut::<Active>().is_none() {
                extensions.insert(Active::default());
            }
            extensions.get_mut::<Active>().unwrap().0.entry(std::thread::current().id()).or_default().push(Instant::now());
        }
    }

    fn on_exit(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        if let Some(started) =
            extensions.get_mut::<Active>().and_then(|active| active.0.get_mut(&std::thread::current().id()).and_then(Vec::pop))
        {
            *self.0.lock().unwrap().entry(span.name()).or_default() += started.elapsed().as_nanos();
        }
    }
}

impl tracing::field::Visit for Elapsed {
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if field.name() == "elapsed_ns" {
            self.0 = Some(value);
        }
    }
    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}
}

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    super::ingest_sync::install();
    {
        let profiled = std::env::var_os("DOKIME_INGEST_PROFILE").is_some();
        let timings = Timings::default();
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(timings.clone().with_filter(
            tracing_subscriber::filter::filter_fn(move |metadata| {
                metadata.name() == "ingest.writer"
                    || metadata.name() == "ingest.ipc.sync"
                    || metadata.name().starts_with("ingest.lock.")
                    || profiled && metadata.name().starts_with("ingest.")
            }),
        )))
        .unwrap();
        PROFILE.set(timings).ok().unwrap();
    }
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let rows = std::env::var("DOKIME_INGEST_ROWS").ok().map(|value| value.parse::<usize>().unwrap()).unwrap_or(100_000);
    let mut group = criterion.benchmark_group("sidecar_ingest");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(rows as u64));
    for (shape, wide, payload_bytes) in
        [("scalar", false, 0), ("wide16", true, 0), ("payload4k", false, 4096), ("payload64k", false, 65536)]
    {
        let rows = if payload_bytes == 0 {
            rows
        } else {
            std::env::var("DOKIME_INGEST_PAYLOAD_ROWS").ok().map(|value| value.parse().unwrap()).unwrap_or(10_000)
        };
        group.throughput(Throughput::Elements(rows as u64));
        let plots: &[u64] = if payload_bytes > 0 { &[0] } else { &[0, 1, 64] };
        for &plots in plots {
            group.bench_function(BenchmarkId::new(shape, plots), |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        elapsed += executor.block_on(measure(rows, wide, plots, payload_bytes));
                    }
                    elapsed
                });
            });
        }
    }
    group.finish();
}

/* === Private Functions === */

async fn measure(rows: usize, wide: bool, plots: u64, payload_bytes: usize) -> Duration {
    let root = tempfile::tempdir().unwrap();
    let config = RuntimeConfig::new(
        root.path(),
        dokime_runtime::Metadata::new(dokime_runtime::Identifier::new("bench.ingest").unwrap(), "Stream Ingest", None),
    )
    .with_plugins([
        PluginBundle::new(super::plugin_bundle("plugin-host.dokime")),
        PluginBundle::new(super::plugin_bundle("plugin-obc.dokime")),
    ]);
    let mut owner = Runtime::start(config).boxed_local().await.unwrap();
    let runtime = owner.handle();
    let registration = runtime.test_suites().await.into_iter().find(|suite| suite.suite_id == "obc.benchmarks").unwrap();
    let template_id = if payload_bytes > 0 {
        format!("obc.benchmarks.ingest_payload{payload_bytes}_{plots}")
    } else {
        format!("obc.benchmarks.ingest_{}{plots}", if wide { "wide" } else { "scalar" })
    };
    let template =
        registration.suite.test_case_templates.iter().find(|template| template.metadata.identifier.as_str() == template_id).unwrap();
    let mut parameters = std::collections::BTreeMap::new();
    parameters.insert("rows".into(), Value::from(rows as u64));
    let options = dokime_runtime::definition::CaseMaterializationOptions {
        force: true,
        max_cases: 1,
        registration_id: dokime_runtime::model::identity::new_id(),
        first_label_number: 1,
    };
    let mut cases = dokime_runtime::definition::materialize_template_cases(&registration.suite_id, template, Default::default(), options)
        .unwrap()
        .test_cases;
    for case in &mut cases {
        case.suite = Some(registration.suite.metadata.clone());
        case.parameters = parameters.clone();
    }
    let case = cases[0].test_case_id.clone();
    let cycle = runtime
        .create_cycle(
            dokime_runtime::transactions::CycleDefinition {
                metadata: registration.suite.metadata,
                continue_automatically: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    runtime.register_cases(dokime_runtime::model::identity::new_id(), cases).await.unwrap();
    let execution = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>();
    runtime.append_executions(cycle, vec![(execution.clone(), case)]).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut system = System::new().unwrap();
    let before = cpu(&mut system);
    let counts_before = dokime_runtime::storage::sqlite::counters();
    let syncs_before = super::ingest_sync::counts();
    if let Some(profile) = PROFILE.get() {
        profile.0.lock().unwrap().clear();
    }
    let started = Instant::now();
    let receipt = runtime.start_execution(execution.clone(), dokime_runtime::ExecutionMode::Automatic, CommandContext::default()).await;
    assert_eq!(receipt.status, dokime_runtime::runtime::command::CommandStatus::Accepted);
    tokio::time::timeout(Duration::from_secs(900), async {
        loop {
            if runtime.pending_prompt_projection().await.iter().any(|prompt| prompt.scope.test_execution_id.as_ref() == Some(&execution)) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("native ingest reaches its inspection barrier");
    runtime.storage().flush().await.unwrap();
    let elapsed = started.elapsed();
    let counts_after = dokime_runtime::storage::sqlite::counters();
    let syncs_after = super::ingest_sync::counts();
    let mut timings = PROFILE.get().unwrap().0.lock().unwrap().clone();
    let after = cpu(&mut system);
    let database = walkdir::WalkDir::new(root.path())
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_name() == "local.sqlite")
        .unwrap()
        .into_path();
    let connection = Connection::open(&database).unwrap();
    let stored: i64 = connection
        .query_row(
            "SELECT coalesce(sum(rows),0) FROM streams WHERE stream=?1",
            [if payload_bytes > 0 {
                "obc.benchmarks.payload"
            } else if wide {
                "obc.benchmarks.wide"
            } else {
                "obc.benchmarks.scalar"
            }],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, rows as i64);
    let bound_plots: i64 = connection
        .query_row("SELECT count(*) FROM report_plot_bindings WHERE execution=?1", [execution.as_str()], |row| row.get(0))
        .unwrap();
    assert_eq!(bound_plots, plots as i64);
    let arrow = bytes(root.path(), ".arrow");
    let catalog = std::fs::metadata(&database).unwrap().len();
    let wal = std::fs::metadata(database.with_file_name("local.sqlite-wal")).map(|metadata| metadata.len()).unwrap_or(0);
    let mut statement = connection.prepare("SELECT name, sum(pgsize) FROM dbstat GROUP BY name").unwrap();
    let tables: std::collections::BTreeMap<String, i64> =
        statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().collect::<Result<_, _>>().unwrap();
    let verified_payload_bytes = if payload_bytes > 0 {
        let mut total = 0;
        for entry in walkdir::WalkDir::new(root.path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|suffix| suffix == "arrow"))
        {
            let mut file = std::fs::File::open(entry.path()).unwrap();
            file.seek(SeekFrom::Start(8)).unwrap();
            let reader = arrow_ipc::reader::StreamReader::try_new(file, None).unwrap();
            for batch in reader {
                let batch = batch.unwrap();
                let values = batch.column_by_name("values").unwrap().as_any().downcast_ref::<StructArray>().unwrap();
                let data = values.column_by_name("data").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
                total += data.iter().map(|value| value.unwrap().len() + 8).sum::<usize>();
            }
        }
        assert_eq!(total, rows * payload_bytes);
        Some(total)
    } else {
        None
    };
    let logical_bytes = verified_payload_bytes.unwrap_or(rows * if wide { 128 } else { 8 });
    let flush_count = timings.remove("telemetry_flush_count").unwrap_or(0);
    let arrow_sync_count = timings.remove("arrow_sync_count").unwrap_or(0);
    let locks = timings
        .iter()
        .filter(|(name, _)| name.starts_with("ingest.lock."))
        .map(|(name, ns)| (*name, *ns))
        .collect::<std::collections::BTreeMap<_, _>>();
    timings.retain(|name, _| !name.starts_with("ingest.lock."));
    println!(
        "{}",
        serde_json::json!({"measurement":"sidecar_ingest", "revision":option_env!("GIT_REVISION"),
        "rows":rows, "payload_bytes_per_row":logical_bytes/rows,"verified_payload_bytes":verified_payload_bytes, "numeric_fields":if wide {16} else {1}, "plots":plots, "elapsed_s":elapsed.as_secs_f64(),
        "rows_s":rows as f64/elapsed.as_secs_f64(), "logical_mib_s":logical_bytes as f64/1048576.0/elapsed.as_secs_f64(),
        "cpu_ms_100k":after.iter().map(|(pid, (_, time))| time.saturating_sub(before.get(pid).map_or(0, |(_, time)| *time))).sum::<u64>() as f64*100000.0/rows as f64,
        "cpu_process_ms":after.iter().map(|(pid, (name, time))| (pid.to_string(), serde_json::json!({"name":name,"cpu_ms":time.saturating_sub(before.get(pid).map_or(0, |(_, time)| *time))}))).collect::<std::collections::BTreeMap<_,_>>(),
        "catalog_commits":counts_after.committed_transactions-counts_before.committed_transactions,
        "flush_count":flush_count,"arrow_sync_count":arrow_sync_count,
        "fsync_count":arrow_sync_count+u128::from(syncs_after.0-syncs_before.0),
        "lock_wait_ns":locks,"profiled":std::env::var_os("DOKIME_INGEST_PROFILE").is_some(),
        "sqlite_sync_count":syncs_after.0-syncs_before.0,"sqlite_wal_sync_count":syncs_after.1-syncs_before.1, "arrow_bytes":arrow,"sqlite_bytes":catalog,"wal_bytes":wal,"tables":tables,
        "inclusive_stage_ns":timings})
    );
    drop(statement);
    drop(connection);
    drop(runtime);
    owner.shutdown().await.unwrap();
    elapsed
}

fn bytes(root: &Path, suffix: &str) -> u64 {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().to_string_lossy().ends_with(suffix))
        .map(|entry| entry.metadata().unwrap().len())
        .sum()
}

fn cpu(system: &mut System) -> std::collections::BTreeMap<Pid, (String, u64)> {
    system.refresh_processes(ProcessesToUpdate::All, true);
    let mut selected = BTreeSet::from([Pid::from_u32(std::process::id())]);
    loop {
        let before = selected.len();
        for (pid, process) in system.processes() {
            if process.parent().is_some_and(|parent| selected.contains(&parent)) {
                selected.insert(*pid);
            }
        }
        if selected.len() == before {
            break;
        }
    }
    selected
        .into_iter()
        .filter_map(|pid| {
            system.process(pid).map(|process| (pid, (process.name().to_string_lossy().into_owned(), process.accumulated_cpu_time())))
        })
        .collect()
}

/* === Tests === */
