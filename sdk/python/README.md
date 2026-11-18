# Dokime Python SDK

`dokime-sdk` is a pure Python package for async clients and remote plugin authoring.
Python connects to an existing host; the application owns the plugin endpoint and its shutdown.

## Install

Use Python 3.13 or newer. From the repository root:

```sh
mise -C sdk/python run prepare
```

Prepare synchronizes the repository's `.venv/`, then builds the wheel and installs it there.

Starters and examples resolve the Nightly SDK with `uv sync`:

```toml
dependencies = ["dokime-sdk @ git+https://github.com/stokker-technologies/dokime@nightly#subdirectory=sdk/python"]
```

The Git build uses committed generated sources and needs only `uv`; it does not run repository tooling or protoc.
Repository tests install the wheel built from the working tree, and starter fixtures override the dependency with that wheel.
Consumers can also install the built wheel with `uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl`.
The wheel resolves its pinned transport dependencies automatically. Building it needs no Rust compiler.
The `polars` and `pandas` extras add the optional dataframe helpers, for example
`uv add "dokime-sdk[polars] @ file:///path/to/dokime_sdk-0.1.5-py3-none-any.whl"`.

## Connect a Client

```python
from dokime.connection import connect


async def inspect(endpoint: str) -> None:
    client = await connect(endpoint)
    try:
        entries = await client.plugins.list()
        for entry in entries.entries:
            print(entry.name)
            suites = await client.suites.list(entry.name)
            for suite in suites.values:
                print(suite.metadata.label)
    finally:
        await client.close()
```

`dokime.client.ClientRoot` exposes one composed resource tree.
Pass request content as arguments, such as `client.executions.get(name)` and
`client.plugins.settings.list(name)`. Operations with one response field return
that field; operations with several return their typed response. Client calls accept
`timeout`, `call_metadata` and `credentials` for one call. Close cancels the owned
channel and its active calls; closing twice is safe.

`connect` accepts an HTTP(S) origin, a pairing file, or `DOKIME_REMOTE`. HTTPS
requires a pinned PEM certificate. `access_key` applies bearer metadata to unary
and streaming calls; a paired host identity is checked through Status before use.
Use the generated access resources to manage listener settings and pairings.

Plugin administration and settings use `PluginEntry` identities:

```python
from dokime.client import PluginPlacement, PluginRemote, PluginValue
from dokime.connection import connect


async def register(endpoint: str, plugin_url: str) -> None:
    client = await connect(endpoint)
    try:
        result = await client.plugins.create(
            "example",
            PluginPlacement(remote=PluginRemote(url=plugin_url)),
            PluginValue(),
            request_id="register-example",
        )
        await client.plugins.settings.list(result.entry.name)
        await client.plugins.delete(result.entry.name, result.entry.revision)
    finally:
        await client.close()
```

Retain the same `request_id` when retrying a mutation. The read-only declared-secret
view is nested at `client.plugins.secrets`; it lists each declared identifier and
whether Secret Storage holds it, never secret bytes. The plugin context exposes only the operations plugins may call after composition.

## Author a Plugin

Authors import the product SDK. The starter uses plain values and declaration objects:

```python
from dokime import Plugin, Context, suite, step, criterion, record


@suite(label="Capture")
class Capture:
    confirmation = record(bool, label="Confirmation")

    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        await ctx.record(self.confirmation, answer == "Yes")

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(self.confirmation, step=self.confirm).last()


plugin = Plugin("example", "0.1.0", suites=[Capture])
```

`record(bool)`, `record(str)`, `record(int)` and `record(float)` use the protobuf
Boolean, string, signed 64-bit integer and double wrappers. Reading returns the
plain value; structured protobuf rows retain their message type. `last()` refuses
an empty result: append a record before reading its last value.

