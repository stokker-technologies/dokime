# Dokime Python Client

Async remote control, live records and telemetry, finite snapshot queries, and saved execution evidence.
The Python package connects to a running Dokime backend. Its wheel and source
package contain Python code and generated protobuf bindings; building Dokime's
Rust applications is a separate backend operation.
Use the Rust CLI for terminal workflows.
The distribution is `dokime-client`; Python imports use `dokime_client`.

## Install

Use CPython `>=3.13,<3.16`. For the beginner host/second-PC workflow, follow the
[candidate-artifact walkthrough](../../docs/remote/overview.md) and its
[locked uv setup](../../docs/remote/python.md). That environment uses a
locally built matching wheel and makes no published-registry availability claim.

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

Finite query iteration needs no dataframe library. Install either optional extra
for local conversion, keeping the client and backend revisions matched:

```sh
uv add 'dokime-client[polars]'
uv add 'dokime-client[pandas]'
# For a matching local wheel:
uv add './dokime_client-0.1.2-py3-none-any.whl' --extra polars
```

Only the selected conversion imports its library. A missing extra raises
`QueryDependencyError` with installation guidance before any page is requested.

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
remote access file on its host and import that file instead:

```python
async with DokimeClient.from_pairing("lab.dokime-remote") as client:
    backend = await client.connect()
```

The constructor reads and validates that explicit file before connecting. It
retains all endpoints, expected backend identity, persistent CA and independent access key
for the connection. Protect the remote file as a credential; revoking its
paired client on the host prevents further use. See the
[remote access walkthrough](../../docs/remote/overview.md) for the host and second-PC flow.

Candidates connect in parallel with the configured short stagger (250 ms by
default). The first connection that verifies the pinned CA and backend identity
wins; losing attempts are closed. Reconnects try that endpoint first. A client
loaded with `from_profile` also saves its successful endpoint for later clients.
The listener reissues leaves under the same persistent CA when addresses change.

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
`connect_timeout_ms`, `request_timeout_ms` and `endpoint_stagger_ms`. Direct construction, `from_pairing` and
`from_profile` also accept `preferences=`, a generated client-preferences message
with those fields, and an explicit `rpc_timeout` override. Streaming
operation deadlines come from their declared contracts.

Python does not load or create a local application-settings file. Application
preferences belong to the backend's SQLite settings owner; named profiles remain
in their separate `backends.json` document. Legacy TOML files are left untouched.

## Creating Cycles

Use an owned cycle context to start a template and retain its sealed evidence:

```python
import asyncio
from pathlib import Path
from dokime_client import DokimeClient


async def main() -> int:
    async with DokimeClient.from_pairing("examples/lab.dokime-remote") as client:
        async with client.new_cycle("Example Lab Tracking") as cycle:
            run = await cycle.start("example.demo.tracking", duration_min=10)
            async for record in run.records("example.tool.connected", "example.tool.disconnected"):
                print(record.type)
        await cycle.save(Path("target/reports/tracking") / run.id, report="cycle.summary")
        return run.outcome.exit_code


raise SystemExit(asyncio.run(main()))
```

This template requires the corresponding installed plugin. For the OBC fixture,
use `await cycle.start("obc.checks.pulse", sample_count=8)`.
`new_cycle(label, description=None, identifier=None, continue_automatically=False)`
creates the cycle on entry. Every exit, including exceptions and Ctrl+C, seals
it through the server's durable boundary: admission closes, owned work stops,
unstarted members cancel, and final outcomes drain. Other cycles continue.
Cleanup is shielded against cancellation. An ordinary body failure and cleanup
failure raise `ExceptionGroup`; cancellation or KeyboardInterrupt requires its
`BaseExceptionGroup` subclass so the original failure can remain intact.
A lost creation response leaves the identity unknown; inspect the backend before
creating another cycle. Cleanup requires a reachable backend and reports failures.

`cycle.start(template, **parameters)` resolves the template, materializes cases,
constructs fresh executions, appends them and admits their Starts. One case
returns one `Execution`. Expansion returns a tuple of **all** handles in case
order; Starts enter the existing server queue in that order:

