# Dokime Python Client

Async remote control, live records and telemetry, and saved execution evidence.
The Python package connects to a running Dokime backend. Its wheel and source
package contain Python code and generated protobuf bindings; building Dokime's
Rust applications is a separate backend operation.
Use the Rust CLI for terminal workflows.
The distribution is `dokime-client`; Python imports use `dokime_client`.

## Install

Use CPython `>=3.13,<3.16`. For the beginner host/second-PC workflow, follow the
[candidate-artifact walkthrough](examples/remote/README.md) and its
[locked uv setup](examples/remote/python/README.md). That environment uses a
bundled candidate wheel and makes no published-registry availability claim.

Add the published distribution to a uv project with `uv add dokime-client`. For a separately supplied matching wheel:

```sh
uv add ./dokime_client-0.1.0-py3-none-any.whl
```

Install the prepared source distribution with the same Python toolchain:

```sh
uv add ./dokime_client-0.1.0.tar.gz
```

The source package uses Hatchling and requires no Rust compiler or contract
generation. Runtime dependencies are `grpcio`, `protobuf` and `python-dotenv`; install their
compatible platform wheels as usual.

## Named Backend Profiles

Read an unsecured lab profile explicitly:

```python
from dokime_client import DokimeClient

async with DokimeClient.from_profile("lab") as client:
    backend = await client.connect()
```

`from_profile` reads configuration once and starts no server. Use `path` to
select another profile file, or `DOKIME_PROFILES_FILE` to override the standard
OS configuration location. The Rust CLI manages these profiles through
`dokime backend add/list/show/use/remove`. Python keeps the selected endpoint,
CA and expected backend identity for that connection; later default edits have
no effect. An unexpected identity fails negotiation before workflow mutations.
Python does not read the native keyring. For an authenticated backend, create a
pairing file on its host and import that file instead:

```python
async with DokimeClient.from_pairing("lab.dokime-pair") as client:
    backend = await client.connect()
```

The constructor reads and validates that explicit file before connecting. It
retains the endpoint, expected backend identity, CA and independent access key
for the connection. Protect the pairing file as a credential; revoking its
paired client on the host prevents further use. See the
[pairing walkthrough](examples/remote/README.md) for the host and second-PC flow.

The endpoint scheme selects native gRPC transport: `https://` verifies TLS,
while explicit `http://` permits plaintext for an isolated lab and exposes keys
and data. The listener decides independently whether a key is required. A
profile marked `keyring` is refused by `from_profile`; use `from_pairing`.
CA certificates apply only to HTTPS; certificate verification is never disabled.

```python
async with DokimeClient("http://127.0.0.1:8765") as client:
    backend = await client.connect()
```

## Client Configuration

Each connection uses the generated application defaults for
`connect_timeout_ms` and `request_timeout_ms`. Direct construction and
`from_profile` also accept `preferences=`, a generated client-preferences message
with those two fields, and an explicit `rpc_timeout` override. Streaming
operation deadlines come from their declared contracts.

Python does not load or create a local application-settings file. Application
preferences belong to the backend's SQLite settings owner; named profiles remain
in their separate `backends.json` document. Legacy TOML files are left untouched.

## Creating Cycles

A template materializes immutable cases through the server's domain planner.
Construct each execution locally, then append those exact attempts to a cycle:

```python
from dokime_client import DokimeClient, Execution

async with DokimeClient.from_pairing("lab.dokime-pair") as client:
    template = await client.template("obc.checks.pulse")
    cases = await template.materialize(parameters={"sample_count": 8})
    executions = [Execution(test_case) for test_case in cases]

    cycle = await client.create_cycle(label="Campaign 1", description="Bench A")
    await cycle.append(executions)
```

`create_cycle` returns its bound handle directly. Materialization captures the
definition and provider revision; append refuses a stale case if either changes.
Each `Execution(test_case)` gets a fresh stable UUID without network or device
work. `cycle.append` registers the supplied attempts atomically in order and
binds those same Python objects after confirmation. Repeating an append with the
same identities and bindings cannot duplicate them; conflicting reuse is refused.
Keep the execution objects and IDs when recovering an uncertain append response.

Creating a cycle is not idempotent. After an uncertain creation response, inspect
the backend before creating another container. Constructing or appending an
execution never starts it; start execution explicitly after planning is complete.

Several cycles can remain Active, with one execution running server-wide.
Start each attempt with `await execution.start()`, or use `execution.run()`
to Start and wait for its saved outcome and requested listener drain. Start is
idempotent by execution identity and mode, including queued admission. A sealed
execution never restarts; construct a new `Execution(test_case)` to repeat it.

`create_cycle(..., continue_automatically=False)` defaults to explicit Starts.
Setting it to `True` lets the server continue with the next unstarted member
after natural completion and durable sealing, whatever the previous verdict.
Explicit `execution.stop()` durably sets the cycle's
`continue_automatically=False`. Later explicit Start runs only the selected
attempt until the user explicitly re-enables that policy.
Appending to an idle cycle starts no hardware. Executions seal independently;
their cycle remains Active until explicitly sealed. Cycles have no Run or Stop.

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
The server enforces its configured export concurrency and archive byte allowance;
the client uses the backend's declared byte bound.
Missing required evidence fails explicitly. Downloads verify size, SHA-256 and
successful terminal transport status before atomic publication. Existing files
require `overwrite=True`; failure preserves the old destination and removes the
partial file. Cancelling export never cancels execution. A download retry starts
from the beginning and never reruns device operations.

## Confirmation Prompts

The default `obc.checks.pulse` has no prompt. Materialize its confirmation case,
append an attempt, then Start without blocking so the script can answer:

