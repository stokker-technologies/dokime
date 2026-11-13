# Dokime Python SDK

`dokime-sdk` (import `dokime`) is one pure Python package for remote plugin authoring and async gRPC
clients. It builds on the Harmos Python SDK at revision
`81d32d3bc9ab3e2121746923712ec9508cee3ec7`, serves `harmos.plugin.v1.Instance`, and exchanges
generated messages: `dokime.v1` for shared values, `dokime.client.v1` for client calls,
`dokime.plugin.v1` for lifecycle bindings and `dokime.plugin.host.v1` for host calls.
Python never starts Dokime.

## Install

Use Python 3.13 or newer. Build this candidate and install its portable wheel:

```sh
mise run build:python
uv add --no-cache /path/to/dokime_sdk-0.1.5-py3-none-any.whl
```

The wheel installs `grpcio`, `protobuf`, and the exact-revision `harmos-sdk`
dependency. Building the SDK uses `grpcio-tools`; it requires no Rust or native
Dokime compiler. Registry publication is a separate release action.

## Author a Remote Plugin

```python
import dokime


@dokime.suite("example.measurement", label="Measurement")
class Measurement:
    def initialize(self, ctx):
        self.reading = 2.5

    @dokime.stream("Reading", identifier="example.reading")
    @dokime.observer(interval_ms=200)
    def reading_sample(self, ctx):
        return self.reading

    @dokime.step("Measure", retries=1)
    async def measure(self, ctx):
        await ctx.progress(1, 1, label="Measured")

    @dokime.criterion("Positive Reading")
    def positive(self, ctx):
        return self.reading > 0


async def main():
    endpoint = await dokime.serve(Measurement, address="127.0.0.1:7001")
    print(endpoint.url)
    # The application owns its event loop and shutdown signal.
    await endpoint.close()
```

The runnable [starter](examples/starter/) matches the Rust starter's Example suite:
Confirm asks a declared question and records its answer; Confirmed evaluates that record.
Its endpoint stays open until Ctrl+C.
Its own test drives a complete execution through the prepared product host and client root:

```sh
dokime new bench --lang python
cd bench
uv add --no-cache /path/to/dokime_sdk-0.1.5-py3-none-any.whl
mise run test
mise run serve
```

The CLI embeds `.gitattributes`, `.gitignore`, `main.py`, `suite.py`, `pyproject.toml`,
`mise.toml`, `tests/host.py`, and `tests/test_capture.py` directly from that directory.
CLI integration belongs to the CLI owner; these source files are authoritative.

## Compose Definitions

Python decorators register ordinary values. The equivalent explicit form is
`Suite(identifier, templates=[Template(identifier, instance, steps=[Step(...)],
criteria=[Criterion(...)])])`, passed to
`Plugin(identifier, version, suites=[...])` and then `await dokime.serve(plugin)`.
Pass generated `Parameter` and `FileParameter` messages and composed `Document`
values to `Template`. Duplicate identifiers fail when the plugin is constructed
or its catalog is built, before its endpoint starts.

The SDK derives a pure `<template>.materialize` operation and Protected unary
initialize, step, criterion, and cleanup operations. Step retries become Harmos
operation attempts. Parameter-dependent materialization uses the same handlers:

```python
@dokime.materialize
def materialize(cls, case, parameters):
    for channel in parameters["channels"]:
        case.step(f"Measure Channel {channel}", cls.measure, channel=channel)
```

Materializers are synchronous and must be pure. They may be called again to
recover argument bindings from a frozen case. They cannot call the host.
The SDK constructs one suite instance on its first execution operation, shares
it with steps, criteria, and observers, and releases it after cleanup, including
when cleanup raises. Resources stay on `self` and are never serialized.

## Host Calls and Evidence

Callbacks may be synchronous when they need no awaited work; host calls are
async. `await ctx.operation(operation)(request)` binds an explicit typed `dokime.Operation`
and forwards its policy and cancellation through Harmos. Import hardware drivers inside
operation handlers; suite steps call those operations through their context.
Use async handlers for local Private operations with the pinned library, including
handlers that call synchronous drivers.
`Plugin.requires` defaults to the SDK's host operations. An explicit `requires`
sequence replaces that set; compose it with `dokime.HOST_OPERATIONS`
when adding another plugin's operations.

Conveniences cover `prompt`, `sleep`, `now`, `record`, `records`, `statistics`,
`settings`, `secret`, `progress`, `log`, `traceability`, `document`, `artifact`,
file copying/capture, environment and effect prepare/confirm. They are unary
child calls on the invoking Harmos call, inheriting its execution scope and
cancellation. Sleep always uses the host clock.

Declare records with `Record(identifier, GeneratedMessage)` on the plugin;
publish with `await ctx.record(identifier, **fields)` or pass a message as the
second argument. `await ctx.records(identifier).last()` returns the decoded
message, and `.count()` returns the acknowledged count. Stream values use
generated protobuf messages; a scalar observer returns a float, carried as
`google.protobuf.DoubleValue`. A stream method without `@observer` is an async
generator. The host owns demand, sampling ticks, sample timestamps, and storage.

`dokime.plot`, `Stream.field()`, `Document`, `Section`, and `Summary` compose
canonical document/plot messages. A builder's `.message` is the generated wire
message, suitable for a catalog document or `ctx.document` part. No native
extension or separate binary descriptor is committed.

The pinned Harmos Python sample carrier creates observer contexts without an invocation
channel and refuses operation calls from sample callbacks. Successful observer child
calls need the library to supply that carrier; the SDK retains the library transport.

