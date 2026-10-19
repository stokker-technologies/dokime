# Dokime Python Client

Async remote control, live records and telemetry, and saved execution evidence.
The Python package connects to a running Dokime backend. Its wheel and source
package contain Python code and generated protobuf bindings; building Dokime's
Rust applications is a separate backend operation.
Use the Rust CLI for terminal workflows.

## Install

Use Python 3.13 or newer. Install the supplied wheel in your application environment:

```sh
python -m pip install dokime-0.1.0-py3-none-any.whl
```

Install the prepared source distribution with the same Python toolchain:

```sh
python -m pip install dokime-0.1.0.tar.gz
```

The source package uses Hatchling and requires no Rust compiler or contract
generation. Runtime dependencies are `grpcio`, `protobuf` and `python-dotenv`; install their
compatible platform wheels as usual.

## Named Backend Profiles

Read a saved profile explicitly, supplying its bearer token separately:

```python
async with DokimeClient.from_profile("lab", key_file="remote.key") as client:
    backend = await client.connect()
```

`from_profile` reads configuration once and starts no server. Use `path` to
select another profile file, or `DOKIME_PROFILES_FILE` to override the standard
OS configuration location. The Rust CLI manages these profiles through
`dokime backend add/list/show/use/remove`. Python keeps the selected endpoint,
CA and expected backend identity for that connection; later default edits have
no effect. An unexpected identity fails negotiation before workflow mutations.
Profiles marked `keyring` require a key supplied by the Python caller.

Credential precedence is explicit `key_file=`, process `DOKIME_KEY`, then the
key in an explicitly selected `env_file=`. Selected files are validated before
any connection, even when a higher-priority source supplies the key. Keys are
UTF-8 text with 32–4096 visible ASCII characters. Dotenv files are bounded to
64 KiB, never searched for automatically, and never overwrite process values.
Constructor arguments read these local files; they open no network channel.

```python
async with DokimeClient(endpoint, env_file=".env", ca_certificate="server.pem") as client:
    backend = await client.connect()
```

## Creating Cycles

A cycle is created explicitly and starts empty; cases join it only by being
appended. Suites and templates are named by their readable IDs, and appended
cases continue the cycle's numbering:

```python
async with DokimeClient(endpoint, key_file="remote.key") as client:
    cycle_id = await client.create_cycle(label="Campaign 1", description="Bench A")
    (execution,) = await client.append_cases(
        cycle_id,
        suite="obc.checks",
        template="obc.checks.pulse",
        parameters={"sample_count": 8},
    )
    cycle = await client.cycle(cycle_id)
```

`append_cases` returns the ordered handles created by that exact mutation,
including when another client appends concurrently. Keep their IDs for recovery.
Call `await execution.start(request_key="pulse-1")` explicitly; appending never
runs the device.

`new_cycle(suite=..., template=..., label=...)` composes both for scripts and
validates the selection before creating anything. Neither call is idempotent:
after an ambiguous failure, inspect the backend instead of retrying.

Several cycles can remain Active. A run request selects only the unrun
executions present at admission; later additions need another request. Requests
run in acceptance order, with one execution running server-wide. Executions seal
independently, and their cycle remains Active after completion.

Close the container explicitly when the campaign is finished:

```python
await cycle.seal()
```

This closes admission, stops this cycle's running work, cancels its unrun
executions and waits for all evidence and the cycle report to be durably saved.
Other cycles continue. Repeating `seal()` safely resolves a lost response;
it never runs device operations again. Shutdown seals Active cycles, and startup
finishes old cycles before admitting new work.

Export an execution after it seals, then close and export its cycle:

```python
await execution.wait()
saved = await execution.export("execution.zip")
await cycle.seal()
await cycle.export("cycle.zip", overwrite=True)
print(saved.path, saved.size_bytes, saved.sha256)
```

The execution export works while its cycle is Active; a cycle export requires
a durable seal. Both use the runtime's saved revisions and include a manifest,
SQLite evidence, captured files and saved HTML reports. Inspect `streams` and
`stream_columns` in `evidence.sqlite` for typed table/column mappings and units.
The server allows one export at a time and limits each archive to 8 GiB.
Missing required evidence fails explicitly. Downloads verify size, SHA-256 and
successful terminal transport status before atomic publication. Existing files
require `overwrite=True`; failure preserves the old destination and removes the
partial file. Cancelling export never cancels execution. A download retry starts
from the beginning and never reruns device operations.

## Confirmation Prompts

The default `obc.checks.pulse` has no prompt. Append it with
`parameters={"require_confirmation": True}` to ask one confirmation before
sampling. After starting that execution, discover and answer its own prompt:

```python
async with asyncio.timeout(60):
    while not (pending := [item async for item in execution.pending_prompts()]):
        await asyncio.sleep(0.2)
prompt = pending[0]
print(prompt.title, prompt.message, prompt.actions)
await execution.respond(prompt.id, action="yes")
outcome = await execution.wait(timeout=60)
```

Import `asyncio` for this wait. `execution.prompt(prompt_id)` inspects the admitted
question and any committed response. Reads and responses require this execution's
identity; a prompt from a sibling execution is refused. After an ambiguous error,
inspect the question before deciding whether to submit again. Duplicate or stale
answers are refused by the server. To cancel the prompt, pass `action="cancel",
accepted=False`; stopping the execution also settles its pending prompts.

