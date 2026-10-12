# Dokime Python SDK

The `dokime` distribution contains Python authoring, the native runtime bridge,
and the async remote client under `dokime.client`. Python 3.13 or newer is required.
The Rust binding crate owns the Python source, tests, examples, and Maturin build.
The shared Rust client owns connection negotiation, authenticated RPCs, bounded
stream readers, and cancellation. Python encodes and decodes the generated
protobuf messages; it does not create a second gRPC channel. Unary timeouts
propagate to the server, and closing a client joins its readers without stopping
remote executions. The Rust client also verifies artifact and report framing,
identity, byte count, SHA-256, and final gRPC status before atomically publishing
files. Interrupted downloads retain a `.part` file and preserve any previous
successful destination.

## Installation

Install a wheel built for your platform:

```sh
pip install path/to/dokime-<version>-<platform>.whl
```

From a checkout, `mise run prepare:python` builds and installs the validation wheel.
`mise run build:python:release` builds the source archive, then builds the wheel
from that archive through uv's default build flow. Artifact verification compares
every Python source, stub, and typing marker against the checkout and requires
one nonempty native extension. Installed-wheel acceptance separately exercises
the actual native module and CLI. Source builds require
Rust and the repository's native dependencies. The `dokime-client` command is
installed by this same distribution; no separate client package is required.

## Domain Handles

Python follows the backend's ownership boundaries:

| Handle | Responsibility |
| --- | --- |
| `DokimeClient` | Connection, discovery, cycle creation and durable request lookup. |
| `Cycle` | Cycle controls, prompts, live records/streams, execution membership and reports. |
| `TestExecutionHandle` | Original native execution identity, snapshots, interactive commands and controls. |
| `ExecutionCommandReceipt` | Immutable progress for one identified command; distinct from execution sealing. |
| `RunRequest` | One admitted operation and its durable progress/completion receipt. |
| `ExecutionEvidence` | One immutable historical execution checkpoint. |
| `Capture` | Client-local files, admission recovery, collection and downloads. |

A request is neither a cycle nor an execution. One cycle can contain several
cases and execution attempts; another admission can run it again. `Session`
already describes backend context and is not a synonym for a request or a
client connection. The client does not invent a second session abstraction.
Modules reflect these boundaries: `execution/cycle.py`, `execution/request.py`,
`history.py`, and the local `capture/` workflow.

## Connected Execution Controls

`await client.execution(execution_id)` binds a native handle after connecting;
it performs no execution lookup or device work. Stored `TestExecution` values
and connected handles have separate ownership.

```python
handle = await client.execution(execution_id)
request = await handle.start(request_key=request_key, mode="interactive")
command = await handle.next(request_id=request.id, command_id=command_id)
receipt = await handle.command_receipt(request.id, command_id)
```

Retain the execution ID, request key, request ID and command ID for recovery.
`run_step(step_id, request_id=..., command_id=...)` and
`finish(request_id=..., command_id=...)` use the same native command path.
Receipt lookup never resubmits a command. Its `terminal` property describes that
command, not execution sealing; `Interrupted` does not become `Completed`.
`pause()`, `resume()`, `stop()` and `snapshot()` verify the original live epoch.
Closing the client cancels local work while remote execution remains independent.

## Capture With Short Concurrent Listeners

This self-contained program uses an existing backend and registered suite and
template. Replace the identifiers and put template parameters in a JSON object
in `parameters.json`. The output directory must be new.

```python
import asyncio
import json
from pathlib import Path

from dokime.client import Capture, Cycle, DokimeClient, RunRequest


async def listen_records(request: RunRequest, cycle: Cycle) -> None:
    async for record in cycle.records(until=request):
        print(record.type, record.id, record.sequence, record.severity)


async def listen_streams(request: RunRequest, cycle: Cycle) -> None:
    async for sample in cycle.samples(until=request):
        print(sample.stream, sample.id, sample.sequence)


async def listen_status(request: RunRequest) -> None:
    async for status in request.updates():
        print(status.phase, status.finished, status.planned)


async def show_progress(capture: Capture) -> None:
    async for update in capture.updates():
        print(update.message)


async def main() -> None:
    parameters = json.loads(Path("parameters.json").read_text())
    client = DokimeClient("http://127.0.0.1:8765")

    try:
        capture = await Capture.prepare(
            client,
            output="capture-001",
            suite="YOUR_SUITE",
            template="YOUR_TEMPLATE",
            parameters=parameters,
        )

        request = await capture.start()
        cycle = await client.cycle(request.cycle_id)

        async with asyncio.TaskGroup() as tasks:
            tasks.create_task(listen_records(request, cycle))
            tasks.create_task(listen_streams(request, cycle))
            tasks.create_task(listen_status(request))
            tasks.create_task(show_progress(capture))

            result = await capture.collect(request, report="cycle.summary")

        print(result.result.status)

    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
```

