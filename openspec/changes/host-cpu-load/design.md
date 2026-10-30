# Host CPU Load Design

## Locked Contract

```rust
pub struct LoadHold { pub target: LoadTarget, pub threads: Option<u16> }
pub enum LoadTarget { Percent(f64), Flops(f64) }
pub struct LoadSummary {
    pub target: LoadTarget,
    pub mean: f64,
    pub error_rms: f64,
    pub duration_s: f64,
}
```

`host.cpu.load.hold` is Public. Finite percent targets are in `(0, 100]`;
finite FLOP/s targets are positive. Zero threads is refused; absent threads uses logical cores.
Targets beyond available capacity saturate rather than claiming convergence.
Mean and RMS error use the selected variable's units, weighted by measured interval duration.

## Ownership And Sampling

The existing startup all-core calibration remains the sole capacity measurement.
Calibration and duty workers share eight f64 FMA accumulators, counting two operations per FMA.
A plugin-owned sampler refreshes CPU deltas at configured cadence and caches the latest load.
`host.cpu.load`, `host.system.collect` and the controller read that cache without refreshing CPU state.
Memory, temperatures and other on-demand system probes retain existing collection behavior.

## Control And Evidence

One native worker per selected thread repeats a configured busy/sleep slice.
Workers count completed kernel batches, observe stop between batches and wake from sleep on stop.
A discrete PI controller integrates normalized error with a clamped integrator;
each worker receives duty clamped to `[0, 1]`. Percent feedback includes all host processes;
FLOP/s feedback counts only generator work over actual elapsed wall time.
All controller timing, gains, bounds and settlement deadlines belong to one documented configuration.
The operation policy owns the long-running deadline; retries are disabled.
`host.cpu.load.control` publishes target, measured variable, duty and generator FLOP/s each period.
Separate nullable percent/FLOP/s columns keep structured units unambiguous within one stream.

## Lifecycle

Cancellation must stop and join every worker before returning `LoadSummary`.
Dropping a running attempt must also stop workers. Plugin shutdown stops sampling and all workloads.
Dokime's existing step/run handle sweep cancels unjoined operations before cleanup seals evidence.
No effect-journal inverse is needed: only live process work is created; stopping restores it,
and process death cannot leave CPU generation running or durable native state altered.

## Dependency Decision

At pinned Harmos `9287b860842b1e33b7e6421a00780b8a33f3fe20`, `Context` has no cancellation signal.
The sidecar reply loop drops the attempt future on cancellation and sends Finished without a payload.
A local drop guard can restore CPU use but cannot return the locked summary to the caller.
The pinned portable traits also omit `u16`: PostcardValue, MemorySize and Described are absent,
so the exact locked `Option<u16>` request needs primitive support upstream as well.
Upstream cooperative cancellation or an explicitly approved summary-evidence contract is required.
Implementation must not claim drop cleanup proves cancellation returns a summary.

## Validation Boundary

Commit-only delivery uses the explicitly requested check-only proof: workspace check, formatting,
focused host tests, affected OBC tests and strict OpenSpec validation. No full lane, push or PR.
