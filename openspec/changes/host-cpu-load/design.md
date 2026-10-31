# Host CPU Load Design

## Locked Contract

```rust
pub struct LoadHold { pub target: LoadTarget, pub threads: Option<u32> }
pub enum LoadTarget { Percent(f64), Flops(f64) }
pub struct LoadSummary {
    pub target: LoadTarget,
    pub mean: f64,
    pub error_rms: f64,
    pub duration_s: f64,
    pub samples: u64,
}
```

`host.cpu.load.hold` is Public. Finite percent targets are in `(0, 100]`;
finite FLOP/s targets are positive. Thread counts outside `1..=logical_cores` are refused; absent threads uses logical cores.
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
One Evidence catalog combines collected streams with control rows: the contribution namespace
is unique, while collection operations retain their existing executable catalog.

## Lifecycle

Cancellation must stop and join every worker before returning `LoadSummary`.
Dropping a running attempt must also stop workers. Plugin shutdown stops sampling and all workloads.
Dokime's existing step/run handle sweep cancels unjoined operations before cleanup seals evidence.
No effect-journal inverse is needed: only live process work is created; stopping restores it,
and process death cannot leave CPU generation running or durable native state altered.

## Dependency Decision

Harmos `4e2669fa` supplies `Context::cancelled()` and typed final cancellation responses.
The owner approved portable `Option<u32>` instead of unsupported `u16` on resume.
The handler observes cancellation while waiting, controlling and publishing; worker drop joins
all native threads even if the attempt is forcibly dropped. The policy grace is owned by the
configuration module; the configurable settlement budget must fit that grace.
Summary `samples` counts measured intervals, including the final partial interval.

## Validation Boundary

Commit-only delivery uses the explicitly requested check-only proof: workspace check, formatting,
focused host tests, affected OBC tests and strict OpenSpec validation. No full lane, push or PR.