## Connect a Client

`await dokime.connect("http://127.0.0.1:8765")` opens an async channel.
Use `await client.close()` to cancel its calls and release the channel.
HTTPS requires a pinned PEM certificate; `access_key` supplies Bearer
authorization. A `.pair` path (or `DOKIME_REMOTE`) additionally requests backend
identity verification before returning a client.

`client.stubs` contains each generated `dokime.client.v1` service stub, keyed by its
fully qualified service name. `client.call(request)` locates a unique generated
method by input type; pass `method=MethodDescriptor` for an ambiguous method or
a streaming request. Unary calls are awaited; server-streaming calls are async
iterators. The channel's generated stubs also remain directly available.
`client.plugins`, `suites`, `templates`, `cycles`, `executions`, `prompts`,
`records`, `samples` and `reports` compose requests over those generated methods.
Cycle watches yield `dokime.client.v1.executions.ProgressWatchResponse` values and verify
finished progress followed by successful final status. The descriptor-derived
`client.executions.progress.get(request)` and `.watch(request)` use distinct client
wrappers; context `Progress.Get/Set` supplies whole-value revision checks. Declare
both context operations when selecting requirements for `ctx.progress(...)`.
Registry resources are flat: `client.suites`, `record_schemas`, `stream_schemas`,
`prompt_definitions` and `analyses`. Wheel builds generate concrete typed resource
classes and matching stubs. Convenience resources also accept canonical request
messages and retain their nested resources. The pinned library still owns its
`catalog_entries` root.
Report downloads create a preview and read its file through library Storage,
verifying size and SHA-256 before returning bytes. Downloads never publish a report.

```python
client = await dokime.connect(url)
try:
    await client.plugins.add(plugin_url)
    suites = await client.suites.list()
    cases = await client.templates.materialize("example", "example.capture", {})
    cycle = await client.cycles.start(cases)
    async for progress in cycle.watch():
        if progress.phase == "prompt":
            pending = await client.prompts.pending(cycle.executions[0])
            await client.prompts.respond(pending[0].prompt.prompt_id, "yes", accepted=True)
finally:
    await client.close()
```

The wheel generates Dokime client gRPC methods from the authoritative protos;
the library root supplies Access, Status and Storage. The in-process client story
adds a remote plugin, lists its canonical entry, drains Status.Logs envelopes,
propagates refusals and removes the entry before explicitly closing the client.
Pairing verifies the host identity through Status.Get.
The new execution host also runs the real-process Python client journey; see
[Execution Host](../../docs/execution-host.md).

## Decode Frozen Evidence

Execution details contain a schema digest and record/stream identities. Fetch descriptors
only on a digest miss and reuse them across executions, including after plugin removal:

```python
cache = dokime.FrozenSchemaCache()
execution = await client.executions.get(execution_id)
records = await client.records.list(execution_id)
door = await cache.decode(client, execution, "thermal.v1.Door", records.records[0].payload)
```

The cache fetches every `Executions.Schemas.Get` page at the same returned cut,
checks the digest before caching and assembles dependencies before decoding. Descriptor
snapshots returned by `get(client, execution)` are independent copies. Drop the cache
to release retained descriptors.

## Validate

Run these tasks from the repository root:

```sh
mise -C sdk/python run sdk:prepare
mise -C sdk/python run sdk:build
mise -C sdk/python run sdk:install
mise -C sdk/python run sdk:generated
mise run check:python:bindings
mise -C sdk/python run sdk:unit
mise run build:host-fixtures -- --language rust
mise -C sdk/python run sdk:test
mise -C sdk/python run sdk:starter
mise -C sdk/python run sdk:wheel
```

`sdk:generated` checks every source hash, generated message, service method,
cardinality, serializer, and response type. `check:python:bindings` also runs strict
mypy stubtest against the installed public package and generated resource stubs in a
fresh environment, without an allowlist or ignored missing stubs.
Builds generate messages, `.pyi`
files and gRPC stubs from `proto/`; generated Python is never committed. Source
distributions carry those same proto sources so wheel builds remain self-contained.

For transport conformance, build `harmos-conformance` through the pinned Harmos
checkout's `mise run build -- --package harmos-conformance`, set
`HARMOS_CONFORMANCE_HOST` to that executable, then run
`mise -C sdk/python run sdk:conformance`. It runs the fixed language-neutral
fixture through Dokime's plugin composition against all reference-host rules.

The cross-language suite is [`examples/thermal/suite.py`](examples/thermal/suite.py).
The named host story `python_plugin_matches_rust_thermal_evidence` adds its remote
URL, runs `thermal.cycle` with `limit = 85`, answers the door, and compares the
frozen test case, ordered records/steps/logs, samples and outcome with Rust.
It reopens the sealed evidence after shutdown. The starter retains its own
host-driven authoring story, using the client root rather than an Instance client stub.
The host owns the controlled clock, retries, evidence and cleanup. Linux stories
start an isolated session bus/keyring; install dbus and gnome-keyring first.
Build prerequisites and named host runs are in
[Execution Host](../../docs/execution-host.md).

## Composed Client Services

The async `Client` exposes the library resource root, including
`client.access.listener.get`, `client.access.pairings.create` and
`client.access.certificates.create`, with the client's configured metadata.
Use requests from the matching `harmos.v1.access` modules. Generic identity and
health use `client.status.get`; logs, trace events and diagnostics are nested
under that same root. Connection identity checks use the library's stable host ID.
