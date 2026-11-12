# Dokime Python SDK

`dokime` is one pure Python package for remote plugin authoring and async gRPC
clients. It builds on the Harmos Python SDK at revision
`a7583836e721c8dc8b135af40a78860ea5945ec6`, serves `harmos.v1`, and exchanges
generated `dokime.v1` messages. Python never starts Dokime.

## Install

Use Python 3.13 or newer. Build this candidate and install its portable wheel:

```sh
mise run build -- --language python
uv add /path/to/dokime-0.1.5-py3-none-any.whl
```

The wheel installs `grpcio`, `protobuf`, and the exact-revision `harmos-plugin`
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

The runnable [starter](examples/starter/) keeps its endpoint open until Ctrl+C.
Its own test drives a complete execution through an in-process Harmos host:

```sh
dokime new bench --lang python
cd bench
uv add /path/to/dokime-0.1.5-py3-none-any.whl
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

The SDK derives a pure `<template>.materialize` operation and Private unary
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
async. `ctx.operation(request)` selects a declared typed requirement; use
`operation=Operation(...)` when the same message belongs to several operations.
`Plugin.requires` defaults to the SDK's host operations. An explicit `requires`
sequence replaces that set; compose it with `dokime.context.HOST_OPERATIONS`
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

## Connect a Client

`await dokime.connect("http://127.0.0.1:8765")` opens an async channel.
Use `await client.close()` to cancel its calls and release the channel.
HTTPS requires a pinned PEM certificate; `access_key` supplies Bearer
authorization. A `.pair` path (or `DOKIME_REMOTE`) additionally requests backend
identity verification before returning a client.

`client.stubs` contains each generated `dokime.v1` service stub, keyed by its
fully qualified service name. `client.call(request)` locates a unique generated
method by input type; pass `method=MethodDescriptor` for an ambiguous method or
a streaming request. Unary calls are awaited; server-streaming calls are async
iterators. The channel's generated stubs also remain directly available.
`client.plugins.add/list/remove` are thin request constructors.

**Current protocol dependency:** repository `proto/dokime/v1` contains no service
declarations. The wheel therefore has 172 messages and zero client RPCs.
Generated clients and the plugin convenience calls cannot reach a Dokime host
until its authoritative service protos land. `MissingServiceError` reports the
missing request mapping; pairing cannot skip identity verification. Template
expansion, cycle start/watch, imports, and evidence clients additionally need
their authoritative request/reply/service definitions. No substitute wire
contract is supplied by Python.

## Validate

Run these tasks from the repository root:

```sh
mise -C sdk/python run sdk:prepare
mise -C sdk/python run sdk:build
mise -C sdk/python run sdk:install
mise -C sdk/python run sdk:generated
mise -C sdk/python run sdk:test
mise -C sdk/python run sdk:starter
mise -C sdk/python run sdk:wheel
```

`sdk:generated` checks every source hash, generated message, service method,
cardinality, serializer, and response type. Builds generate messages, `.pyi`
files and gRPC stubs from `proto/`; generated Python is never committed. Source
distributions carry those same proto sources so wheel builds remain self-contained.

For transport conformance, build `harmos-conformance` through the pinned Harmos
checkout's `mise run build -- --package harmos-conformance`, set
`HARMOS_CONFORMANCE_HOST` to that executable, then run
`mise -C sdk/python run sdk:conformance`. It runs the fixed language-neutral
fixture through Dokime's plugin composition against all reference-host rules.

The shared system suite is `examples/starter/suite.py`: plugin `example`,
template `example.capture`, parameter `limit`, steps `measure` and `confirm`,
observer stream `example.reading` every 200 ms, record `example.confirmation`,
and criterion `within.limit`. The host must register the remote endpoint,
expand/materialize parameters, create a PluginRun origin and matching scope,
drive retries and ticks on its controlled clock, acknowledge records/progress,
answer the choice prompt, serve step-filtered statistics and records, run cleanup,
close evidence, evaluate the cycle, and publish/seal/reopen the report. That real
Dokime system story remains pending the host and client service definitions.