The SDK generates identifiers from code names, independently of labels: remove a
trailing `Suite` or `Template` from class names, separate word boundaries in every code name
with underscores, and lowercase. `ReadVoltage`, `readVoltage` and `read_voltage`
all become `read_voltage`.
For example, `VoltageAccuracy` becomes `voltage_accuracy`, `HTTPAccuracySuite`
becomes `http_accuracy`, and `read_voltage` stays `read_voltage`. An explicit
`identifier=` wins.
A single-class suite uses its suite identifier for its sole template:
`Capture` gives template `capture` and record `capture.confirmation`.
A grouped `Suite("accuracy", [Template(VoltageTemplate)])` gives template
`accuracy.voltage` and member `accuracy.voltage.measure`.
The grouping builder lives in `dokime.authoring`; root `Suite` is the wire model.

Records and reusable prompts live on template classes. Declare a reusable prompt
with `question = prompt("Continue?", choices=["Yes", "No"])` and pass it to
`await ctx.prompt(self.question)`. The short text call uses the qualified
`<template>.question` identifier too. Plugin construction qualifies declarations and
rejects empty or duplicate identifiers before serving. `ctx.record`, `ctx.records`
and `ctx.statistics` take a local declaration or stream method; step filters take
the bound step method. Materializers are synchronous and pure. Repeated calls to
one handler must supply distinct local `identifier=` values when adding case steps
or criteria; the same identifier function qualifies them during materialization.
Both class methods and bound instance methods may reference their declared handlers.

The runnable examples are:

- [Client](examples/client/main.py) browses a host.
- [Independent Operation](examples/plugin/operations.py) serves a plain string operation.
- [Capture](examples/test/suite.py) is the starter above.
- [Grouped Template](examples/test/template.py) composes the capture class explicitly.
- [Thermal](examples/thermal/suite.py) provides the cross-language host fixture.

Each plugin example has `plugin.py`, which starts only the sidecar, and `main.py`,
which starts it and drives a connected host through the client. The test project
includes `pyproject.toml`, suite modules and tests. Serve it with
`uv run --project sdk/python/examples/test python sdk/python/examples/test/plugin.py`.
Drive it with `uv run python sdk/python/examples/test/main.py --host <endpoint>`.
The application embedding these sources must use `plugin.py` as its entry point.
The owner of an endpoint closes it explicitly.

Additional operations use the same product package and plain payloads:

```python
from dokime import Context, Plugin, Visibility, operation


class Operations:
    @operation("example.echo", request=str, response=str, visibility=Visibility.PUBLIC)
    async def echo(self, ctx: Context, request: str) -> str:
        return request


plugin = Plugin("example.echo", "1.0.0", operations=Operations())
```

Structured operation payloads remain protobuf messages. `Visibility` has `PUBLIC`,
`PROTECTED` and `PRIVATE` members. Lifecycle operations remain protected.
The SDK creates one execution-owned template instance, shares it with steps,
criteria and observers, and releases it after cleanup, including when cleanup raises.

Settings and secrets are typed declarations on the plugin value. Secret reads name
only declared secrets; the host authorizes the trusted plugin identity and binding.
Omitting `requires` selects the generated host requirements. An explicit sequence
replaces them; host requirement enforcement remains in place.

## Observe Streams

`@stream("Temperature", row=Reading)` declares a stream method on a template;
its identifier comes from the method name unless explicitly supplied.
`@observer(interval_ms=...)` samples once per host-granted sample, while
`@window_observer(columns=..., interval_ms=...)` receives `ctx` and a `Window`
and returns a message repeating each row field across the window.
Import these decorators, `Context` and `Window` from `dokime`.

The host grants a window containing `count` samples from `start`, one `period` apart.
`interval_ms` may be fractional; `window_setting` selects the plugin duration
setting that replaces the host's window length. Streams use the shared payload
conversion: the default row is `float` (a double wrapper), and `row=bool`, `str`
or `int` selects the corresponding wrapper. Structured rows and window columns
return their declared protobuf messages.

## Call the Host

