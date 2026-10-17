# OBC Charger Testbench

OBC is Dokime's maintained demo and validation plugin. It tests an onboard
charger on a deterministic simulated AC testbench: connect the unit, energize
the supply, inspect startup, measure charging input, trip overcurrent protection,
and recover from power loss. The model produces AC input waveforms, inrush,
phase shifts, and latched faults; each execution owns its device.

## Demonstrated Behaviors

| Suite | Procedure | SDK Behavior |
| --- | --- | --- |
| Bench Signal Check | Verify a deterministic pulse before connecting the charger | Typed streams, criteria, optional scoped system capture |
| Charger Power-Up | Confirm the connection, energize the supply, inspect inrush | Connection listener, typed routes, operator prompts, traceability, artifacts |
| Steady Charging | Observe input voltage and current while charging | Independent periodic and event observers, captured evidence, parameter expansion |
| Overcurrent Protection | Increase the simulated load until protection trips | Ordered load inputs, fault records, evidence-based criteria |
| Power-Loss Recovery | Interrupt the supply, clear the fault, restart | Paired input lists, multiple steps, cleanup and recovery evidence |

The process listener remembers connected bench resources across executions.
Templates ask the plugin's routes for readiness and connections, without naming
or borrowing the plugin. Execution observers publish scoped measurements;
the process system observer publishes monitoring data. Explicit subscriptions
capture host observations for an execution.

## Source Layout

- `src/lib.rs`: the plugin owner, lifecycle, and complete catalogs.
- `src/routes/`, `src/listeners/`, `src/observers/`: bench interaction and process observation.
- `src/suites/`: one module per suite, with typed parameters and shared measurement shape.
- `src/prompts/` and `src/settings.rs`: operator checkpoints and bench configuration.
- `runtime/`: the embedded Harmos bench application and deterministic device model.
- `tests/suites/`: public lifecycle, cancellation, observer shutdown, and evidence checks.
- `tests/acceptance/`: the built executable's inventory and live dispatch.

All charger streams and records use the `mini.charger` namespace. `mini.pulse`
is the bench signal check, and `obc-plugin.system` is process monitoring.
`images_directory` optionally supplies a startup reference image. Operator and
host identity are captured as traceability, not configured as template inputs.

## Build And Validate

```sh
mise run build:plugins:dev
mise run test:integration:rust
mise run test:acceptance:rust
```

The bundle is `target/debug/plugins/obc-plugin.dokime`. Install it through
**Settings → Plugins** and restart to admit its frozen catalog. The screenshot
and native desktop scenarios build and install this same bundle.

For a focused plugin check:

```sh
mise exec -- cargo nextest run --locked -p obc-plugin -p obc-runtime --profile integration
```

The plugin stops execution observers before cleanup, settles its embedded
Harmos runtime, and releases process resources. Shutdown, cancellation, and
prompt cancellation are validated as well as successful measurements.