`prepare()` validates supplied values, reserves the directory, creates the cycle,
and saves its ID and admission key before device work is submitted. `start()`
submits or resolves that saved key. `collect()` concurrently drains records and
samples through the request's terminal receipt, then exports sealed execution
evidence, traceability, artifacts, and the streamed report. `capture.result`
is available only after successful collection; a failed test can still have a
complete capture. Its `request` field describes acquisition; its `result` field
contains backend cycle result counters.

Edit the listener bodies to handle each event record, stream sample, or changed
request status. Events are exposed as records; there is no separate event
subscription on `Cycle`. `until=request` keeps data listeners alive through
completion and final draining. These listeners independently replay the same
data that `Capture` saves, so filtering a listener does not filter saved evidence.
On recovery, listeners replay earlier data too; use record/sample IDs to
deduplicate external side effects. Listener failures propagate through the task
group and cancel local collection without automatically stopping remote work.

`updates()` broadcasts coarse progress, with up to 32 recent updates replayed
for late listeners. Slow listeners may miss intermediate updates; they never
block acquisition and always see the final stage. Each iterator closes on
collection success or failure. The library does not print, install signal
handlers, close your client, or answer operator prompts.

A cancelled local task leaves incomplete evidence and does **not** stop remote
work. Stop the claimed execution through `TestExecutionHandle.stop()` and allow
collection to finish if that is the desired policy. The installed CLI implements this policy for Ctrl-C.

Recover without creating another cycle:

```python
capture = await Capture.open(client, output="capture-001")
request = await capture.start()
await capture.collect(request)
```

Saved captures retain format version 1 and existing CLI directories remain
readable. `.collect.lock` excludes concurrent writers. Normal failures and
cancellation remove the lock; after a process crash, remove a stale lock only
once you have confirmed no collector is still using that directory.

## Custom Observation And Control

For applications with their own persistence, use the domain handles directly:

```python
cycle = await client.new_cycle(
    suite="YOUR_SUITE", template="YOUR_TEMPLATE", parameters={}
)
# Persist cycle.id and an application-owned request_key before admission.
request = await cycle.run(request_key="lab-order-1042")

async for status in request.updates():
    if status.terminal:
        break
    if status.last_execution_id is not None:
        execution = await client.execution(status.last_execution_id)
        await execution.pause()
        await execution.resume()
        # await execution.stop()  # Cooperative cancellation of this execution.
        break
await request.wait()
result = await cycle.result()
```

The client constructor does no network IO. Call `await client.connect()` for
explicit negotiation or let the first operation connect. Always close the client
in `finally`, or use its optional async context manager.

Short independent listeners compose through a task group:

```python
async def records(request: RunRequest, cycle: Cycle) -> None:
    async for record in cycle.records(until=request):
        await store_record(record)


async def streams(request: RunRequest, cycle: Cycle) -> None:
    async for sample in cycle.samples(until=request):
        await store_sample(sample)


cycle = await client.cycle(request.cycle_id)
async with asyncio.TaskGroup() as tasks:
    tasks.create_task(records(request, cycle))
    tasks.create_task(streams(request, cycle))
    await request.wait()
```

`store_record` and `store_sample` are your application's async sinks. `until`
is an explicit completion boundary: both iterators drain their final pages
before closing. Without `until`, records read a fixed current journal frontier
and samples read until caught up; neither promises an immutable running snapshot.
`request.updates()` polls changed durable receipts and closes after its terminal
receipt. A request from another cycle or connection is rejected by the iterators.

Records support `record_types=`, `record_ids=`, and `execution_id=` filters;
samples support `streams=`, `execution_id=`, `page_size=`, and `poll_interval=`.
`record.type` identifies an authored kind; `record.id` identifies an occurrence.
The default evidence scope is the cycle, including earlier attempts. `until`
controls completion, **not** filtering to that request's execution attempts.
Use a fresh cycle for an independently controlled acquisition. The protocol
cannot atomically guard a cycle control against another caller rerunning it.

Records use `Evidence.WatchRecords` and bounded final pagination; samples use
`Telemetry.ListSamples` with forward cursors and epoch checks. Oversized replies
fail explicitly; lower `page_size` rather than accepting truncated data.
Interrupted receipts do not certify complete capture. Live replay fails after a
backend restart; historical checkpoints are a separate API.