```python
runs = await cycle.start("lab.sweep", voltage=230)
# For a template that expands to multiple cases:
for run in runs:
    outcome = await run
    print(run.id, outcome.status, outcome.exit_code)
```

Each handle exposes `.id`, is awaitable, and retains `.outcome` after awaiting it
or fully consuming either evidence iterator. Accessing `.outcome` before then
raises `RuntimeError`. `outcome.status` and `outcome.result` retain the backend
verdict. `outcome.exit_code` is 0 for passed, 1 for failed/errored or any other
nonpassing verdict, and 130 for interrupted/cancelled.
`await run.wait(timeout=60)` remains available for an explicit local deadline.

`run.records(*types)` and `run.samples(*streams)` replay from the beginning and
follow through the execution's saved seal, then terminate. Empty filters select
all; filters are case-sensitive identifier globs with `*` and `?` wildcards.
Execution and cycle queries include records and samples they produced or
captured, including plugin-wide observers. Returned rows keep their producer
scope, which may be empty or identify a different execution.
Concurrent readers are independent. Each record and sample has `.to_json()`,
including nested datetime values as RFC 3339 text; no custom encoder is needed.
Use an `asyncio.TaskGroup` for concurrent record and sample consumption. Close an
early-abandoned iterator with `contextlib.aclosing`.

`await cycle.save(directory, report=None)` creates the directory and writes
`execution-<id>.zip` for every known member, `cycle.zip`, optional `report.html`,
and `manifest.json`. It waits for pending execution reports, and exposes publication
failures. `await execution.snapshot()` includes the report state and failure cause.
It returns the same JSON-serializable manifest:

```json
{"cycle_id": "...", "files": [{"path": "cycle.zip", "size": 123, "sha256": "..."}]}
```

Manifest paths are relative to the directory; each listed file has its byte
size and SHA-256. The manifest does not hash itself. Existing outputs are refused;
select a fresh destination. A failed save leaves completed files for inspection
and publishes no success manifest. Evidence ZIPs already include records and
telemetry; there is no separate local JSONL capture option. Reports require the
specified installed report definition, such as `cycle.summary`.

For explicit planning, interactive control and uncertain-admission recovery,
`client.template().materialize()`, `Execution(case)`, `client.create_cycle()`,
`cycle.append()`, `execution.start()`, `execution.wait()`, `execution.stop()` and
`cycle.seal()` remain available. Materialization snapshots the definition and
provider revision; append registers exact UUIDs atomically without starting work.
Starts are idempotent by identity and mode; mutations are never blindly retried.
`execution.stop()` waits for its durable execution seal, including any report
publication already in flight; publication failures propagate to the caller.
`execution.export(path)` and `cycle.export(path)` retain explicit `overwrite=True`.
A sealed execution can export while its cycle remains Active; cycle exports
require a durable seal. Downloads verify size, SHA-256 and final transport status
before publication. Bind the cycle again to refresh externally added membership.

## Confirmation Prompts

The default `obc.checks.pulse` has no prompt. Start its confirmation case
without waiting for completion so the script can answer:

```python
import asyncio
from contextlib import aclosing

async with DokimeClient.from_pairing("lab.dokime-remote") as client:
    async with client.new_cycle("Confirmation") as cycle:
        execution = await cycle.start("obc.checks.pulse", require_confirmation=True)
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
        outcome = await execution
```

`execution.prompt(prompt_id)` inspects the admitted
question and any committed response. Reads and responses require this execution's
identity; a prompt from a sibling execution is refused. After an ambiguous error,
inspect the question before deciding whether to submit again. Duplicate or stale
answers are refused by the server. To cancel the prompt, pass `action="cancel",
accepted=False`; stopping the execution also settles its pending prompts.

## Remote Control And Listening

