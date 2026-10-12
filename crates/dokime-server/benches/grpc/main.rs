/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::env;
use std::time::{Duration, Instant};
// Third Party Imports
use anyhow::{Context, Result, bail};
use criterion::{Criterion, criterion_group, criterion_main};
use serde::Serialize;
use tokio::task::JoinSet;
use tonic::{Request, transport::Channel};
// Local Imports
use dokime_protocol::v1::{self, observation_client::ObservationClient};

/* === Constants === */
const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:5173";
const DEFAULT_REQUESTS: usize = 1_000;
const DEFAULT_CONCURRENCY: usize = 64;
const DEFAULT_WARMUP_REQUESTS: usize = 10;
const DEFAULT_PAGE_ITEMS: u32 = 50;
const DEFAULT_P95_BUDGET_MS: f64 = 100.0;
const DEFAULT_TOTAL_BUDGET_MS: f64 = 20_000.0;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RUNTIME_SHELL_METHOD: &str = "/dokime.v1.Observation/GetRuntimeShell";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug)]
pub(crate) struct Args {
    endpoint: String,
    bearer_token: Option<String>,
    requests: usize,
    concurrency: usize,
    warmup_requests: usize,
    page_items: u32,
    p95_budget_ms: f64,
    total_budget_ms: f64,
}

#[derive(Debug, Serialize)]
struct ProbeSummary {
    endpoint: String,
    method: &'static str,
    page_items: u32,
    requests: usize,
    concurrency: usize,
    warmup_requests: usize,
    errors: usize,
    total_ms: f64,
    requests_per_second: f64,
    min_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    max_ms: f64,
    mean_ms: f64,
    p95_budget_ms: f64,
    total_budget_ms: f64,
    within_budget: bool,
}

#[derive(Debug)]
struct ProbeSample {
    duration_ms: f64,
    ok: bool,
    error: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
fn api_benchmark(criterion: &mut Criterion) {
    // The Backend Is An Explicit Benchmark Prerequisite
    let args = Args {
        endpoint: default_endpoint(),
        bearer_token: env::var("DOKIME_DEV_BEARER_TOKEN").ok(),
        requests: DEFAULT_REQUESTS,
        concurrency: DEFAULT_CONCURRENCY,
        warmup_requests: DEFAULT_WARMUP_REQUESTS,
        page_items: DEFAULT_PAGE_ITEMS,
        p95_budget_ms: DEFAULT_P95_BUDGET_MS,
        total_budget_ms: DEFAULT_TOTAL_BUDGET_MS,
    };
    assert_eq!(percentile(&[10.0, 20.0, 30.0, 40.0], 50.0), 25.0);
    assert_eq!(percentile(&[10.0, 20.0, 30.0, 40.0], 95.0), 38.5);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("native benchmark runtime");
    let mut group = criterion.benchmark_group("api");
    group.sample_size(10);
    group.bench_function("runtime_shell", |bencher| {
        bencher.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let summary = runtime
                    .block_on(run_probe(&args))
                    .expect("native API workload");
                print_summary(&summary);
                assert!(
                    summary.within_budget,
                    "native API workload exceeded its latency/error budget"
                );
                elapsed += Duration::from_secs_f64(summary.total_ms / 1_000.0);
            }
            elapsed
        });
    });
    group.finish();
}

criterion_group!(benches, api_benchmark);
criterion_main!(benches);

/* === Private Functions === */
async fn run_probe(args: &Args) -> Result<ProbeSummary> {
    let endpoint = args.endpoint.trim_end_matches('/').to_owned();
    let client = tokio::time::timeout(
        REQUEST_TIMEOUT,
        ObservationClient::connect(endpoint.clone()),
    )
    .await
    .context("native gRPC benchmark connection timed out")?
    .context("failed to connect native gRPC benchmark client")?;

    for _ in 0..args.warmup_requests {
        let sample =
            get_runtime_shell(client.clone(), args.page_items, args.bearer_token.clone()).await;
        if !sample.ok {
            bail!(
                "native gRPC benchmark warmup failed: {}",
                sample
                    .error
                    .unwrap_or_else(|| "unknown request error".to_owned())
            );
        }
    }

    run_probe_windowed(args, client, endpoint).await
}

