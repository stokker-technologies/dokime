# OBC Charger Testbench

OBC is Dokime's maintained demo and validation plugin. It tests an onboard
charger on a deterministic simulated AC testbench: connect the unit, energize
the supply, inspect start-up, measure charging input, trip overcurrent protection,
and recover from power loss. The model produces AC input waveforms, inrush,
phase shifts, and latched faults; each execution owns its device.

## Demonstrated Behaviors

| Suite | Procedure | SDK Behavior |
| --- | --- | --- |
| OBC Checks | Verify a deterministic pulse before connecting the charger | Typed streams, criteria, optional confirmation and scoped system capture |
| Charger Start-Up | Confirm the connection, energize the supply, inspect inrush | Connection listener, typed operations, operator prompts, traceability, artifacts |
| Charger Steady State | Observe input voltage and current while charging | Independent periodic and event observers, captured evidence, parameter expansion |
| Charger Protection | Increase the simulated load until protection trips | Ordered load inputs, fault records, evidence-based criteria |
| Charger Recovery | Interrupt the supply, clear the fault, restart | Paired input lists, multiple steps, cleanup and recovery evidence |

The process listener remembers connected bench resources across executions.
Templates ask the plugin's operations for readiness and connections, without
naming or borrowing the plugin. The settings card reads `get_status` live, and
the `obc-plugin.ripple` analysis gives any plotted trace its peak-to-peak ripple. Execution observers publish scoped measurements;
the process system observer publishes monitoring data. Explicit subscriptions
capture host observations for an execution.

The remote demonstration uses `obc.checks.pulse`. It runs without a prompt by
default. Set `require_confirmation` to `true` for one Yes/No confirmation before
capture begins; the request identifies its execution and prompt. No pulse
samples are produced while waiting. Declining fails the execution, and
cancellation preserves its collected evidence through normal sealing. The
charger suites retain their separate inspection forms.

For manual remote control, `obc.device.read` accepts `{}` and reports
`enabled`, `voltage_rms_v` and an optional `fault`. `obc.device.set_power`
accepts `{"enabled": true}` or `{"enabled": false}` and returns the observed
state after changing power. Both declare gRPC bindings in `obc.device.v1.Device`
(`Read` and `SetPower`), so CLI and Python discovery can invoke them remotely.
The manual device starts off and reports 230 V RMS
when enabled. A stopped bench refuses changes; a latched fault refuses power-on.
Shutdown powers it down. These operations control one demonstration device in
the plugin's embedded runtime; charger executions continue to own separate
devices. Manual operations are not recorded in execution evidence.

## Source Layout

- `src/lib.rs`: the plugin owner, lifecycle, and complete catalogs.
- `src/domain/`: what the plugin declares, kind first and domain second: `model/` for the replies and `operations/` for the requests it serves, each request its own operation.
- `src/operations/`: each request's `impl Operation`, reaching the bench through its declared `Resources`.
- `src/listeners/`, `src/observers/`: record listeners and process observation.
- `src/analyses/`: the plot analyses the plugin contributes.
- `src/suites/`: the `Suites` catalog and one self-contained folder per suite, each template file declaring its own parameters; `src/suites/common/` holds the shared measurement shape, bench parameter rules, host system section and verdict helpers.
- `src/prompts/` and `src/settings.rs`: operator checkpoints and bench configuration.
- `runtime/`: the embedded Harmos bench application and deterministic device model.
- `tests/integration/`: in-process lifecycle, confirmation, observer shutdown, and evidence checks.
- `tests/acceptance/`: the built executable's inventory and live dispatch.

All charger streams and records use the `obc.charger` namespace. `obc.pulse`
is the OBC Checks pulse, and `obc-plugin.system` is process monitoring.
`images_directory` optionally supplies a startup reference image. Operator and
host identity are captured as traceability, not configured as template inputs.

## Build And Validate

```sh
mise run build --profile dev
mise run test:integration:rust
mise run test:acceptance:rust
```

The bundle is `target/build/dev/<target>/plugins/obc-plugin.dokime`. Install it through
**Settings → Plugins** and restart to admit its frozen catalog. The screenshot
and native desktop scenarios consume this same built bundle.

For a focused plugin check:

```sh
mise exec -- cargo check --locked -p obc-plugin --all-targets
```

The plugin stops execution observers before cleanup, settles its embedded
Harmos runtime, and releases process resources. Shutdown, cancellation, and
prompt cancellation are validated as well as successful measurements.