Use the backend's HTTPS origin, its access key and the public certificate
downloaded from the host. Keep the key outside source files. The scripts in
[`examples/remote/python`](../../docs/remote/python.md) demonstrate manual device
power, two executions in one cycle, and an execution confirmation. The test
script iterates filtered records and samples concurrently, drains both readers
after the durable outcome, then exports each execution and the sealed cycle.
Each script has editable globals in the same order as the
[native shell examples](../../docs/remote/cli.md): connection parameters,
case and execution parameters, observation filters where applicable, and file
parameters. Set `PAIRING_FILE` to the explicitly transferred pairing file, then run
without arguments:

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
drains its iterators and saves `execution.json`.
[`client/poll_run.py`](examples/client/poll_run.py) starts `TEMPLATE` in an owned
cycle and writes `cycle.json` and `summary.json`. Edit their globals,
then run the file without arguments.

Repository Python Build produces client distributions under
`target/build/<profile>/any/python/`; Python Package copies those distributions to
`target/package/<profile>/any/python/` without rebuilding. Tutorials remain in the
[source examples](../../docs/remote/overview.md). They are not packaged or attached to
releases; follow their setup instructions to prepare a matching local wheel.

```python
from dokime_client import DokimeClient

async with DokimeClient.from_pairing("lab.dokime-remote") as client:
    execution = await client.execution(execution_id)
    await execution.start()
    async for record in execution.records("obc.run.*"):
        print(record.to_json())
    print(execution.outcome.exit_code)
```

Retain execution identity and mode before Start. Resolve uncertain admission
using the same identity and mode. Closing the client, abandoning an iterator,
or reaching a local timeout leaves backend work running when using these
lower-level primitives. Use `new_cycle` for owned cleanup on every exit.
The callback-based `execution.run()` API has been removed.

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
async with DokimeClient.from_pairing("lab.dokime-remote") as client:
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

## Finite History Discovery

```python
from dokime_client import DokimeClient, Predicate, QuerySettings

async with DokimeClient(query_settings=QuerySettings(page_rows=500)) as client:
    async for execution in client.executions(
        template="example.measurement",
        where=[Predicate("status", "eq", "failed")],
        between=("2026-10-01", "2026-11-01"),
    ):
        print(execution.id, execution.status, execution.checkpoint_available)
    async for cycle in client.cycles(phase="sealed", archive="all"):
        print(cycle.id, cycle.execution_count)
```

Factories perform no I/O. Iteration pulls finite pages from the connected
backend's local retained history, defaulting to unarchived rows. Summaries
retain captured metadata and availability; they do not wait for completion.
Use canonical statuses such as `passed` or `failed`.
`since`/`until` or `between` select half-open UTC windows; date strings mean
midnight UTC. Naive datetimes are refused. Each reader permits one consumer.
`pages()` exposes rows, captured schema, effective limits, snapshot, continuation
and completeness. Reuse `reader.snapshot` through `at=` after the first response.
Close an abandoned reader with `await reader.aclose()`.

Readers also support async contexts and explicit bounded materialization:

```python
async with execution.query_records("example.event") as records:
    rows = await records.collect(max_rows=10_000, max_bytes=16 * 1024 * 1024)
print(records.complete, records.snapshot)
```

Omitted collection limits use finite `QuerySettings.max_rows` and `max_bytes`.
The byte budget charges decoded Python row objects and page metadata cumulatively,
including schema/snapshot data on empty pages; it is separate from encoded page
bytes and process memory. A limit, malformed continuation or later RPC failure
raises without returning a partial list. Narrow the selection or consume pages
when a collection exceeds its budget. Readers fetch only on demand, with no
prefetch; closing cancels an in-flight request and prevents another fetch.
`reader.complete` reports successful consumption; early item close stays
incomplete even if the fetched final page advertises `page.complete=True`.
Metadata and snapshot remain inspectable after closing. History pages may have
`schema=None`; evidence pages retain their captured schema. Aggregate results
are also available through the telemetry reader's `metadata` and `statistics`.

## Finite Evidence Queries

```python
from dokime_client import Aggregate, Predicate

async for execution in client.executions(status="failed"):
    records = execution.query_records("example.event", where={"status": "failed"})
    async for page in records.pages():
        for record in page.rows:
            print(record.id, record.details)
    samples = execution.telemetry(
        "example.wifi.link",
        fields=["rssi"],
        at=records.snapshot,
        between=("2026-10-01", "2026-10-02"),
        where=[Predicate("rssi", "lt", -50)],
        every="1s",
        agg=Aggregate.mean(),
    )
    async for bucket in samples:
        print(bucket.start, bucket.fields)
```

