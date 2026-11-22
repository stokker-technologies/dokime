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
`identifier` overrides the class-derived identity. The plugin binds every local
registration below its identifier: `LayoutSuite` / `TrackingTestTemplate.status`
in `ExamplePlugin` registers `example.layout.tracking.status`. Explicit local
identifiers follow the same rule; `example.status` stays `example.status`.
`uses_streams` and qualified document queries
keep their identifiers as written.

## Operation Calls

Call operations by identifier with an optional dict using protobuf's standard JSON
mapping. The host owns JSON conversion, schema validation, operation admission and
failure wording. Unknown fields are rejected.

```python
@step("Measure")
async def measure(self, ctx):
    reply = await ctx.operation("example.measure", {"channel": 1}, policy=None)
```

An omitted request sends an empty object. Replies are dicts. No peer binding or
request declaration is required. Catch `dokime_sdk.operations.FailureError` and
read `error.failure` for the host's classification, operation, field expectation
and recovery action. The callee's operation policy applies unless the call supplies
an override.

## Standard Logging

Use `log = logging.getLogger(__name__)` and ordinary synchronous `log.debug(...)`,
`log.info(...)`, `log.warning(...)` or `log.error(...)` calls. While a handler runs,
records flush to execution evidence before awaited SDK host calls and on return.
Outside execution scopes, records are written to plugin stderr for host diagnostics.

## Secrets and Resources

Declare secrets and resources as plugin attributes. Secret reads return UTF-8 text;
missing or empty values refuse with an action naming the secret to store before Retry.
Resources open lazily once per activation, serialize concurrent opens, and close
when the plugin finalizes, including when its finalizer raises.

```python
from dokime_sdk.operations import Context, Resource, Secret


async def open_remote(ctx: Context) -> Remote:
    token = await ctx.secret(ConnectionPlugin.api_token)
    return await Remote.connect(token)


async def close_remote(remote: Remote) -> None:
    await remote.close()


class ConnectionPlugin(Plugin):
    api_token = Secret("Remote API Token")
    remote = Resource(open=open_remote, close=close_remote)
```

Handlers use `remote = await ctx.resource(ConnectionPlugin.remote)`. Each declaration
gets its identifier from its attribute name. Resource open functions receive the
current context and can read declared secrets or open other declared resources.

## Declarations

```python
class ConnectionTestTemplate(TestTemplate):
    serial = Parameter.text("Serial", required=True)
    channels = Parameter.list("Channels", int, default=[1, 2], min_items=1, max_items=8)
    mode = Parameter.text("Mode", choices=("fast", "slow"), default="fast")

    @step("Connect")
    async def connect(self, ctx):
        serial = ctx.parameter(ConnectionTestTemplate.serial)


reading = record(int, identifier="reading")
Section("Readings").repeat(reading.field(), order_by=reading.field().max()).traceability()
time_series(field("measurement.samples", "value")).markers("reading")
```

Import these authoring values from `dokime_sdk.templates`. Document record fields
require an explicit record identifier; template binding qualifies local record
references and markers. Observer and window payloads preserve proto3 optional
field presence, including explicit zero values; interval storage adoption remains
pending. Execution observers inherit execution resources for admitted child calls,
including record publication. Typed stream subscription remains unavailable
in the product context adapter.

Use `ended.time_since(began)` for elapsed-time document expressions between record
declarations. Binding qualifies both identifiers in the expression. Reference
lines and bands use the existing `plot.reference_line(...)` and
`plot.reference_band(...)` builders and are placed explicitly in sections.

Stop releases execution-scoped host waits normally. The current step finishes,
criteria evaluate, cleanup runs and the execution seals with `Termination.STOPPED`,
retaining streams and records. Author loops read `ctx.stop_requested`:

```python
while not ctx.stop_requested:
    await ctx.sleep(10)
```

Cancel still aborts active work.

Handler exceptions become Permanent failures carrying their identity and exception text.
`OperationPolicy(transient=lambda error: isinstance(error, ConnectionError))` marks
selected handler exceptions Transient; existing structured failures keep their classification.

Setting bounds are inclusive and use the default's value type. The library
validates defaults at host admission and every saved write:

```python
from dokime_sdk.operations import SettingDeclaration, SettingValue

sample_count = SettingDeclaration(
    "sample_count",
    "Sample Count",
    "Samples per Capture",
    SettingValue(integer=10),
    minimum=SettingValue(integer=0),
    maximum=SettingValue(integer=100),
)
```

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
Registration waits for Ready and refuses unavailable or offline entries while releasing its listener.
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
the starter. Fixture lookup reads the exported build artifacts for `TEST_PROFILE`,
using `dev` when unset; `DOKIME_TEST_HOST` selects a prepared host explicitly.
Installed-wheel and cross-language stories remain repository proof.

## Plugin Lifecycle

Override `async def initialize(self, ctx: dokime_sdk.operations.Context) -> None`
and `async def finalize(self, ctx: dokime_sdk.operations.Context) -> None` on the
plugin class. The SDK encodes protected `<plugin>.initialize` and
`<plugin>.finalize` operations with empty payloads. Both contexts expose the author
instance through `ctx.plugin`; template initialize/cleanup retain execution scope.
