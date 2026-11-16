# OBC Charger Testbench

Source: `plugins/sidecars/obc/`. Paths below are relative to it.

OBC is Dokime's maintained demonstration and validation plugin. It tests an
onboard charger on a deterministic simulated AC testbench: energize the supply,
inspect start-up, measure steady charging input, trip overcurrent protection,
recover from power loss, and capture a ten-module charger at scale. The model in
`src/device.rs` produces AC input waveforms, inrush, phase lag and latched faults;
each execution owns its charger through the `Testbench` resource in
`src/testbench.rs`. The plugin is authored with the [Rust SDK](rust-sdk.md) and
served over gRPC like the SDK examples.

## Suites

| Suite | Template | Procedure |
| --- | --- | --- |
| OBC Checks | `obc.checks.pulse` | A deterministic pulse with an optional Yes/No confirmation before capture |
| Charger Start-Up | `obc.charger.startup.sequence` | Energize, measure the inrush, inspect the powered charger, observe settling, attach evidence |
| Charger Steady State | `obc.charger.steady.measurement` | Sweep AC voltage and load across cases; traceability, hold events and operator acceptance |
| Charger Protection | `obc.charger.protection.overcurrent` | Step the load ladder until the overcurrent latch opens; a latched charger refuses to restart |
| Charger Recovery | `obc.charger.recovery.power` | Replay paired outage durations and recovery limits; clear the latch and restart each time |
| Bench | `obc.bench.module_array` | Ten power modules with ten channels each and the exact revenue meter register |

Samples come from observers that answer the host's interval windows. Steps change
the charger (energize, load, fault, clear) and advance host time with `ctx.sleep`;
each sample advances the model by the template's Simulation Sample Interval, or by
the sample interval itself when **Pace Samples in Real Time** is set. The charger
suites keep one-sample observers because each sample depends on the state steps
set between samples. The Bench module array answers whole windows: each module
computes sample `i` of a window from the charger at that sample's model time, and the
meter advances the model by the window's sample count. The observer cadence is
the plugin setting **Sample Interval** (1 ms by default). Under a controlled host
clock a run is exact: the default start-up case saves 20 samples, 10 before the
inspection prompt. **Observe Until Stopped** keeps the start-up measurement
running until a host stop or cancellation ends it.

The start-up document covers the ten plot kinds. Its Signal Maps section
`obc.charger.startup.sequence.maps` pairs a sparse input-power contour (mean
reduction, filled bands) with the Electrical Trajectory and the Channel Survey
grid: the bench scans eight heatsink sensors on four power modules every sweep,
so the grid plays back each sensor's rise above ambient as the charger warms. Phase Breakdown
shows pie/donut and Pareto options, and Distributions includes the frequency
series. The screenshot journey in `crates/dokime-gui/e2e/screenshots.mjs` seeks,
replays and pauses these controls.

The Bench suite carries the scale and exactness workloads. Its document holds
one hundred time-series plots, ten channels for each of ten modules, and eleven
summary tiles (Count, Sum, Mean, Median, Min, Max, First, Last, P05, P95 and P99)
over the meter register `1152921504606847975`, read every sample. That value
exceeds every floating-point integer, so the tiles prove exact integer
statistics end to end; the sum exceeds 64 bits and stays exact.

`obc.device.read` and `obc.device.set_power` are Public operations controlling
one demonstration charger outside any execution. `obc.analysis.ripple` gives any
plotted trace its peak-to-peak ripple. Start-up and steady-state attach their
JSON summaries as execution artifacts; start-up optionally attaches
`reference.png` from the **Reference Images Directory** setting.

## Not Restored

- File capture of the charger log: the host's file capture operations are
  unimplemented and the owner deferred capture.
- Host telemetry sections and the CPU load template: the host plugin that
  supplied them no longer exists.
- Waiting for a unit connection record: the host has no record listeners.
- The feature-gated ingest benchmark templates: step-pushed samples have no SDK
  equivalent; the Bench suite carries the scale workloads instead.
- The status, connection-list and readiness operations and the process system
  observer: they served the removed settings status card, connection listener and
  host telemetry; procedures now read their bench directly.
- The Warning level on inspection prompts: the SDK prompt builder sets no level.

## Source Layout

- `src/lib.rs`: the complete plugin composition.
- `src/device.rs`, `src/testbench.rs`: the charger model and its execution resource and observers.
- `src/measurement.rs`, `src/records.rs`, `src/prompts.rs`: rows, record messages and operator checkpoints.
- `src/suites/`: one file per suite, each owning its streams, steps, criteria and document.
- `src/operations.rs`, `src/analyses.rs`, `src/settings.rs`, `src/evidence.rs`: manual device, ripple, settings, artifacts and traceability.
- `tests/integration/`: the charger model and the catalog's document bindings.

## Build and Validate

```sh
mise run build:host-fixtures -- --language rust
mise run build:desktop-fixtures
mise run test -- --package plugin-obc --target integration
mise run test -- --package dokime-runtime --target system --exact client_runs_packaged_obc_plugin::client_runs_packaged_obc_plugin
```

The host fixtures place `plugin-obc` beside the SDK fixtures, and the desktop
fixtures bundle it as `fixtures/desktop/plugin-obc.dokime` with the same
`dokime plugin bundle` command the product build uses. The system story installs
that bundle through a verified managed upload, runs the start-up sequence and the
module array bench, and checks their records and exact statistics.

The product build stages `bin/plugin-obc.dokime`; packaging copies it beside the
executables, and the release publishes it in the `dokime-plugins-<version>-<triple>`
archive with its entry in the release index's plugin feed, as declared under
`release.plugins` in `tools/project.toml`. Install it through **Settings → Plugins**.
