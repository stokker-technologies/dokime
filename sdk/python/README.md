# Dokime Python SDK

`dokime-sdk` is a pure Python package for async clients and remote plugin authoring.
It composes Harmos at immutable revision `299b7ce492e3a82bc4ddf61dc60c2161b6824110`.
Python connects to an existing host; the application owns the plugin endpoint and its shutdown.

## Install

Use Python 3.13 or newer. From the repository root:

```sh
mise -C sdk/python run prepare
mise -C sdk/python run build
mise -C sdk/python run install
```

Consumers install the built wheel with `uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl`.
The wheel depends on `grpcio`, `protobuf` and the pinned Harmos SDK. Building it needs no Rust compiler.

## Connect a Client

```python
from dokime.connection import connect


async def inspect(endpoint: str) -> None:
    client = await connect(endpoint)
    try:
        entries = await client.plugins.list()
        for entry in entries.entries:
            print(entry.name)
        suites = await client.suites.list()
        for suite in suites.suites:
            print(suite.metadata.label)
    finally:
        await client.close()
```

`dokime.client.Client` exposes one composed resource tree.
Pass request content as arguments, such as `client.executions.get(name)` and
`client.plugins.settings.list(name)`. Operations with one response field return
that field; operations with several return their typed response. Client calls accept
`timeout`, `call_metadata` and `credentials` for one call. Close cancels the owned
channel and its active calls; closing twice is safe.

`connect` accepts an HTTP(S) origin, a pairing file, or `DOKIME_REMOTE`. HTTPS
requires a pinned PEM certificate. `access_key` applies bearer metadata to unary
and streaming calls; a paired host identity is checked through Status before use.
Use the generated access resources to manage listener settings and pairings.

Plugin administration and settings use the library's PluginEntry identity:

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

Retain the same `request_id` when retrying a mutation. Product secret bindings
are nested at `client.plugins.secrets`; they contain host secret identifiers and
availability, never secret bytes. Binding replacement uses the revision returned
by List. The plugin context exposes only the operations plugins may call after composition.

## Author a Plugin

The examples are the starter sources:

- [Client](examples/client/main.py) browses a running host.
- [Plugin](examples/plugin/main.py) serves an independent typed operation.
- [Test Suite](examples/test/suite.py) declares a confirmation journey with decorators.
- [Test Template](examples/test/template.py) composes that journey explicitly.
- [Thermal Suite](examples/thermal/suite.py) retains the cross-language host fixture.

Serve the test example with `uv run python sdk/python/examples/test/main.py`.
The endpoint stays open until cancellation. Its owner always calls `close()`.
The command-line tool embeds these example sources; its embedding is owned by the CLI.

Import authoring builders and decorators from `dokime.authoring`, document builders
from `dokime.documents`, and `Plugin`, `Context` and `HostOperation` from
`dokime.plugin`. Root types are model values. For example, the authoring `Suite`
builder and the root Suite model have distinct modules.

`Plugin(identifier, version, suites=[...], records=[...])` composes one Harmos
plugin. `requires` accepts generated HostOperation members and independently typed
operation declarations. Omitting it selects the generated plugin host requirements;
an explicit sequence replaces that set. `settings` accepts library setting
declarations. `secrets` accepts SecretDeclaration values; duplicate or empty
identifiers fail before serving. A secret read must name a declared secret, and the
host also authorizes its trusted plugin identity and binding.

Lifecycle and analysis payloads come from `dokime.plugin`. Independent lifecycle
bindings remain Protected. Use the library's `operation` decorator for additional
operations, and pass their instance through `Plugin(operations=...)`. The SDK
constructs an execution-owned suite instance, shares it with steps, criteria and
observers, and releases it after cleanup even when cleanup raises.

Materializers are synchronous and pure. They freeze parameter-dependent steps
and criteria without host calls. Resources remain on the suite instance and are
never serialized. Stream callbacks retain their declared protobuf row types and
raw analysis time axes; scalar observers use DoubleValue rows.

## Call the Host

Callbacks receive a typed Context. Its `host` is the generated plugin resource root:

```python
from dokime.plugin import Context


async def confirm(context: Context) -> None:
    answer = await context.host.prompts.create("confirmation", "example.confirmation", {})
    await context.record("example.confirmation", value=answer.accepted and answer.action == "yes")
```

Conveniences cover records, prompts, progress, logs, traceability, document parts,
settings and declared secrets. `context.operation(operation)(request)` binds an
independent typed operation and supports a policy override. Host calls inherit
execution scope and cancellation. Time reads and sleeps use the library's host
clock; observer intervals and retries belong to operation policies and settings.
The library's sample carrier must provide an invocation channel before sample
callbacks can make child calls.

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

## Build and Verify

```sh
mise -C sdk/python run format
mise -C sdk/python run check
mise -C sdk/python run generate -- --check
mise -C sdk/python run build
mise -C sdk/python run install
mise -C sdk/python run unit
mise -C sdk/python run test
mise -C sdk/python run generated
mise -C sdk/python run test:generation
mise -C sdk/python run package
mise -C sdk/python run wheel
```

`check` covers SDK sources, examples, tests and tooling with Ruff and the type
checker, without type ignores. `generate --check` regenerates wire bindings in
temporary storage and reports drift without writing committed output. `generated`
checks the installed wheel's source manifest, private descriptor set, message
shapes, RPC cardinalities and serializers. `tools/stubs.py` checks authored public
modules and generated resource signatures against the installed package.
`test:generation` imports the public barrels and type-checks client and plugin
content calls independently of the product host, using the same lint and typing
rules as authored code.

Build exports a pure wheel and source distribution under
`target/build/dev/any/python`; package verifies and copies those bytes to
`target/package/dev/any/python`. Repository tasks share the root `.venv`.
The SDK's immutable git dependency remains in its package metadata and lockfile.

Host-backed stories require prepared host fixtures and a controlled clock.
They await readiness and progress conditions. Each owned host removes its
credentials before disposing of its temporary storage, including on Windows.
Existing roots are never reset; incompatible roots require selecting a fresh root.
See [Execution Host](../../docs/execution-host.md) for fixture build commands.

For transport conformance, set `HARMOS_CONFORMANCE_HOST` to the pinned library's
reference host and `HARMOS_CONFORMANCE_FIXTURE` to its
`sdk/python/tests/fixtures/conformance` directory, then run
`mise -C sdk/python run conformance`. Full platform validation belongs to hosted CI
and Nightly; a local native run does not substitute for the other platform.
