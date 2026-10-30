/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow Telemetry Write Query Finalize And Export Benchmarks

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    sync::Arc,
    time::{Duration, Instant},
};

// Third Party Imports
use criterion::{Criterion, SamplingMode, Throughput};
use futures_util::FutureExt;

// Local Imports
use dokime_domain::model::exports::EvidenceSubject;
use dokime_runtime::runtime::{
    command::{CommandContext, CommandStatus},
    query::{PlotPayloadRequest, RuntimePlotQueryMode},
};
use dokime_runtime::{Runtime, RuntimeHandle};

/* === Constants === */
const QUERY_ROWS: usize = 1_000_000;
const WRITE_ROWS: usize = 1_024;
const CLOSE_ROWS: usize = 65_536;
const EXPORT_ROWS: u32 = 16_384;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Fixture {
    root: tempfile::TempDir,
    owner: Runtime,
    runtime: RuntimeHandle,
}

struct ExportFixture {
    fixture: Fixture,
    subject: EvidenceSubject,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Fixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let owner = Runtime::start(super::plots::config(root.path())).boxed_local().await.unwrap();
        let runtime = owner.handle();
        Self { root, owner, runtime }
    }

    async fn populate(&self, rows: usize) {
        for first in (0..rows).step_by(WRITE_ROWS) {
            self.runtime.publish_telemetry_batch(super::plots::batch((rows - first).min(WRITE_ROWS))).await.unwrap();
        }
        self.runtime.storage().flush().await.unwrap();
    }

    async fn close(mut self) {
        drop(self.runtime);
        self.owner.shutdown().await.unwrap();
        drop(self.owner);
        self.root.close().unwrap();
    }
}

