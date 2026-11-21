# Dokime Python SDK

`dokime-sdk` provides typed plugin declarations and async clients for a running
Dokime host. It requires Python 3.13 or later. Resources close explicitly.

## Operation Lanes

`Lane` is re-exported by `dokime_sdk`. Declare `instrument = Lane("instrument", serial=True)`, list it in the plugin class's `lanes = [instrument]`, and select it with `@operation("example.measure", lane=instrument)`. `Lane("workers", concurrent=4)` bounds concurrency; `Lane("independent")` is unbounded. The library validates declarations and rejects an operation whose lane the plugin does not declare.

## Public Homes

```python
from dokime_sdk import Plugin, ResultStatus, Suite, connect
from dokime_sdk.operations import Context, Visibility, operation
from dokime_sdk.templates import TestTemplate, criterion, record, step
```

The operation and template modules each own a distinct `Context`. Import a
context from its module when authoring that kind of handler. Plugin handlers
reach shared plugin state through `ctx.plugin`.

`ExamplePlugin().identifier` is `example` before and after registration. A class
`identifier` overrides the class-derived identity.

## Public Peer Operations

Create `Operation(identifier, Request, Response, Visibility.PUBLIC)` at module
scope from the callee's generated message types, or import a typed reference
published by its package. Make the reference available before constructing the
plugin. Own and peer operations use the same bound call:

```python
from peer_package.operations import increment


@step("Increment")
async def increment_peer(self, ctx):
    result = await ctx.operation(increment)(41)
```

The SDK freezes requirements automatically from handler references, including
helpers, closures, referenced module/class members and reference collections.
Authors declare no requirements. Conflicting contracts fail at construction.
An operation constructed or supplied only after activation requires rebuilding
and registering the plugin with its typed reference available beforehand.
The callee must expose the operation as Public. The host preserves execution
scope and applies the callee's lane, policy, deadline and cancellation. A bound
call accepts the existing optional `policy=` override for that call alone.

Catch `dokime_sdk.operations.FailureError` around a bound call and inspect
`error.failure.classification`, `code`, `message`, `action`, `operation`, `plugin`
and `cause`. These are product failure values, including cancellation and timeout
classifications; the library's exception does not escape a bound peer call.

Read a decorated function's public contract with `declaration(increment)` from
`dokime_sdk.operations`. It returns the immutable `Operation` with its identifier,
request and response Python types, visibility and policy. Publish that value for
peer consumers to pass to `ctx.operation(...)`.

## Declaration Builders

```python
Parameter.text("serial", "Serial").required().build()
Parameter.list("channels", "Channels", int).default([1, 2]).items(1, 8).build()
Parameter.choice("mode", "Mode", ["fast", "slow"]).default("fast").build()
reading = record(int, identifier="reading")
Section("Readings").repeat(reading.field(), order_by=reading.field().max()).traceability()
time_series(field("measurement.samples", "value")).markers("reading")
```

Import these authoring values from `dokime_sdk.templates`. Document record fields
require an explicit record identifier; template binding qualifies local record
references and markers. Setting bounds and optional observer fields
need upstream library contracts; typed stream subscription remains unavailable at
the current library pin. Stop-completes-run has no current product declaration.

## Starter

[The starter](examples/starter/) contains one uv project with an installed
`src/starter_project/` package. `dokime new <name> --lang python` substitutes the
project name and its normalized Python package name. There are no root entry
scripts, argument parsing or shipped lockfile. The user's first `uv sync`
creates the lock.

```sh
uv sync
uv run python -m unittest
uv run python -m starter_project.plugin
uv run python -m starter_project
```

The starter calls `await plugin.serve()`. Launch credentials
(`HARMOS_HOST_ENDPOINT` and `HARMOS_LAUNCH_TOKEN`) select the library's authenticated
announcement; otherwise `DOKIME_REMOTE` selects self-registration, and an absent
remote selects standalone serving. These paths do not print readiness output.
Programmatic `client.plugins.register(plugin, address=...)` owns the listener.
Closing that client stops its listener and retains the host entry. Use
`unregister()` explicitly to remove an entry.

## Observer Records

Declare records on the plugin class for plugin observers, or on a template for
template handlers. Both contexts use the same publication method. Publication
inherits the active execution scope; callers cannot select another execution.

```python
class MeasurementPlugin(Plugin):
    version = "1.0.0"
    reading = record(int)

    @stream("Measurements", row=int)
    @observer()
    async def measurements(self, ctx):
        await ctx.record(self.reading, 42)
        return 42
```

The plugin version comes from its
owning installed distribution, including editable installs; a class `version`
overrides that metadata. Source-only plugins must declare a class version.
The package module connects to the host,
registers the plugin, creates a cycle, materializes its template classes and runs
each execution. It prints progress and sealed outcomes, seals the cycle and
writes its evidence archive under `_output/`. Operator questions remain pending
until answered or the execution is stopped. Answer them in the desktop or with
`dokime execution prompt answer EXECUTION --action 0 --accepted`.
`dokime execution list --json` lists all cycles; each execution ID is at
`executions[i].summary.execution_id`. Add `--cycle CYCLE` to narrow the list.
`mise run test` checks the declared suites, template, steps, criterion
and operations without starting a product host.

Set `DOKIME_REMOTE` to an HTTP origin or `.pair` file in the process environment
or working-directory `.env`. Explicit connection arguments take precedence, then
the process environment, then `.env`. Listeners bind `127.0.0.1:0` unless a
programmatic registration call supplies an address. Keep secrets in Secret Storage.

## Bound Client Flow

```python
client = await connect(connect_timeout=30.0)
try:
    await client.plugins.register(ExamplePlugin())
    cycle = await client.cycle(label="Capture")
    cases = await client.cases.materialize(ConfirmationTestTemplate)
    for case in cases:
        execution = await cycle.execution(case)
        async for progress in execution.run():
            print(progress.phase)
        outcome = await execution.seal()
        print(outcome.status == ResultStatus.PASSED)
    await cycle.seal()
    evidence = await cycle.evidence()
finally:
    await client.close()
```

`execution.start()` and `execution.watch()` also remain available separately.
Closing a client stops its listeners, retains their host entries and closes its connection.
Execution sealing and sealed progress require the corresponding host contract;
the SDK reports unsupported contracts explicitly.

## Examples and Repository Proof

- [Starter](examples/starter/): confirmation suite, public echo operation and client flow.
- [Thermal](examples/thermal/): parameterized procedure, records, observers and window sampling.
- [Client](examples/client/src/client_example/__main__.py): browse bound cycle handles.

```sh
mise -C sdk/python run prepare
mise -C sdk/python run format
mise -C sdk/python run check
mise -C sdk/python run test -- --layer unit
mise -C sdk/python run example-test
```

The check task verifies formatting, lint, types, strict authored/generated stubs
and the three intentional public homes. Generated wire modules remain private
and tracked. Regenerate through the generation tasks; never edit them by hand.
Repository host stories live in `tests/system/`; their shared host support lives
in `tests/support/`. They require prepared host fixtures and are not shipped in
the starter. Installed-wheel and cross-language stories remain repository proof.

## Plugin Lifecycle

Override `async def initialize(self, ctx: dokime_sdk.operations.Context) -> None`
and `async def finalize(self, ctx: dokime_sdk.operations.Context) -> None` on the
plugin class. The SDK encodes protected `<plugin>.initialize` and
`<plugin>.finalize` operations with empty payloads. Both contexts expose the author
instance through `ctx.plugin`; template initialize/cleanup retain execution scope.