`Context` provides `prompt`, `record`, `records`, `statistics`, `progress`, `log`,
`traceability`, `document`, `artifact`, `settings`, `secret`, `sleep` and `now`.
Its `host` exposes the typed resource tree for calls needing the full contract.
`ctx.operation(Operation(...))(value)` binds an independent operation and accepts
a per-call policy override. Calls inherit execution scope and cancellation.
Time reads and sleeps use the host clock; observer cadence and retries belong to
operation policies and settings.

Media capture, FTP capture and the effects journal remain unimplemented. Their
host operations refuse the action rather than returning empty success. Storage
is an in-process host boundary; there is no SDK storage service.

## Compose and Decode Evidence

`dokime.plot` composes plot expressions. `Document`, `Section` and `Summary` in
`dokime.documents` retain canonical model messages, including pending/result
bindings and card completion supplied by host document responses.

`dokime.values.encode` and `decode` preserve signed/unsigned integers and Fraction
values through ExactNumber, including statistics sums, typed presence and plot
category totals. Drawing coordinates do not replace exact totals.

`FrozenSchemaCache` in `dokime.schemas` fetches execution schema pages at one read
cut, verifies their SHA-256, caches by digest and decodes frozen record/stream
bindings after plugin removal. Returned descriptor snapshots are independent copies.

Use `upload_file` in `dokime.transfers` for unified Files uploads and
`download_report(client, execution, max_bytes=...)` for verified report previews.
Uploads honor returned chunk/message bounds and preserve mutation identities;
report downloads verify file length and digest and never publish a report.

## Query and Export Evidence

`dokime.evidence` lists retained executions with `executions(client, template=...,
status=...)`, reads one stream at a fixed cut with `execution.telemetry(...)` and
converts it with `collect`, `to_polars` or `to_pandas`. `execution.export(path)` and
`execution.cycle.export(path)` download verified evidence archives atomically and
refuse existing destinations unless `overwrite=True`. See
[Use Python Remotely](../../docs/remote/python.md#query-retained-evidence).

## Build and Verify

```sh
mise -C sdk/python run format
mise -C sdk/python run check
mise -C sdk/python run generate -- --check
mise -C sdk/python run build
mise -C sdk/python run install
mise -C sdk/python run test -- --layer unit
mise -C sdk/python run test
mise -C sdk/python run generated
mise run check:generated -- --language python
mise -C sdk/python run package
mise -C sdk/python run wheel
```

`check` covers SDK sources, examples, tests and tooling with Ruff and the type
checker, including the shared Python host and example story under `examples/test/tests`,
without type ignores. `generate --check` regenerates wire bindings in
temporary storage and reports drift without writing committed output. `generated`
checks the installed wheel's source manifest, private descriptor set, message
shapes, RPC cardinalities and serializers. `tools/stubs.py` checks authored public
modules and generated resource signatures against the installed package.
`check:generated` imports the public barrels and type-checks client and plugin
content calls independently of the product host, using the same lint and typing
rules as authored code.

Build exports a pure wheel and source distribution under
`target/build/dev/any/python`; package verifies and copies those bytes to
`target/package/dev/any/python`. Repository tasks share the root `.venv`.
The SDK's immutable git dependency remains in its package metadata and lockfile.

Host-backed stories require prepared host fixtures and a controlled clock.
`mise -C sdk/python run example-test` runs the test example against those fixtures.
They await readiness and progress conditions. Each owned host removes its
credentials before disposing of its temporary storage, including on Windows.
Existing roots are never reset; incompatible roots require selecting a fresh root.
See [Execution Host](../../docs/execution-host.md) for fixture build commands.

Product/carrier integration stories serve real sidecars and exercise protobuf
round trips, private operation calls, records and qualified step filters. Generic
transport conformance belongs to the pinned transport dependency.

Full platform validation belongs to hosted CI and Nightly; a local native run
does not substitute for the other platform.