Existing execution handles expose the same factories. Cycle handles and cycle
summaries expose `query_records(..., execution_id=...)` for optional member
narrowing. `versions=` selects captured contract versions; omitted versions
and telemetry fields are resolved by the server. Dotted IDs remain literal.
Raw telemetry yields existing `Sample` models; reductions yield distinct
`Bucket` models with bounds, partition, counts, units and reducers. Page metadata
retains captured schema, nullability and versions even for an empty page.
Sparse empty pages may continue. `page_rows`/`page_bytes` request smaller pages;
the server advertises effective limits. `at=` accepts a query snapshot or saved
`Checkpoint`. These finite readers never follow, start, stop or seal work;
existing `records()`/`samples()` and checkpoint APIs retain their behavior.

## Typed Status And Record Schemas

Use typed statuses to refuse a new run while the backend has active work:

```python
from dokime_client import CycleStatus, DokimeClient, ExecutionStatus

async with DokimeClient.from_pairing("lab.dokime-remote") as client:
    busy = [c async for c in client.cycles(status=[CycleStatus.RUNNING, CycleStatus.PAUSED, CycleStatus.STOPPING])]
    if busy:
        raise RuntimeError("Backend has active work")
    async for execution in client.executions(status=ExecutionStatus.FAILED):
        print(execution.id, execution.status)
    async for schema in client.record_schemas():
        columns = [field.identifier for field in schema.fields]
        print(schema.record_type, columns)
```

Cycle summaries expose `CycleStatus` in `status` and an integer `running_count`;
execution summaries expose `ExecutionStatus` in `status`. Named filters accept
one enum or a sequence; existing lowercase string filters remain supported.
These are fixed snapshot values. Lifecycle transitions commit their history
projections with runtime state, so `running_count` includes Running executions
without waiting for a checkpoint. Later transitions require a fresh query.
Backends with older development cycle-query projections need a fresh storage root;
preserve existing roots rather than resetting them.
Enum `.value` strings are stable:

| Enum | Members And Values |
| --- | --- |
| `CycleStatus` | `UNSPECIFIED="unspecified"`, `READY="ready"`, `RUNNING="running"`, `PAUSED="paused"`, `STOPPING="stopping"`, `COMPLETED="completed"`, `FAILED="failed"`, `BLOCKED="blocked"`, `STOPPED="stopped"` |
| `ExecutionStatus` | `UNSPECIFIED="unspecified"`, `INVALIDATED="invalidated"`, `PENDING="pending"`, `RUNNING="running"`, `PASSED="passed"`, `FAILED="failed"`, `BLOCKED="blocked"`, `CANCELLED="cancelled"`, `INTERRUPTED="interrupted"`, `SKIPPED="skipped"` |

`UNSPECIFIED` represents an unspecified returned status and cannot be used as a
filter. The busy check observes a snapshot; concurrent clients may start afterward,
and the backend still controls Start admission.

`record_schemas(page_rows=100)` returns a lazy async iterator of exported
`RecordSchema` objects with `record_type`, `schema_version`, optional `label` and
an ordered tuple of `RecordField` objects. Fields expose `identifier`,
`value_type: RecordValueType`, optional `label`, optional `unit` symbol, `required`
and `nullable`. Literal identifiers, including dots, retain declaration order for
CSV headers; columns are never inferred from observed values. These are current
catalog declarations; query page metadata describes captured historical schemas.
Catalog changes can abort discovery; closing the iterator stops further fetches.

`RecordValueType` has stable lowercase values `unspecified`, `boolean`, `integer`,
`number`, `text`, `file`, `object`, `array` and `datetime`, with uppercase member
names. Object/array declarations describe the top-level field only.

## Optional Dataframes

