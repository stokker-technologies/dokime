# Run The Remote Examples With Python

Use the three editable Python scripts to control the simulated OBC, run two
attempts with record and sample readers, and answer a confirmation. Begin with the
[shared walkthrough](../README.md) to obtain the matching candidate artifacts,
install/configure the host plugin and transfer the public certificate and key.

This environment uses the **bundled candidate wheel**, not a published-registry
claim. Its `dokime-client==0.1.0` dependency resolves to
`../wheels/dokime_client-0.1.0-py3-none-any.whl`. The accompanying `uv.lock` records
that wheel's exact SHA-256 and the dependency versions. Keep the entire
extracted examples tree together; a copied script by itself is not the locked
tutorial environment.

## Prepare The Environment

Use CPython `>=3.13,<3.16` on Windows or Linux x86-64. Python 3.13 is the
baseline used below. On Windows, set `$INSTALL_DOKIME = $false` in
`../cli/bootstrap.ps1`, then run [`../cli/bootstrap.bat`](../cli/bootstrap.bat)
to install PowerShell 7.6.7, mise and uv for your user without the optional CLI;
see the
[Windows setup guidance](../cli/README.md#bootstrap-windows-tools).
On Linux, install [uv](https://docs.astral.sh/uv/getting-started/installation/)
using its official installer or your normal package manager. Open a new
terminal after setup and check:

```sh
uv --version
```

From the extracted `remote-demo/python/` directory, on either platform:

```sh
uv python install 3.13
uv sync --locked --python 3.13
uv run --locked python -c "from dokime_client import DokimeClient, Execution; print('Dokime Candidate Imports Ready')"
```

uv creates a local `.venv/`; activation is unnecessary. The import check opens
no server connection and runs no device work. The first setup needs internet
access for Python and locked third-party dependencies, or those inputs already
in uv's cache. The bundled Dokime wheel is portable Python, but this ZIP is
not a complete offline wheelhouse. Installing this client does not require a
Rust compiler or a Dokime source checkout.

Do not replace the wheel with an unrelated package from a registry or regenerate
the lock to suppress a candidate hash mismatch. Obtain the complete matching
examples artifact again. A future published-release tutorial must carry that
release's own verified dependency selection.

## Edit Connection Globals

Open each `control-*.py` file in a text editor. Under **Connection Parameters**,
replace the example IP with the host's actual certificate identity:

```python
ENDPOINT = "https://192.168.1.50:8766"
CA_CERTIFICATE: Path | None = Path("../server.pem")
KEY_FILE: Path | None = Path("../remote.key")
ENV_FILE: Path | None = None
```

The shared walkthrough places `server.pem` and `remote.key` one directory above
`python/`. Run from this directory so those relative paths resolve. Keep
`Path` imported as supplied by the scripts. The endpoint speaks native gRPC;
its `https://` scheme enables verified TLS.

Secret precedence is explicit `KEY_FILE`, process `DOKIME_KEY`, then
`DOKIME_KEY` in an explicitly selected `ENV_FILE`. Explicit files are validated
even when another source supplies the key. For dotenv instead of a key file,
copy the parent `.env.example` to `.env`, fill it privately, set
`KEY_FILE = None` and `ENV_FILE = Path("../.env")`. No parent-directory search
or environment overwrite occurs. Keep actual keys out of script source.

The same global names, grouping and meanings appear in the
[native CLI scripts](../cli/README.md). Python uses `Path` objects for files,
tuples for filters, `None` to omit an optional file, and `True`/`False` for
boolean choices. These are edit-and-run examples; they accept no CLI arguments.

## Control The Manual Device

In `control-device.py`, optionally set `RESPONSE_DIRECTORY` to a new path,
such as `Path("device-responses")`. Its default `None` prints replies only.

```sh
uv run --locked control-device.py
```

The script discovers OBC operations, reads the manual device, powers it on,
reads its powered state, powers it off and performs a final read. The expected
voltage is 230 V RMS when enabled and zero when disabled. Optional files are
`before.json`, `power-on.json`, `running.json`, `power-off.json` and
`after.json`; Python stores JSON objects with protobuf field names.

An existing response directory is refused before connecting. A local write
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
`WAIT_SECONDS` is each Run's positive deadline. Filters are case-sensitive
whole-identifier globs: `*` matches any sequence, `?` one character, and
alternatives in a tuple are OR. An empty tuple observes all records or streams
for its supplied reader. Filters affect observation, not captured evidence.

```sh
uv run --locked control-test.py
```

Read the script from top to bottom. Its only coroutine, `run()`, lets Python
await the client calls; there are no workflow helpers. It checks evidence
destinations, materializes one immutable case, constructs two attempts with
separate execution UUIDs, creates a cycle and appends without starting hardware.
The first explicit `first.run(...)` blocks until the saved outcome and reader
drain. Client callbacks put records and samples into queues; the script then
prints each returned item, `Saved Outcome:` and `Drained Evidence:`, and saves
that attempt. The second explicit `second.run(...)` repeats those calls for the
other attempt. Data-reading loops print observations, never dispatch steps.

The script saves `test-evidence/execution-1.zip`,
`test-evidence/execution-2.zip` and the explicitly sealed
`test-evidence/cycle.zip`. The default successful attempts each capture eight
pulse samples. Nonpassing saved outcomes still export and return a nonzero
process status. Set a fresh `OUTPUT_DIRECTORY` before rerunning or explicitly
choose `OVERWRITE = True` to replace bundle destinations atomically.

The cycle defaults to `continue_automatically=False`, so the script controls
each explicit Start and saves one attempt before starting another. Explicit
execution Stop durably sets that policy to false; a later explicit Start starts
only the selected attempt until the operator explicitly re-enables continuation.

## Answer One Confirmation

In `control-prompt.py`, review `TEMPLATE`, `CYCLE_LABEL`, `ANSWER`,
`WAIT_SECONDS`, `OUTPUT_DIRECTORY` and `OVERWRITE`. The default `ANSWER`
is `"yes"`; `"no"` gives a truthful nonpassing result.

```sh
uv run --locked control-prompt.py
```

The script materializes one case with `require_confirmation=True`, appends
its attempt and explicitly Starts it without blocking. It waits for that
execution's pending prompt through one commented, bounded wait because the
Python client only lists pending prompts. It prints its question and answers
the exact prompt UUID once. Sampling begins after confirmation. It then waits, saves
`prompt-evidence/execution.zip`, seals its cycle and saves
`prompt-evidence/cycle.zip`.

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

Follow [Inspect Saved Evidence](../README.md#inspect-saved-evidence) for the
actual SQLite tables, field/unit mapping, records, sample counts, captured-file
checksums and saved HTML reports. Keep the host running until downloads finish;
its saved evidence can also be retrieved after reconnecting or restarting it.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| uv cannot find a supported Python | Run `uv python install 3.13`, then sync with `--python 3.13`. Supported versions are 3.13, 3.14 and 3.15, not an unrestricted future version. |
| The local wheel path is missing | Extract the complete examples ZIP. `python/` and `wheels/` must remain siblings. |
| A locked wheel hash differs | Obtain the complete same-candidate artifact again; do not relock arbitrary replacement bytes. |
| Third-party package download fails | First setup needs registry/network access or cached locked dependencies. This bundle includes the Dokime wheel, not all dependency wheels. |
| `server.pem` or `remote.key` is not found | Run inside `remote-demo/python/` and check the edited paths and actual transferred filenames. |
| A secret/certificate is refused | Follow the shared [connection troubleshooting](../README.md#troubleshooting), including process-key precedence and certificate identity. |
| There is no record or sample output | Confirm the installed matching OBC plugin and the configured glob selections. A queued attempt may be waiting for another client's running execution. |
| Previous evidence blocks a run | Select a new output directory or explicitly enable overwrite; preserve prior bundles. |

For a different institute plugin, follow its installation/settings instructions
and substitute its discovered template and parameters. Keep the same
materialize → construct → append → Start/Run → seal/export lifecycle.