```python
import asyncio
from contextlib import aclosing

template = await client.template("obc.checks.pulse")
(test_case,) = await template.materialize(parameters={"require_confirmation": True})
execution = Execution(test_case)
await cycle.append([execution])
await execution.start()
async with asyncio.timeout(60):
    while True:
        async with aclosing(execution.pending_prompts()) as reader:
            pending = [item async for item in reader]
        if pending:
            break
        if await execution.checkpoint() is not None:
            raise ValueError("execution sealed without its confirmation")
        await asyncio.sleep(0.2)
if len(pending) != 1:
    raise ValueError("expected one confirmation")
prompt = pending[0]
print(prompt.title, prompt.message, prompt.actions)
await execution.respond(prompt.id, action="yes")
outcome = await execution.wait(timeout=60)
```

Use the client and cycle created above. `execution.prompt(prompt_id)` inspects the admitted
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
Each script has editable globals in the same order as the
[native shell examples](examples/remote/cli/README.md): connection parameters,
case and execution parameters, observation filters where applicable, and file
parameters. Set `ENDPOINT`, `CA_CERTIFICATE`, `KEY_FILE` and `ENV_FILE` to the
explicit connection sources, then run without arguments:

```sh
python control-device.py
python control-test.py
python control-prompt.py
```

The device script writes `before.json`, `power-on.json`, `running.json`,
`power-off.json` and `after.json` in the current working directory, replacing
these fixed response files on each run. Replies are JSON objects with their
Protobuf field names. A local write failure can follow a successful device command;
read the current device state before resending an unconfirmed operation.
The test and prompt scripts request cancellation on Ctrl+C and save collected
evidence. They seal their own cycle before waiting for the saved result, closing
admission even if interruption raced with submission; a failed connection reports recovery identities without claiming
cancellation succeeded. The beginner candidate path carries its exact wheel and
uv lock; a matching published release combination remains separately required
before describing it as a released tutorial.
The printed cycle and execution UUIDs remain recovery identities after an
uncertain reply. Scripts never manage Start keys or retry mutations automatically.

All Python examples use direct client APIs and editable globals, with no
argument parser or Dokime CLI subprocess. The standalone
[`remote_control.py`](examples/remote_control.py) attaches to `EXECUTION_ID`,
drains its callbacks and saves `execution.json`.
[`client/poll_run.py`](examples/client/poll_run.py) materializes `TEMPLATE`,
runs one attempt and writes `cycle.json` and `summary.json`. Edit their globals,
then run the file without arguments.

Repository Python Build stages the canonical guides, scripts, locked tutorial
environment and bundled candidate wheel under
`target/build/<profile>/any/python/examples/remote/`. Python Package consumes those
files and produces
`target/package/<profile>/any/python/dokime-remote-examples-<version>.zip`, alongside
the client distributions in that same Python subtree, without
rebuilding. It includes only the explicit public
example inventory; local `.env`, key files and collected evidence are excluded.
The archive's shared [beginner guide](examples/remote/README.md) explains the
candidate installation and its limits. The bundled wheel pin does not establish
published package availability or a verified release combination.

```python
from dokime_client.client import DokimeClient


async def record(item):
    print(item)


async with DokimeClient.from_pairing("lab.dokime-pair") as client:
    execution = await client.execution(execution_id)
    outcome = await execution.run(on_record=record, record_types=("obc.run.*",), timeout=60)
```

Retain execution identity and mode before Start. If admission
is uncertain, resolve it using the same identity and mode. Creating a new
cycle is a separate non-idempotent mutation. The client never retries mutations
automatically. Closing the client, cancelling a callback, or reaching a local
timeout leaves admitted backend work running; `execution.stop()` explicitly
requests cooperative stop. Start admission and a saved execution outcome are
separate facts. `execution.run()` returns after the saved outcome and final
drain of every supplied callback; an empty filter sequence selects all types
or streams for that callback.

Live cursors belong to one backend epoch. Replacing or restarting the backend
fails live readers; recover immutable evidence through the original checkpoint.
Downloads verify identity, contiguous offsets, completion, byte count and SHA-256
before replacing the destination.

## Canonical Operations And Plugin Discovery

The generated SDK exposes typed operation constants and standard protobuf request
and response classes from the canonical Harmos catalog. The client also discovers
caller-authorized operations from separately installed plugins, without requiring
those plugins in the Python distribution. A valid remote access key grants full
control over the installed plugins, including their request-dependent scopes;
see [Full-Control Access And Plugin Capabilities](../../README.md#full-control-access-and-plugin-capabilities).
Python discovery projects providers with gRPC declarations. Native-only operations
are omitted; declared bindings that are unavailable retain `grpc_available=False`.
Internal host providers stay unavailable remotely.
The plugins' own requirements for calls into other plugins or host services
remain enforced independently of the access key.

```python
async with DokimeClient.from_pairing("lab.dokime-pair") as client:
    providers = await client.operations()
    provider = await client.show("stokker_sensor.read", owner="stokker_sensor")
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
from dokime_client import DOKIME_QUERY_AGGREGATE
from dokime_client.client import Aggregate

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

`Execution.follow_file(relative_path, destination)` keeps a local file
up to date while its registered backend producer writes it. Each update includes
the stable `FileIdentity` and locally committed offset. Retain that identity and
pass `identity=...` with the existing destination to resume after interruption.
The complete retained prefix participates in final SHA-256 verification.
A `finished` update requires the original seal and successful final RPC status;
`abandoned` retains partial bytes and carries no seal. Cancellation leaves the
committed local prefix available for explicit resume.