impl ExportFixture {
    async fn new() -> Self {
        let mut fixture = Fixture::new().await;
        fixture
            .owner
            .register(Arc::new(crate::service_support::smoke::Smoke::new(crate::service_support::smoke::Behaviour::new(
                EXPORT_ROWS,
                0,
                false,
            ))))
            .await
            .unwrap();
        let registration =
            fixture.runtime.test_suites().await.into_iter().find(|suite| suite.source == crate::service_support::smoke::SOURCE).unwrap();
        let options = dokime_runtime::definition::CaseMaterializationOptions {
            force: true,
            max_cases: 1_000,
            registration_id: dokime_runtime::model::identity::new_id(),
            first_label_number: 1,
        };
        let mut cases = Vec::new();
        for template in &registration.suite.test_case_templates {
            for mut case in dokime_runtime::definition::materialize_template_cases(
                &registration.suite_id,
                template,
                Default::default(),
                options.clone(),
            )
            .unwrap()
            .test_cases
            {
                case.suite = Some(registration.suite.metadata.clone());
                cases.push(case);
            }
        }
        let cycle = fixture
            .runtime
            .create_cycle(
                dokime_runtime::transactions::CycleDefinition {
                    metadata: registration.suite.metadata,
                    continue_automatically: true,
                },
                CommandContext::default(),
            )
            .await
            .unwrap();
        let executions = cases
            .iter()
            .map(|case| (dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>(), case.test_case_id.clone()))
            .collect::<Vec<_>>();
        let execution = executions[0].0.clone();
        fixture.runtime.register_cases(dokime_runtime::model::identity::new_id::<dokime_runtime::CommandId>(), cases).await.unwrap();
        fixture.runtime.append_executions(cycle.clone(), executions).await.unwrap();
        let receipt = fixture.runtime.start_execution(execution, dokime_runtime::ExecutionMode::Automatic, CommandContext::default()).await;
        assert_eq!(receipt.status, CommandStatus::Accepted);
        let settle = async {
            loop {
                let settled =
                    fixture.runtime.run_request(&receipt.command_id).await.unwrap().is_some_and(|receipt| receipt.summary.phase.terminal());
                let saved = fixture
                    .runtime
                    .cycles()
                    .await
                    .into_iter()
                    .find(|stored| stored.test_cycle_id == cycle)
                    .is_some_and(|stored| stored.test_executions.first().is_some_and(|execution| execution.saved.is_some()));
                if settled && saved {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        };
        let deadline = <dokime_runtime::operations::reports::MaterializeReport as harmos::OperationDeclaration>::POLICY.timeout;
        tokio::time::timeout(deadline, settle).await.expect("benchmark execution finalizes");
        let cycles = fixture.runtime.cycles().await;
        let execution = &cycles.iter().find(|stored| stored.test_cycle_id == cycle).unwrap().test_executions[0];
        assert_eq!(execution.status, dokime_runtime::ExecutionStatus::Passed);
        let subject = EvidenceSubject::Execution(execution.test_execution_id.to_string());
        Self { fixture, subject }
    }
}

/* === Public Functions === */
pub fn benchmark(criterion: &mut Criterion) {
    let executor = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let mut group = criterion.benchmark_group("arrow_telemetry");
    group.sampling_mode(SamplingMode::Flat);
    let mut write = None;
    group.throughput(Throughput::Elements(WRITE_ROWS as u64));
    group.bench_function("write_1024_durable", |bencher| {
        let fixture = write.get_or_insert_with(|| executor.block_on(Fixture::new()));
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let batch = super::plots::batch(WRITE_ROWS);
                    let started = Instant::now();
                    fixture.runtime.publish_telemetry_batch(batch).await.unwrap();
                    fixture.runtime.storage().flush().await.unwrap();
                    elapsed += started.elapsed();
                }
                elapsed
            })
        });
    });
    if let Some(fixture) = write {
        executor.block_on(fixture.close());
    }

    let mut query = None;
    group.throughput(Throughput::Elements(QUERY_ROWS as u64));
    group.bench_function("window_query_1000000", |bencher| {
        let fixture = query.get_or_insert_with(|| {
            executor.block_on(async {
                let fixture = Fixture::new().await;
                fixture.populate(QUERY_ROWS).await;
                fixture
            })
        });
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let request = PlotPayloadRequest {
                        stream: "bench.plot".into(),
                        x_key: Some("timestamp".into()),
                        y_keys: vec!["value".into()],
                        max_points: Some(1_500),
                        window_s: Some(86_400.0),
                        ..Default::default()
                    };
                    let started = Instant::now();
                    let result =
                        fixture.runtime.query_plot_payloads(vec![request], RuntimePlotQueryMode::Snapshot).boxed_local().await.unwrap();
                    elapsed += started.elapsed();
                    assert_eq!(result[0].payload.series[0].raw_points, QUERY_ROWS);
                    assert!(result[0].payload.series[0].points.len() <= 1_500);
                }
                elapsed
            })
        });
    });
    if let Some(fixture) = query {
        executor.block_on(fixture.close());
    }

    group.throughput(Throughput::Elements(CLOSE_ROWS as u64));
    group.bench_function("finalize_and_close_65536", |bencher| {
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let fixture = Fixture::new().await;
                    fixture.populate(CLOSE_ROWS).await;
                    let Fixture { root, mut owner, runtime } = fixture;
                    drop(runtime);
                    let started = Instant::now();
                    owner.shutdown().await.unwrap();
                    elapsed += started.elapsed();
                    drop(owner);
                    root.close().unwrap();
                }
                elapsed
            })
        });
    });

    let mut export = None;
    group.throughput(Throughput::Elements(EXPORT_ROWS.into()));
    group.bench_function("execution_export_16384", |bencher| {
        let fixture = export.get_or_insert_with(|| executor.block_on(ExportFixture::new()));
        bencher.iter_custom(|iterations| {
            executor.block_on(async {
                let mut elapsed = Duration::ZERO;
                for _ in 0..iterations {
                    let started = Instant::now();
                    let mut bundle = fixture.fixture.runtime.export_evidence(fixture.subject.clone()).await.unwrap();
                    let mut bytes = 0;
                    while let Some(batch) = bundle.next_chunk(NonZeroUsize::new(2 * 1024 * 1024).unwrap()).await.unwrap() {
                        bytes += batch.data.len();
                    }
                    elapsed += started.elapsed();
                    assert!(bytes > 0);
                }
                elapsed
            })
        });
    });
    if let Some(fixture) = export {
        executor.block_on(fixture.fixture.close());
    }
    group.finish();
}

/* === Private Functions === */

/* === Tests === */