## Remote Control And Listening

Use the backend's HTTPS origin, its access key and the public certificate
downloaded from the host. Keep the key outside source files. The scripts in
[`examples/remote/python`](examples/remote/python/) demonstrate manual device
power, two executions in one cycle, and an execution confirmation. The test
script follows filtered records and samples concurrently, drains both readers
after the durable outcome, then exports each execution and the sealed cycle.
The test and prompt scripts request cancellation on Ctrl+C and save collected
evidence; a failed connection reports recovery identities without claiming
cancellation succeeded. Matching first-party release artifacts and the tutorial
dependency lock remain required before the beginner installation is available.

```python
from dokime.client import DokimeClient


async def record(item):
    print(item)


async with DokimeClient(
    "https://dokime.example:8765",
    key_file="remote.key",
    ca_certificate="/path/to/ca.pem",
) as client:
    execution = await client.execution(execution_id)
    outcome = await execution.run(request_key=request_key, on_record=record)
```

Retain execution identity, request key and mode before submission. If admission
is uncertain, resolve it using the same identity, key and mode. Creating a new
cycle is a separate non-idempotent mutation. The client never retries mutations
automatically. Closing the client, cancelling a callback, or reaching a local
timeout leaves admitted backend work running; `execution.stop()` explicitly
requests cooperative stop. A terminal admission receipt and a saved execution
outcome are separate facts.

Live cursors belong to one backend epoch. Replacing or restarting the backend
fails live readers; recover immutable evidence through the original checkpoint.
Downloads verify identity, contiguous offsets, completion, byte count and SHA-256
before replacing the destination.

## Canonical Operations And Plugin Discovery

The generated SDK exposes typed operation constants and standard protobuf request
and response classes from the canonical Harmos catalog. The client also discovers
caller-authorized operations from separately installed plugins, without requiring
those plugins in the Python distribution. Standalone `--auth-file` connections
require an exact owner/operation permission on the server's bearer grant; see
[Plugin Operation Permissions](../../README.md#plugin-operation-permissions).
Unlisted plugin operations are denied on those connections. Native trusted-local
connections admit installed unscoped operations through their live contracts;
request-dependent scopes still require supported scoped authorization.

```python
async with DokimeClient(endpoint, key_file="remote.key") as client:
    providers = await client.operations()
    provider = await client.show("acme.sensor.read", owner="acme.sensor")
    request = provider.operation.request()
    response = await client.call(provider.operation, request)
```

Use an operation ID with a standard protobuf JSON mapping for a dynamic unary
call. Unknown fields are refused. Protobuf JSON represents 64-bit integers as
strings; enum branches use their canonical message fields. `show` returns the
owner, generation, grants, conditional-access flag, gRPC availability and contract.
Ambiguous providers require an explicit owner.

`upload`, `download` and `exchange` implement the shared Harmos streaming
interfaces. Input streams begin with the declared request. Output streams yield
items and retain the declared final response, available through `response()`.
Close an abandoned stream with `close()`; closing the client cancels its local
calls and does not stop admitted work on the backend.

Before operation transmission, the client checks the installed contract against
the live provider. Contract drift and unavailable gRPC bindings are refused.
RPC failures retain their status, initial/trailing metadata and native cause;
mutating calls are never automatically retried.

## Aggregate Queries

`Query.Aggregate` and plot expressions name their statistic with the Harmos
aggregate enum message: one `_harmos_variant` oneof whose members are `Count`,
`Sum`, `Mean`, `Median`, `Min`, `Max`, `Rms`, `Std` and `Percentile`, which
carries a `rank` from 0 to 100. `Aggregate` builds and reads that field for any
generated message; ranks outside 0 to 100 are refused before transmission.

```python
from dokime import DOKIME_QUERY_AGGREGATE
from dokime.client import Aggregate

request = DOKIME_QUERY_AGGREGATE.request(
    source={"Steps": {}},
    group_by=["identifier"],
    aggregates=[
        {
            "aggregate": Aggregate.percentile(95).to_wire(),
            "column": "duration_ms",
            "alias": "p95",
        },
        {"aggregate": Aggregate.count().to_wire(), "alias": "samples"},
    ],
)
reply = await client.call(DOKIME_QUERY_AGGREGATE, request)

assert Aggregate.from_wire(request.aggregates[0].aggregate) == Aggregate.percentile(95)
```

The generated field remains a plain protobuf oneof:
`entry.aggregate.Percentile.rank = 95` selects the percentile, and
`entry.aggregate.WhichOneof("_harmos_variant")` names the member. An absent
member reads as `Count`.

## Follow And Resume Execution Files

`TestExecutionHandle.follow_file(relative_path, destination)` keeps a local file
up to date while its registered backend producer writes it. Each update includes
the stable `FileIdentity` and locally committed offset. Retain that identity and
pass `identity=...` with the existing destination to resume after interruption.
The complete retained prefix participates in final SHA-256 verification.
A `finished` update requires the original seal and successful final RPC status;
`abandoned` retains partial bytes and carries no seal. Cancellation leaves the
committed local prefix available for explicit resume.
