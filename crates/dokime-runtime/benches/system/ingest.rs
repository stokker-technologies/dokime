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
    path::Path,
    time::{Duration, Instant},
};

// Third Party Imports
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

struct Active(std::time::Instant);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<S: tracing::Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Timings {
    fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        if span.name().starts_with("ingest.") {
            span.extensions_mut().insert(Active(Instant::now()));
        }
    }

    fn on_exit(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        if let Some(active) = span.extensions_mut().remove::<Active>() {
            *self.0.lock().unwrap().entry(span.name()).or_default() += active.0.elapsed().as_nanos();
        }
    }
}

/* === Public Functions === */

pub fn benchmark(criterion: &mut Criterion) {
    if std::env::var_os("DOKIME_INGEST_PROFILE").is_some() {
        let timings = Timings::default();
        tracing::subscriber::set_global_default(tracing_subscriber::registry().with(timings.clone())).unwrap();
        PROFILE.set(timings).ok().unwrap();
    }
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let rows = std::env::var("DOKIME_INGEST_ROWS").ok().map(|value| value.parse::<usize>().unwrap()).unwrap_or(100_000);
    let mut group = criterion.benchmark_group("sidecar_ingest");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);
    group.throughput(Throughput::Elements(rows as u64));
    for wide in [false, true] {
        for plots in [0, 1, 64] {
            let shape = if wide { "wide16" } else { "scalar" };
            group.bench_function(BenchmarkId::new(shape, plots), |bencher| {
                bencher.iter_custom(|iterations| {
                    let mut elapsed = Duration::ZERO;
                    for _ in 0..iterations {
                        elapsed += executor.block_on(measure(rows, wide, plots));
                    }
                    elapsed
                });
            });
        }
    }
    group.finish();
}

/* === Private Functions === */

async fn measure(rows: usize, wide: bool, plots: u64) -> Duration {
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
    let template_id = format!("obc.benchmarks.ingest_{}{plots}", if wide { "wide" } else { "scalar" });
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
            [if wide { "obc.benchmarks.wide" } else { "obc.benchmarks.scalar" }],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored, rows as i64);
    let arrow = bytes(root.path(), ".arrow");
    let catalog = std::fs::metadata(&database).unwrap().len();
    let wal = std::fs::metadata(database.with_file_name("local.sqlite-wal")).map(|metadata| metadata.len()).unwrap_or(0);
    let mut statement = connection.prepare("SELECT name, sum(pgsize) FROM dbstat GROUP BY name").unwrap();
    let tables: std::collections::BTreeMap<String, i64> =
        statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().collect::<Result<_, _>>().unwrap();
    let logical_bytes = rows * if wide { 128 } else { 8 };
    let timings = PROFILE.get().map(|profile| profile.0.lock().unwrap().clone());
    println!(
        "{}",
        serde_json::json!({"measurement":"sidecar_ingest", "revision":option_env!("GIT_REVISION"),
        "rows":rows, "numeric_fields":if wide {16} else {1}, "plots":plots, "elapsed_s":elapsed.as_secs_f64(),
        "rows_s":rows as f64/elapsed.as_secs_f64(), "logical_mib_s":logical_bytes as f64/1048576.0/elapsed.as_secs_f64(),
        "cpu_ms_100k":(after-before) as f64*100000.0/rows as f64, "arrow_bytes":arrow,"sqlite_bytes":catalog,"wal_bytes":wal,"tables":tables,
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

fn cpu(system: &mut System) -> u64 {
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
    selected.into_iter().filter_map(|pid| system.process(pid)).map(|process| process.accumulated_cpu_time()).sum()
}

/* === Tests === */