async fn run_probe_windowed(
    args: &Args,
    client: ObservationClient<Channel>,
    endpoint: String,
) -> Result<ProbeSummary> {
    let started = Instant::now();
    let mut samples = Vec::with_capacity(args.requests);
    let mut requests = JoinSet::new();
    for _ in 0..args.requests {
        let client = client.clone();
        let bearer_token = args.bearer_token.clone();
        let page_items = args.page_items;
        requests.spawn(async move { get_runtime_shell(client, page_items, bearer_token).await });
        if requests.len() >= args.concurrency {
            samples.push(
                requests
                    .join_next()
                    .await
                    .expect("join set cannot be empty")
                    .context("native gRPC benchmark request task panicked")?,
            );
        }
    }
    while let Some(sample) = requests.join_next().await {
        samples.push(sample.context("native gRPC benchmark request task panicked")?);
    }
    Ok(summarize(
        args,
        endpoint,
        samples,
        started.elapsed().as_secs_f64() * 1_000.0,
    ))
}

async fn get_runtime_shell(
    mut client: ObservationClient<Channel>,
    page_items: u32,
    bearer_token: Option<String>,
) -> ProbeSample {
    let started = Instant::now();
    let mut request = Request::new(v1::GetRuntimeShellRequest {
        limit: page_items,
        ..Default::default()
    });
    if let Some(token) = bearer_token {
        let value = match format!("Bearer {token}").parse() {
            Ok(value) => value,
            Err(error) => {
                return ProbeSample {
                    duration_ms: started.elapsed().as_secs_f64() * 1_000.0,
                    ok: false,
                    error: Some(format!("invalid bearer token: {error}")),
                };
            }
        };
        request.metadata_mut().insert("authorization", value);
    }

    let result = tokio::time::timeout(REQUEST_TIMEOUT, client.get_runtime_shell(request)).await;
    let duration_ms = started.elapsed().as_secs_f64() * 1_000.0;
    match result {
        Ok(Ok(_)) => ProbeSample {
            duration_ms,
            ok: true,
            error: None,
        },
        Ok(Err(error)) => ProbeSample {
            duration_ms,
            ok: false,
            error: Some(format!("gRPC {error}")),
        },
        Err(_) => ProbeSample {
            duration_ms,
            ok: false,
            error: Some(format!(
                "gRPC request timed out after {} seconds",
                REQUEST_TIMEOUT.as_secs()
            )),
        },
    }
}

fn summarize(
    args: &Args,
    endpoint: String,
    samples: Vec<ProbeSample>,
    total_ms: f64,
) -> ProbeSummary {
    let mut latencies: Vec<f64> = samples.iter().map(|sample| sample.duration_ms).collect();
    latencies.sort_by(f64::total_cmp);
    let errors = samples.iter().filter(|sample| !sample.ok).count();
    let p95_ms = percentile(&latencies, 95.0);
    let within_budget =
        errors == 0 && p95_ms <= args.p95_budget_ms && total_ms <= args.total_budget_ms;
    let mean_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;

    ProbeSummary {
        endpoint,
        method: RUNTIME_SHELL_METHOD,
        page_items: args.page_items,
        requests: args.requests,
        concurrency: args.concurrency.min(args.requests),
        warmup_requests: args.warmup_requests,
        errors,
        total_ms,
        requests_per_second: args.requests as f64 / (total_ms / 1_000.0),
        min_ms: *latencies
            .first()
            .expect("benchmark has at least one request"),
        p50_ms: percentile(&latencies, 50.0),
        p95_ms,
        p99_ms: percentile(&latencies, 99.0),
        max_ms: *latencies
            .last()
            .expect("benchmark has at least one request"),
        mean_ms,
        p95_budget_ms: args.p95_budget_ms,
        total_budget_ms: args.total_budget_ms,
        within_budget,
    }
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let rank = (values.len() - 1) as f64 * percentile / 100.0;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        return values[lower];
    }
    values[lower] * (upper as f64 - rank) + values[upper] * (rank - lower as f64)
}

fn print_summary(summary: &ProbeSummary) {
    println!(
        "Native gRPC performance probe: {} {} (page_items={})\nrequests={} concurrency={} warmup={} errors={}\ntotal={:.1} ms throughput={:.1} req/s\nlatency min/p50/p95/p99/max={:.1}/{:.1}/{:.1}/{:.1}/{:.1} ms mean={:.1} ms\nbudgets p95<={:.1} ms total<={:.1} ms within_budget={}",
        summary.endpoint,
        summary.method,
        summary.page_items,
        summary.requests,
        summary.concurrency,
        summary.warmup_requests,
        summary.errors,
        summary.total_ms,
        summary.requests_per_second,
        summary.min_ms,
        summary.p50_ms,
        summary.p95_ms,
        summary.p99_ms,
        summary.max_ms,
        summary.mean_ms,
        summary.p95_budget_ms,
        summary.total_budget_ms,
        summary.within_budget,
    );
}

fn default_endpoint() -> String {
    env::var("DOKIME_DEV_ENDPOINT")
        .or_else(|_| env::var("DOKIME_DEV_BASE_URL"))
        .unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned())
}

/* === Tests === */
