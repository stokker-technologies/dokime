# Use Python Remotely

Install the pure `dokime-sdk` wheel with Python 3.13 or newer. It provides remote
Harmos plugin serving, typed Dokime catalogs, composed documents and generated
async client stubs. See the [SDK guide](../../sdk/python/README.md).

```sh
uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl
```

## Serve a Plugin

The [starter](../../sdk/python/examples/test/) is the source embedded by
`dokime new bench --lang python`. Start Dokime separately, then serve the Python
plugin and add its printed URL to the host:

```sh
dokime new bench --lang python
cd bench
uv add /path/to/dokime_sdk-0.1.5-py3-none-any.whl
mise run test
mise run serve
```

The starter's test uses a prepared product host with a controlled clock. Set
`DOKIME_TEST_HOST` to the host fixture executable before `mise run test`. The
story answers the operator prompt, reads the retained confirmation record and
checks the evaluated outcome and plugin cleanup.

## Connect a Client

```python
from dokime.connection import connect

client = await connect("http://127.0.0.1:8765")
try:
    print((await client.plugins.list()).entries)
finally:
    await client.close()
```

Use `await dokime.connect("lab.pair")` to require the paired backend identity,
or supply HTTPS, a pinned `certificate_pem` and `access_key` explicitly.
Identity verification requires the generated backend service method; the SDK
refuses the pairing connection when that definition is absent.

## Query Retained Evidence

`dokime.evidence` finds retained executions and reads their saved evidence at one
fixed cut. These reads never start, stop, seal or follow an execution.

```python
from dokime.connection import connect
from dokime.evidence import executions

client = await connect("lab.pair")
try:
    async for execution in executions(client, template="thermal.cycle", status="sealed"):
        telemetry = execution.telemetry("thermal.temperature", fields=["celsius"])
        frame = await telemetry.to_polars(max_rows=10_000)
        print(execution.id, frame, telemetry.snapshot)
        await execution.export(f"{execution.id}.zip")
        await execution.cycle.export(f"{execution.detail.cycle_id}.zip")
finally:
    await client.close()
```

`status` selects `open`, `closed` or `sealed`; `template` and `cycle` select by
identifier. `execution(client, identifier)` and `cycle(client, identifier)` return
one handle. `telemetry(stream, fields=..., between=Window(...), at=..., page_size=...)`
pages at the cut returned by its first page; `snapshot` exposes that cut for a later
consistent read. `collect`, `to_polars` and `to_pandas` default to 100,000 rows and
raise rather than return a partial result when the selection holds more.

Install `dokime-sdk[polars]` or `dokime-sdk[pandas]` for the dataframe helpers;
`pages` and `collect` need neither. Frames hold `sequence`, a nanosecond UTC
`timestamp` and one column per field, typed from the wire value: unsigned and
signed integers stay exact as `UInt64`/`Int64`, and exact fractions or mixed kinds
keep their Python values. `TelemetryRow.timestamp_ns` keeps the exact source time.

`export(path)` downloads the host's evidence archive, verifies its length and
SHA-256 and publishes the ZIP atomically. An existing destination is refused
unless `overwrite=True`; a failed download leaves no partial file. Execution
archives hold `manifest.json`, `evidence.pb` and the unchanged telemetry files;
cycle archives hold each execution's archive and a manifest.

The [host query contract](../query-api.md) describes finite reads and
continuations.