Discover definitions and handle prompts deliberately:

```python
async for suite in client.suites():
    print(suite.id, suite.identifier, suite.templates)

async for prompt in cycle.pending_prompts():
    print(prompt.id, prompt.title, prompt.actions, prompt.fields)
# await cycle.respond(prompt_id, action="submit", values={"accepted": True})
```

The synthetic steady template requires operator confirmation; respond in the
GUI or through the prompt API before expecting completion.

## Submission And Recovery

Cycle creation is not idempotent. A failed or cancelled `new_cycle()` call can
have created the cycle remotely; inspect the backend before repeating it.
`Capture.prepare()` persists confirmed creation before admitting work, but
cannot automatically recover uncertain cycle creation either.

On `SubmissionUncertain`, resolve the same cycle and key:

```python
cycle = await client.cycle(error.cycle_id)
request = await cycle.run(request_key=error.request_key)
```

`await client.get_run_request(request_id)` recovers a durable receipt even after
backend restart. `status()` and `wait()` remain available. Resolve a cycle
separately; a request owns progress and cannot resurrect live state or resume
device actions. A wait timeout only stops
waiting. Other mutations, including controls and prompt responses, are not
automatically retried after ambiguous failures.

## Remote Capture CLI

The unified SDK also installs `dokime-client`. From a checkout:

```sh
mise run prepare:report
uv tool install ./crates/dokime-python
```

Source installations from a checkout require the generated report above. The
release source archive contains the Rust path dependencies, crate-local protobuf
sources and generated report, so consumers do not need the frontend toolchain.
Use a built wheel or source archive when installing without a checkout.

Run against an existing backend with a registered suite and template:

```sh
dokime-client run \
  --endpoint http://127.0.0.1:8765 \
  --suite YOUR_SUITE --template YOUR_TEMPLATE \
  --parameters-file parameters.json --output capture-1042
```

`--parameters-file` reads a JSON object. Alternatively use
`--parameters '{"duration_s": 1}'`; the two options are mutually exclusive.
Lists are whole parameter values, not parameter sweeps. Input is validated
before creating a remote cycle. Resolved parameters, including server defaults,
are saved in the captured execution document.

The output directory must be new. Progress goes to stderr: `Capturing Data...`,
operator-input notices, finalization, and report download. Individual records
and samples are never printed. The final result and output path go to stdout.
Answer operator prompts in the Dokime GUI. Ctrl-C or `--cancel-after 10`
requests cooperative cancellation and waits for the terminal receipt and final
capture; it does not forcefully kill the server. `--token-file` and `--ca`
configure authenticated TLS. `--report` selects a report (default `cycle.summary`).

```text
capture-1042/
├── capture.json
├── cycle.json
├── records.jsonl
├── streams.jsonl
├── executions/
│   └── <execution-id>/
│       ├── execution.json
│       ├── records.jsonl
│       ├── streams.jsonl
│       ├── traceability.json
│       ├── schemas.json
│       └── artifacts/
│           ├── manifest.json
│           └── <artifact-id>
└── report.html
```

This is client capture format version 1, not the native Delta storage layout or
an importable cycle bundle. `capture.json` stores backend/cycle/request identity,
the admission key, checkpoints, capture stage, and completeness. `cycle.json`
and `execution.json` preserve protobuf JSON field names; the execution document
contains frozen inputs and plugin/template identity. `records.jsonl` contains
one public `Record` per line. `streams.jsonl` contains one public `Sample` per
line, including stream stream, schema version, sequence, source, scope, and
values; it is not a stream-definition table. `schemas.json` retains captured
record schemas and telemetry table schemas. `traceability.json` contains the
checkpoint's captured groups, values, scopes, metadata and provenance sources.

Live listeners write root-level `.part` files, promoted after the terminal
receipt and final drain. These files retain the cycle-wide replay, including
cycle-only lifecycle records. Each execution directory separately exports its
immutable History checkpoint, which can also contain captured ancestor context.
These are different scopes and should not be concatenated blindly. Historical
issues or unsealed documents fail explicitly. Attachments are stored by artifact ID;
the manifest retains original filenames and complete/cancelled/rejected states.
Report HTML comes from `Reports.DownloadCycleReport`; attachments come from
`History.DownloadCheckpointArtifact`. Downloads validate framing, offsets,
length, SHA-256, and the final gRPC status before promoting `.part` files.

