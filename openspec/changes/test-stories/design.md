# Test Stories Design

Status: agreed on 2026-11-10; not implemented. Locations are from two
read-only audits of the `dev` tree and are rechecked when each task starts.
`S` is `crates/dokime-server`, `R` is `crates/dokime-runtime`, `G` is
`crates/dokime-gui`.

## Shape of a Story

```rust
#[tokio::test(start_paused = true)]
async fn operator_runs_a_case_and_reads_the_report() -> Result<(), TestError> {
    // Start a backend with one plugin
    let mut backend = Backend::start(Config::local()).await?;
    let plugin = backend.connect_plugin(thermal::plugin()).await?;

    // Run one case
    let cases = backend.materialize("thermal.cycle", parameters! { "limit" => 85 }).await?;
    let cycle = backend.start_cycle(cases).await?;
    cycle.finished().await?;

    // The verdict and the report follow from the evidence
    assert_eq!(cycle.verdict().await?, Verdict::Passed);
    assert!(backend.report(&cycle).await?.contains("Temperature Within Limit"));

    // Stop
    plugin.stop().await?;
    backend.shutdown().await
}
```

## Shared Modules

| Module | Replaces |
| --- | --- |
| Backend lifecycle | Runtime, service, listener and client tuples assembled locally, for example `S/tests/integration/grpc/backend/start.rs:153` |
| Wire calls | Typed calls, stream decoding and authentication in `S/tests/support/wire.rs` and `G/src-tauri/tests/support/wire.rs`, which overlap |
| Providers and catalogs | Fake providers in `S/tests/support/mod.rs:122,141` and `S/tests/integration/grpc/backend/mod.rs:80` |
| Events and storage readback | Polling loops and readback helpers written per test |
| Connection fixtures | `G/src-tauri/tests/integration/connection/fixture.rs`, which is shared support placed inside a test directory |

## Driver Files to Split

| File | Problem |
| --- | --- |
| `S/tests/integration/grpc/catalog/mod.rs:51` | Dispatches separate scenarios with no single actor |
| `S/tests/integration/grpc/remote/mod.rs:67` | Bundles secrets, binding, encryption, certificate refresh and retries; rewritten with `remote-access-simplification` |
| `S/tests/integration/grpc/content/transfer.rs:857,894,940` | Three upload outcomes rebuilt separately; one story on one host |
| `R/tests/integration/execution/mod.rs:86`, `S/tests/integration/grpc/backend/mod.rs:990` | Wrappers that combine scenarios without sharing their fixtures |

## Workloads That Move to Benchmarks

| Test | Workload | Keeps as a Test |
| --- | --- | --- |
| `S/tests/system/artifact_transfer.rs:38` | 300 MiB in 1 MiB chunks | A multi-frame upload of a few kilobytes |
| `R/tests/integration/plugins/store/installation.rs:43` | A 70 MiB archive | A small archive against a configurable limit |
| `S/tests/integration/grpc/content/transfer.rs:834` | 8 MiB | A deliberately multi-frame small upload |
| `S/tests/system/plot_admission.rs:48` | 40,000 rows | The smaller case in `S/tests/integration/grpc/telemetry/plots.rs:109` |
| `R/tests/system/execution_sampling.rs:28..80` | 20,000 rows, 8,192 spectrum rows | The acceptance counterparts using the same helpers |
| `R/tests/system/evidence_durability.rs:47`, `managed_durability.rs:47`, `retained_evidence.rs:47`, `telemetry_plot_history.rs:49` | 10,080 rows; 50,000 samples | The integration counterparts using the same helpers |

Slowest recorded tests to shorten by sharing startup: storage recovery
requests (128 s), plugin sealing (73 s), hot reload and replacement (68 s
each), stop and report retry (54 s). The figures are from older timing
reports and are measured again in task 1.1.

## Real Time

Replaced by the virtual or paused clock, or by an awaited event:

| Location | Dependency |
| --- | --- |
| `S/tests/integration/host.rs:78,91` | Retries every 10 ms for up to 10 s |
| `S/tests/integration/grpc/backend/start.rs:152` | Elapsed-time loop past the client idle timeout |
| `S/tests/integration/grpc/execution/records.rs:275,298` | A 250 ms listener expiry |
| `S/tests/system/plugin.rs:750` | Deadline loops with 25 ms and 50 ms sleeps |
| `R/tests/integration/plugin/demand.rs:530` | A 20 ms sleep before an assertion |
| `R/tests/integration/plugin/resources.rs:647` | A settle sleep before a rewrite |
| `R/tests/integration/plugin/sdk_error.rs:162` | A 50 ms listener expiry |
| `R/tests/integration/domains/telemetry/sampling.rs:196,246` | A 10 ms sleep; stop raced against a timeout |
| `R/tests/integration/runtime/storage/artifacts.rs:150` | A 10 ms sleep |
| `R/tests/integration/execution/report_cancellation.rs:159` | A 5 ms polling sleep |
| `G/src-tauri/tests/acceptance/native_session.rs:167` | Asserts less than 4 s elapsed |
| `G/src-tauri/tests/integration/connection/client.rs:107,134` | A 100 ms sleep after a 10 ms timeout |
| `G/tests/integration/cycle/document-query.test.ts:91,179`, `plots/nominal.test.ts:215,420`, `shell/status.test.ts:176` | Real timers and `Date.now()` |

Wall-clock reads that only stamp fixtures become fixed timestamps, including
the roughly fifty in `R/tests/integration/execution/cycles.rs` and the shared
factories in `R/tests/support/services/evidence.rs:245,331,347`.

Kept as guards only: bounds on real processes and browsers, such as
`S/tests/system/plugin/shutdown.rs:82`, `S/tests/system/remote/interruption.rs:630`
and the 30 s browser guards. Each waits for its event first.

## Tests Removed

| Test | Reason |
| --- | --- |
| `R/tests/integration/execution/workflow/simulation.rs:45` | Constructs a simulated runtime and drops it |
| `R/tests/integration/execution/workflow/definition.rs:97` | Checks only a generated identifier and counts |
| `R/tests/integration/plugin/plugin_settings.rs:322,373` | Echo authored labels and names |
| `R/tests/integration/plugin/analysis.rs:231` | Checks declaration metadata; no analysis runs |
| `R/src/parameters.rs:504` | Checks generated field names; folded into parameter decoding |
