# Run The Remote Examples With Python

Scripts: `packages/dokime-client/examples/remote/python/`. Commands below run from that folder.

Use the three editable Python scripts to control the simulated OBC, run two
attempts with record and sample readers, and answer a confirmation. Begin with the
[shared walkthrough](overview.md) to obtain the matching candidate artifacts,
install/configure the host plugin and transfer the public certificate and key.

These source examples use a locally built wheel from the same revision as the
backend. Tutorials and Python distributions are not release downloads. Prepare
the wheel and refresh the tutorial lock before transferring the example tree.

## Prepare The Environment

Use CPython `>=3.13,<3.16` on Windows or Linux x86-64. Python 3.13 is the
baseline used below. On Windows, set `$INSTALL_DOKIME = $false` in
`../cli/bootstrap.ps1`, then run [`../cli/bootstrap.bat`](../../packages/dokime-client/examples/remote/cli/bootstrap.bat)
to install PowerShell 7.6.7, mise and uv for your user without the optional CLI;
see the
[Windows setup guidance](cli.md#bootstrap-windows-tools).
On Linux, install [uv](https://docs.astral.sh/uv/getting-started/installation/)
using its official installer or your normal package manager. Open a new
terminal after setup and check:

```sh
uv --version
```

In the matching source checkout, build the client:

```sh
mise run build:python
```

Copy `packages/dokime-client/examples/remote/` to a standalone `remote-demo/`
directory outside the checkout. Create `remote-demo/wheels/` and copy the single
`dokime_client-<version>-py3-none-any.whl` from `target/build/dev/any/python/`
into it. The version must match the tutorial's exact dependency and wheel path
in `python/pyproject.toml`. From `remote-demo/python/`, refresh the
lock for those wheel bytes, then prepare the environment:

```sh
uv lock --upgrade-package dokime-client
uv python install 3.13
uv sync --locked --python 3.13
uv run --locked python -c "from dokime_client import DokimeClient, Execution; print('Dokime Candidate Imports Ready')"
```

Transfer the complete `remote/` tree, including the refreshed `python/uv.lock`
and `wheels/`, to the remote PC as `remote-demo/`. Run `uv sync --locked --python 3.13`
there before running the scripts. uv creates a local `.venv/`; activation is
unnecessary. Setup needs internet access for Python and locked dependencies or
those inputs already in uv's cache. The remote PC needs no Rust compiler.

## Query Retained Evidence

```python
from dokime_client import Aggregate, DokimeClient, Predicate

async with DokimeClient.from_pairing("../lab.dokime-remote") as client:
    async for execution in client.executions(status="failed", since="2026-10-01"):
        records = execution.query_records("example.event", where={"status": "failed"})
        async for record in records:
            print(record.to_json())
        async for bucket in execution.telemetry(
            "example.wifi.link", fields=["rssi"],
            where=[Predicate("rssi", "gte", -80)],
            between=("2026-10-01", "2026-10-02"), at=records.snapshot,
            every="1s", agg=Aggregate.mean(),
        ):
            print(bucket.start, bucket.fields)
```

Query factories are lazy and read one fixed cut of the connected backend's local
retained evidence. Windows are `[start, end)` in UTC. Reusing `snapshot` keeps
records and telemetry at the same compatible cut without waiting for a seal.
Use `pages()` to inspect schema, effective limits and completeness, and
`await reader.aclose()` when abandoning a reader. Existing `records()` and
`samples()` still replay and follow through the execution seal. See the
[client guide](../../packages/dokime-client/README.md#finite-evidence-queries)
for projection, version selection and cycle record queries.

To materialize a finite result, close the reader through its async context:

```python
async with execution.query_records("example.event") as records:
    rows = await records.collect(max_rows=10_000, max_bytes=16 * 1024 * 1024)
```

Omitted limits use finite client `QuerySettings` defaults. Row or decoded-byte
overflow and later page errors raise without returning a partial list. Early
close/cancellation prevents another fetch and leaves backend execution running.
`records.complete` reports consumption; page metadata retains server completeness,
schema, limits and snapshot after closing. No dataframe dependency is needed.

For exact statistics on raw selected telemetry, use one awaited operation:

```python
stats = await execution.telemetry("example.wifi.link").aggregate(
    "rssi", [Aggregate.mean(), Aggregate.percentile(95)],
)
print([(value.name, value.value) for value in stats.values])
print(stats.quality_counts, stats.schema, stats.snapshot)
```

Omit `every`/`agg` for statistics. The facade uses existing `Aggregate` objects
and computes on the server, preserving typed limit failures. Raw `client.call`
continues to expose the generated request; the older catalog aggregate operation
groups catalog rows and is separate from these telemetry statistics.

## Edit Connection Globals

Open each `control-*.py` file and select the private pairing file:

```python
PAIRING_FILE = Path("../lab.dokime-remote")
```

Run from `python/` so the path resolves. Each script constructs its client with
`DokimeClient.from_pairing(PAIRING_FILE)`. The file supplies ordered endpoints, expected
backend identity, CA and access key together. The client validates the file before
connecting and checks TLS and backend identity before application calls.
It retains the key only in its transport credentials.

The remaining execution, filter and evidence parameters match the
[native CLI scripts](cli.md). These are edit-and-run examples; they
accept no command-line arguments.

## Control The Manual Device

Run `control-device.py` from the directory where its fixed response files
should be saved.

```sh
uv run --locked control-device.py
```

The script discovers OBC operations, reads the manual device, powers it on,
reads its powered state, powers it off and performs a final read. The expected
voltage is 230 V RMS when enabled and zero when disabled. The response files are
`before.json`, `power-on.json`, `running.json`, `power-off.json` and
`after.json`; Python stores JSON objects with protobuf field names.

Fixed response files are replaced on each run. A local write
failure can follow a successful device command. Ctrl+C stops waiting and
does not prove whether an unconfirmed command acted; read state before resending.
Manual control is independent of the devices owned by test executions and is
not execution evidence.

## Run Two Independent Attempts

Review `control-test.py`:

```python
TEMPLATE = "obc.checks.pulse"
CYCLE_LABEL = "Remote Pulse Demonstration"
SAMPLE_COUNT = 8
EXECUTION_COUNT = 2
WAIT_SECONDS = 60.0

RECORD_TYPES = ("obc.run.*",)
STREAMS = ("obc.pulse",)

OUTPUT_DIRECTORY = Path("test-evidence")
OVERWRITE = False
```

The story spells out two attempts of one materialized case. Set `EXECUTION_COUNT`
to `1` to study only the first attempt or leave it at `2`; add explicit calls
and evidence destinations to teach additional attempts.
`WAIT_SECONDS` is each execution's positive local deadline. Filters are case-sensitive
whole-identifier globs: `*` matches any sequence, `?` one character, and
alternatives in a tuple are OR. An empty tuple observes all records or streams
for its supplied reader. Filters affect observation, not captured evidence.

```sh
uv run --locked control-test.py
```

Read the script from top to bottom. `run()` awaits the client calls and its
local reader coroutines consume the evidence iterators. It checks evidence
destinations, materializes one immutable case, constructs two attempts with
separate execution UUIDs, creates a cycle and appends without starting hardware.
Each attempt starts explicitly. Independent async iterators consume its
records and samples inside an `asyncio.TaskGroup`; awaiting the handle retrieves
the saved outcome, and the task group waits for both final drains. The script
then prints each item, `Saved Outcome:` and `Drained Evidence:`, and saves that
attempt. The second attempt repeats this flow with its own execution identity.
Data-reading loops print observations, never dispatch steps.

The script saves `test-evidence/execution-1.zip`,
`test-evidence/execution-2.zip` and the explicitly sealed
`test-evidence/cycle.zip`. The default successful attempts each capture eight
pulse samples. The process returns the largest execution exit code: 0 for passed, 1 for
nonpassing and 130 for interrupted; exceptions report a failed workflow. Set a fresh
`OUTPUT_DIRECTORY` before rerunning or explicitly
choose `OVERWRITE = True` to replace bundle destinations atomically.

The script also saves `cycle.json`, `execution-1.json` and `execution-2.json`
in the current directory. These fixed JSON responses are replaced on each run.

The cycle defaults to `continue_automatically=False`, so the script controls
each explicit Start and saves one attempt before starting another. Explicit
execution Stop durably sets that policy to false; a later explicit Start starts
only the selected attempt until the operator explicitly re-enables continuation.

## Answer One Confirmation

In `control-prompt.py`, review `TEMPLATE`, `CYCLE_LABEL`, `ANSWER`,
`WAIT_SECONDS`, `OUTPUT_DIRECTORY` and `OVERWRITE`. The default `ANSWER`
is `"yes"`; `"no"` records a truthful nonpassing result.

```sh
uv run --locked control-prompt.py
```

The script materializes one case with `require_confirmation=True`, appends
its attempt and explicitly Starts it without blocking. It waits for that
execution's pending prompt through one commented, bounded wait because the
Python client only lists pending prompts. It prints its question and answers
the exact prompt UUID once. Sampling begins after confirmation. It then waits, saves
`prompt-evidence/execution.zip`, seals its cycle and saves
`prompt-evidence/cycle.zip`. The cycle identity and inspected prompt are saved
as `cycle.json` and `prompt.json` in the current directory.

This demonstrates scripted confirmation; it does not wait for terminal input.
Use the native CLI prompt commands if an operator needs to inspect and answer
separately. Rejected, duplicate or uncertain answers must be inspected through
their original execution and prompt identities before any manual resubmission.

## Cancellation And Inspection

Ctrl+C during test/prompt work stops the selected execution, seals the script's
own cycle and saves collected evidence in its configured output directory.
That cycle's unstarted members are cancelled; other clients' cycles and the
shared server remain available. Retain printed UUIDs if the connection fails or
recovery is interrupted. A lost cycle-creation reply leaves its identity
unknown; inspect the host before creating another container.

A normal client disconnect, reader failure or local timeout does not itself
stop admitted work. The example reports failure without retrying mutations.
To fetch evidence again, attach to the retained execution/cycle and export;
do not rerun the example to recover a download.

Follow [Inspect Saved Evidence](overview.md#inspect-saved-evidence) for the
actual SQLite tables, field/unit mapping, records, sample counts, captured-file
checksums and saved HTML reports. Keep the host running until downloads finish;
its saved evidence can also be retrieved after reconnecting or restarting it.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| uv cannot find a supported Python | Run `uv python install 3.13`, then sync with `--python 3.13`. Supported versions are 3.13, 3.14 and 3.15, not an unrestricted future version. |
| The local wheel path is missing | Extract the complete examples ZIP. `python/` and `wheels/` must remain siblings. |
| Third-party package download fails | First setup needs registry/network access or cached locked dependencies. This bundle includes the Dokime wheel, not all dependency wheels. |
| `lab.dokime-remote` is not found | Run inside `remote-demo/python/` and check the edited paths and actual transferred filenames. |
| A secret/certificate is refused | Follow the shared [connection troubleshooting](overview.md#troubleshooting), including process-key precedence and certificate identity. |
| There is no record or sample output | Confirm the installed matching OBC plugin and the configured glob selections. A queued attempt may be waiting for another client's running execution. |
| Previous evidence blocks a run | Select a new output directory or explicitly enable overwrite; preserve prior bundles. |

For a different institute plugin, follow its installation/settings instructions
and substitute its discovered template and parameters. Keep the same
materialize → construct → append → Start/Run → seal/export lifecycle.