A failed download leaves `complete: false` and preserves available evidence.
Retry without starting another execution:

```sh
dokime-client collect --endpoint http://127.0.0.1:8765 --output capture-1042
```

Collection reconstructs the saved request (or resolves its original admission
key), replays evidence, and retries downloads. It requires the cycle to remain
available in the current runtime. It does not resume device actions after a
backend restart. If cycle creation itself was uncertain, inspect the backend;
there is no confirmed cycle/key to recover automatically. Exit codes are 0 for
a passed test with complete capture, 1 for another test outcome with complete
capture, 2 for invalid arguments/configuration, and 3 for operational failure.

## Self-Contained Examples

The installed entry point lives in `python/dokime/client/cli.py`; `commands/run.py`
and `commands/collect.py` own their typed arguments and execution. Shared console
options and presentation live in `_console.py`. Package builds and imports
are independent of the examples. Both examples use the public client;
`lab_run.py` also uses Rich from the optional `examples` extra. Copy either file
into your own project and edit its constants. They accept no command-line arguments.

Configure `ENDPOINT`, `SUITE`, `TEMPLATE`, and the Python `PARAMETERS` dictionary.
Set `TOKEN_FILE` and `CA_CERTIFICATE` for an authenticated TLS backend.
Both examples bind a planned execution before admission and retain its original
native client connection.

`examples/client/lab_run.py` shows `listen_records`, `listen_streams`, and
`listen_status` in a Rich display. It shows the latest record/sample identities
and native sequence numbers. Customize the listener bodies and set `RECORD_TYPES`
or `STREAMS` to filter what they consume. Ctrl-C or `CANCEL_AFTER` requests
cooperative stop; the script waits for the persisted outcome and consumer drain.
Leave `CANCEL_AFTER = None` to wait for completion.

```sh
uv run --package dokime --extra examples python crates/dokime-python/examples/client/lab_run.py
```

`examples/client/poll_run.py` counts records and samples through execution-owned
listeners, then prints the frozen result after both consumers drain.
`POLL_INTERVAL` controls native polling cadence.

```sh
uv run --package dokime python crates/dokime-python/examples/client/poll_run.py
```

The examples use the backend's native execution storage and create no local
export tree. Ctrl-C in the polling example disconnects without stopping remote
execution. Both default to the synthetic steady template; answer its operator
prompts in Dokime. Adjust suite/template identifiers and parameters for your plugins.

## Immutable Evidence API

```python
await request.wait()
cycle = await client.cycle(request.cycle_id)
for execution in await cycle.executions():
    document = await execution.document()
    traceability = await execution.traceability()
    schemas = await execution.schemas()
    async for record in execution.records():
        ...
    async for sample in execution.samples():
        ...
    for artifact in await execution.artifacts():
        await execution.download_artifact(artifact, destination)

await cycle.download_report("report.html")
```

`cycle.executions()` resolves current cycle membership into exact historical
checkpoints. Reusing a cycle includes its prior attempts. Documents retain
protobuf JSON representation (including typed values); traceability values,
records, and samples use the public Python value types. Download destinations
are caller-selected paths. Failed downloads retain `.part` files; explicit
retry code owns removing those partial files, as `Capture.collect()` implements.

## Native Release Contract

Release wheels target standard CPython 3.13 with ABI3 on x86_64 Linux and
Windows. Free-threaded interpreters and other architectures are not declared
supported. Linux requires manylinux with a glibc floor of 2.28 or earlier;
Windows requires a native MSVC build. Installed-wheel tests on each declared
platform remain mandatory.

`build:python:release` verifies the exact wheel filename and WHEEL metadata,
the extension's ELF/PE architecture, and its stable Python ABI using strict
`abi3audit`. On Linux, `auditwheel show --json` must confirm the claimed
manylinux floor, external library compatibility, and baseline instruction set.
A developer-host `linux_x86_64` wheel is refused by this release gate. Build
Linux release artifacts in a compatible manylinux environment; renaming a
wheel does not establish portability. These audits do not replace native
installation tests or the two-platform release manifest checks.

## Development

- `mise test`: socket, packaging, and native end-to-end tests with matching host and plugin builds.
- `mise run build:python:release`: native SDK wheel and source archive.
- `mise run bindings:python:generate`: regenerate private bindings from `crates/dokime-protocol/proto/`.
- `mise run bindings:python:check`: compare generated bindings with canonical sources.

Generated code keeps the canonical wire package and descriptor file names; only
Python import paths are private. Ruff excludes generated code; authored code
and examples use the repository conventions and pinned Python type checker.
