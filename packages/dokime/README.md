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
generation. Runtime dependencies are `grpcio` and `protobuf`; install their
compatible platform wheels as usual.

## Named Backend Profiles

Read a saved profile explicitly, supplying its bearer token separately:

```python
async with DokimeClient.from_profile("lab", token=token) as client:
    backend = await client.connect()
```

`from_profile` reads configuration once and starts no server. Use `path` to
select another profile file, or `DOKIME_PROFILES_FILE` to override the standard
OS configuration location. The Rust CLI manages these profiles through
`dokime backend add/list/show/use/remove`. Python keeps the selected endpoint,
CA and expected backend identity for that connection; later default edits have
no effect. An unexpected identity fails negotiation before workflow mutations.
Profiles marked `keyring` require the Python caller to supply a token.

## Remote Control And Listening

Use the backend's HTTPS origin, a bearer token issued for that backend, and its
CA certificate when it uses a private certificate authority. Keep tokens outside
source files. The example runs an existing planned execution and drains record
and telemetry callbacks after its durable outcome:

```sh
python examples/remote_control.py \
  --endpoint https://dokime.example:8765 \
  --token-file /path/to/token \
  --ca-certificate /path/to/ca.pem \
  --execution REGISTERED_EXECUTION_UUID \
  --request-key UNIQUE_REQUEST_KEY
```

```python
from dokime.client import DokimeClient


async def record(item):
    print(item)


async with DokimeClient(
    "https://dokime.example:8765",
    token=token,
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
those plugins in the Python distribution. Their operations require an exact
owner/operation permission on the server's existing bearer grant; see
[Plugin Operation Permissions](../../README.md#plugin-operation-permissions).
Unlisted plugin operations are denied in both discovery and invocation.

```python
async with DokimeClient(endpoint, token=token) as client:
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