Given an execution handle or discovery summary, convert record or telemetry
queries with either [optional extra](#install). Construct a fresh reader for each
conversion; item/page/collect/aggregate/dataframe consumption is exclusive.

```python
async with execution.telemetry("example.stream", fields=["counter", "cpu.load"]) as samples:
    frame = await samples.to_polars(max_rows=10_000, max_bytes=16 * 1024 * 1024)
    metadata = samples.metadata
    assert metadata is not None and metadata.schema is not None
    print(metadata.schema.fields, metadata.schema.versions)
    print(samples.snapshot, samples.complete)

async with execution.query_records("example.event") as records:
    frame = await records.to_pandas(max_rows=10_000)
    print(frame.attrs["dokime"])
```

Both helpers use finite `QuerySettings.max_rows`/`max_bytes` when omitted. The byte
budget charges decoded rows and page metadata as for `collect`; dataframe
allocation is an additional client cost, not a server dependency or a process
memory ceiling. Limits and later page failures raise without returning a partial
frame. `QueryDependencyError.extra` identifies a missing library and gives its
installation command before fetching any page; the reader remains available for
typed iteration.

Rows keep query order. Raw frames begin with `@id`, `@timestamp`, `@received_at`,
`@source`, `@schema_version`, plus records' `@severity`, followed by captured
schema fields in their exact order with literal IDs, including dots. Bucket
frames begin with `@start`, `@end`, `@execution_id`, `@source`, `@sample_count`,
then the captured reduction fields. These envelope columns describe the frame;
telemetry `fields=` still selects declared stream fields.

| Captured Type | Polars | Pandas |
| --- | --- | --- |
| Int64 / Uint64 | Int64 / UInt64 | Nullable Int64 / UInt64 |
| Integer (Signed/Unsigned Union) | Int128 | `object` (Exact Python Integers) |
| Float64 | Float64 | Nullable Float64 |
| Boolean | Boolean | Nullable boolean |
| Text / Severity | String | string |
| Datetime | Datetime(ns, UTC) | datetime64[ns, UTC] |

Empty and all-null columns retain these dtypes. Integers above 2^53 stay exact;
there is no float intermediate. Missing and explicit null values share a nullable
frame cell; use typed iteration to distinguish them. Timestamps normalize to UTC
without discarding nanoseconds. Nested Object/Array fields, mismatched values and
timestamps outside the signed nanosecond range raise `QueryConversionError`
with its `field`, including unsupported types in an empty selection. Narrow the
projection or use typed rows; values are never stringified to make conversion fit.

Pandas `frame.attrs["dokime"]` contains `schema`, `units`, `versions`, `projection`,
`snapshot` and `complete`. For Polars, keep the originating reader: its
`metadata.schema.fields` carries ordered field types, nullability and units,
`metadata.schema.versions` carries captured versions, and `snapshot`/`complete`
remain accessible after closing. This metadata comes from the captured contract,
including when no rows match.

## Aggregate Queries

Telemetry statistics use one awaited server operation over raw selected samples:

```python
stats = await execution.telemetry(
    "example.wifi.link",
    between=("2026-10-01", "2026-10-02"),
    versions=[1],
).aggregate("rssi", [Aggregate.mean(), Aggregate.percentile(95), Aggregate.max()])
for value in stats.values:
    print(value.name, value.value)  # mean, p95, max, in request order
print(stats.selected_rows, stats.quality_counts, stats.snapshot)
```

The server computes all outputs together at one cut; Python performs no local
history reduction. `stats.schema` retains the returned captured output types,
units and versions, including empty/all-null populations. Quality counts report
finite, missing, null and non-finite inputs. Field count counts finite numbers;
`.aggregate("", [Aggregate.count()])` counts selected rows. Duplicate outputs,
strings such as `"mean"`, nonintegral percentile ranks and bucket settings are
refused locally. Statistics and item/page/collect consumption are exclusive.
`QueryLimitError` reports work/resource exhaustion without partial statistics;
query exceptions retain RPC status, typed details, metadata and native cause.

This facade constructs the generated `dokime.query.aggregate_samples` request;
raw `client.call` still exposes its canonical contracts. The existing catalog
`dokime.query.aggregate` remains a separate grouping operation, using the same
`Aggregate` vocabulary for catalog columns rather than stream samples.

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
