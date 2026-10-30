# Control The Simulated OBC From Another PC

Use Dokime's simulated on-board charger (OBC) to learn remote device control,
test execution, confirmation prompts and evidence inspection without hardware.

The **host PC** runs Dokime, its server and the OBC plugin. The **remote PC**
runs the CLI or Python examples. A case describes a test; an execution is one
attempt at that case. A cycle groups attempts and their saved evidence.
The host can have several Active cycles, with one execution running at a time.

This is a **candidate-artifact walkthrough**. No matching published Dokime release
is claimed here. Obtain one complete CI candidate as described below; its native
installation and workflow checks must pass before treating it as validated.
A version string alone does not identify compatible candidate bytes.

## Obtain One Matching Candidate

Open [Dokime's GitHub Actions](https://github.com/stokker-technologies/dokime/actions)
and select the maintainer-designated pull-request run for the candidate revision.
Record its full commit SHA. Use a completed successful run with successful
**Package / Rust** jobs for the required platforms.
An artifact appearing on a failed or unfinished run does not establish readiness.

Download these artifacts from that same run:

| PC Or Content | Artifact Name | File Inside The Download |
| --- | --- | --- |
| Windows x86-64 installer and host plugin | `package-dev-rust-windows` | `package-dev-rust-windows.tar` |
| Linux x86-64 installers and host plugin | `package-dev-rust-linux` | `package-dev-rust-linux.tar` |

GitHub wraps each artifact in a ZIP. Extract that ZIP, then extract its TAR,
for example `tar -xf package-dev-rust-windows.tar` on Windows or
`tar -xf package-dev-rust-linux.tar` on Linux. These are archive extraction
commands, not source-build commands. The extracted `target/` directory is
packaged content; no repository checkout, Rust compiler, mise or development
server is needed.

Use the version shown in the candidate artifact filenames in place of `<version>`.
Find these files after extraction:

| Platform | Installer | Matching Host Plugin |
| --- | --- | --- |
| Windows | `target/package/dev/windows-x86_64/rust/installers/dokime-<version>-x86_64-pc-windows-msvc-setup.exe` | `target/package/dev/windows-x86_64/rust/plugins/plugin-host.dokime` |
| Debian/Ubuntu x86-64 | `target/package/dev/linux-x86_64/rust/installers/dokime_<version>_amd64.deb` | `target/package/dev/linux-x86_64/rust/plugins/plugin-host.dokime` |
| RPM-based Linux x86-64 | `target/package/dev/linux-x86_64/rust/installers/dokime-<version>-1.x86_64.rpm` | `target/package/dev/linux-x86_64/rust/plugins/plugin-host.dokime` |

The installer includes the host telemetry bundle. OBC is a source-built test
fixture, not a candidate download; build it with `mise run build:rust:tests`
and transfer `target/build/dev/<target>/fixtures/plugins/plugin-obc.dokime`
when running the OBC walkthrough below.

Tutorials and Python distributions are not release downloads. Check out the
recorded candidate SHA and copy `packages/dokime-client/examples/remote/` to
`remote-demo/` on the remote PC. For Python, follow the
[local wheel setup](python.md#prepare-the-environment) before transferring
the complete tree. Keep its `cli/`, `python/` and prepared `wheels/` directories together.
On Windows, set `$INSTALL_DOKIME = $false` in `cli/bootstrap.ps1` for this
unreleased candidate, then run `cli/bootstrap.bat` (or `cli/bootstrap.ps1`)
to install PowerShell 7.6.7, mise and uv without elevation. The default
`INSTALL_DOKIME=true` setup uses mise to install the exact official GitHub
release asset for the Dokime CLI; this walkthrough instead uses the matching candidate installer.
Follow the
[Windows bootstrap instructions](cli.md#bootstrap-windows-tools),
then open a new terminal. These are optional client-tool prerequisites;
the native Dokime installation itself does not need mise.

Use the plugin for the **host's** OS. A Windows remote PC can control a Linux
host and vice versa; obtain both native artifacts from the same revision when
needed. Keep the original archives and commit SHA with your test notes.
CI artifacts expire, so download the complete set together. When a matching
release combination is published, use that release's own documented artifacts
and verification instructions instead of mixing candidates and releases.

## Install Dokime

Install the matching native package on both PCs if using CLI scripts. The
Python-only remote path needs the examples and Python environment, not a local
Dokime server.

On Windows, run `dokime-<version>-x86_64-pc-windows-msvc-setup.exe` and complete installation.
The installer adds its directory to your user `PATH`. Close existing terminals
and open a new one afterward. The installed CLI and GUI stay together; each embeds the server library;
do not move only `dokime.exe` out of that directory.

On Debian/Ubuntu, open a terminal in the extracted `installers/` directory:

```sh
sudo apt install ./dokime_<version>_amd64.deb
```

On an RPM-based system with DNF, from its `installers/` directory:

```sh
sudo dnf install ./dokime-<version>-1.x86_64.rpm
```

Use your package manager to resolve the package's native desktop dependencies.
These candidates target Linux and Windows x86-64; this guide does not establish
support for another architecture or a minimum OS release.

On either PC, verify command discovery in a new terminal:

```sh
dokime --version
```

Launch **Dokime** from the host's application menu. Keep that host application
open throughout the walkthrough. The remote PC only needs terminal commands;
launching another local server there is unnecessary.

## Install The Host Plugin

On the host, open **Settings → Plugins**. Under **Install A Plugin**, choose
**Choose File**, then select the host-platform `plugin-obc.dokime` from the
candidate's `plugins/` directory. Selecting the bundle installs it directly.

Choose **Restart Now** in the restart dialog. The installed plugin activates
when the host restarts. Return to Plugins and check that the
installation is enabled and its programs loaded without an error. The simulated
plugin needs no physical device or institute secrets.

Do this before starting any tests: a host/server restart stops work and seals
its Active cycles.

## Configure Host Remote Access

Use the host's connected local desktop session, then open
**Settings → Remote Access**.

1. Find the host's LAN IP address in its operating system's network settings.
   The examples below use `192.168.1.50`; replace it with that actual address.
   `127.0.0.1` on the remote PC would refer to the remote PC itself.
2. Enable remote access, TLS and authentication. Set the bind address to
   `0.0.0.0:8766` and certificate identities to the host's actual IP or DNS name.
   Apply the settings and wait for a successful listening state.
3. Allow that TCP port through the host firewall for the intended lab network.
4. Choose **Create Pairing File**, enter a client label and the reachable
   advertised endpoint, such as `https://192.168.1.50:8766`. Save `lab.dokime-pair`.

Transfer that pairing file privately to `remote-demo/` on the remote computer.
It contains a one-time access key and public CA certificate; keep the file private.
The host retains only a verifier. Create a separate pairing for each client, and
revoke a lost client's entry in **Paired Clients**. Revocation leaves other clients usable.

## Connect From The Remote PC

Import the pairing once into the native registry; its credential goes to the OS keyring:

```sh
dokime backend add --pairing lab.dokime-pair --identifier lab
dokime status --backend lab --json
dokime operation list --backend lab
dokime catalog list --backend lab --json
```

Status must report `ready`. Discovery must include `obc.device.read`,
`obc.device.set_power` and the `obc.checks.pulse` template. The client checks both
the TLS certificate and expected backend identity before calling application operations.
Python scripts use `DokimeClient.from_pairing` and keep the credential only in
that client's transport. Importing a pairing does not connect or change the default backend.

## Run The Three Workflows

Choose one family:

- [Native CLI scripts](cli.md): POSIX shell, PowerShell 7.3 or newer,
  or Command Prompt. PowerShell 7 is a separate program from Windows PowerShell;
  see Microsoft's [installation instructions](https://learn.microsoft.com/en-us/powershell/scripting/install/install-powershell-on-windows).
- [Python scripts](python.md): supported CPython versions are
  `>=3.13,<3.16`, with a uv environment and a locally built matching wheel.

Edit the globals at the top of each selected script. Native CLI scripts use
`BACKEND_IDENTIFIER = lab` after the import above. Python scripts use
`PAIRING_FILE = Path("../lab.dokime-pair")` when run inside `python/`.
Paths resolve relative to the terminal's working directory. Run scripts without arguments.

`control-device` reads the manual device, powers it on, verifies its 230 V RMS
state, powers it off and reads zero volts. Fixed JSON files in the current
directory contain the five replies. This manual device is separate from each execution's simulation;
manual commands are not logged as execution evidence.

The device CLI preserves existing response files through its native output
policy; use a fresh working directory for another run. Python replaces its
fixed response files when rerun.

`control-test` checks evidence destinations, materializes one case, appends two
independent attempts to a new cycle, then spells out each run and export in order.
Its default configuration produces two attempts of eight pulse samples each.
The client waits for the durable outcomes and final listener drain; Python
prints the returned records and samples after each run, while the CLI prints them
live. The script downloads `execution-1.zip`,
`execution-2.zip` and `cycle.zip` into `test-evidence/`.

`control-prompt` creates a separate confirmation attempt, starts it without
blocking, displays its pending question and answers the exact prompt UUID with
`ANSWER` (default `yes`). Sampling begins after confirmation. It downloads
`execution.zip` and `cycle.zip` into `prompt-evidence/`. A `no` answer
produces a nonpassing execution outcome. Inspect that outcome rather than
treating the demonstration script's exit status as an aggregated verdict.

Use a new evidence directory before rerunning, or explicitly choose overwrite.
Shell error handling can stop a script before its later export commands; use
the printed identities to inspect and export that attempt. Do not resubmit an uncertain
mutation just because a response or connection was lost.

## Stop And Recover

Ctrl+C in a test or prompt workflow requests scoped cancellation and evidence
recovery. The script's own cycle is sealed, cancelling its unstarted members;
other clients' cycles and the shared server remain available. Native scripts
write recovery bundles under `recovery/`; Python saves collected evidence in
its configured output directory.

The [CLI guide](cli.md#interruption-and-recovery) states the exact batch
limitations: connection setup, gaps between commands and an unknown cycle after
a lost Create reply cannot receive universal automatic cleanup. Keep every
printed cycle/execution identity and inspect the same server after reconnecting.
For manual device interruption, read its state before resending.

Explicit execution Stop durably sets its cycle's
`continue_automatically=false`. A later explicit Start runs only its selected
attempt until an operator explicitly re-enables continuation. Closing a remote
client does not shut down the host. Closing a GUI-managed host server does:
download evidence before shutting that host down. Saved evidence can be fetched
after restarting or reconnecting to the same server without rerunning hardware.

## Inspect Saved Evidence

Extract one evidence ZIP into its own new directory, for example
`unpacked-test/`. It contains `manifest.json`, `evidence.sqlite`,
`files/` when the execution captured files, and saved HTML under `reports/`.
Open a report in a browser to inspect the saved document.

The CLI/Python download already checks the archive's size and SHA-256 before
publishing it. Check that the manifest has `schema_version: 1` and
`complete: true`. It also lists each payload's `path`, `size_bytes`
and `sha256`. To compare the database payload manually, open its manifest entry
and run `sha256sum unpacked-test/evidence.sqlite` on Linux, or
`Get-FileHash .\unpacked-test\evidence.sqlite -Algorithm SHA256` in PowerShell.
Compare against the **database payload's** digest, not the whole ZIP digest.

Install the [SQLite command-line tools](https://sqlite.org/download.html) or use
an existing SQLite browser. Use SQLite 3.37 or newer: the database uses
[STRICT tables](https://sqlite.org/stricttables.html). Open the extracted file
read-only:

```sh
sqlite3 -readonly unpacked-test/evidence.sqlite
```

Run these queries; they use the exported schema directly:

```sql
SELECT cycle_id, phase FROM cycles;
SELECT execution_id, cycle_id, status FROM executions;
SELECT execution_id, direction, identifier, source_kind, value, unit_json
FROM inputs ORDER BY execution_id, direction, identifier;
SELECT execution_id, record_type, count(*) AS records
FROM records GROUP BY execution_id, record_type;
SELECT execution_id, stream_id, schema_version, sample_count, table_name
FROM streams ORDER BY execution_id, stream_id;
SELECT s.execution_id, s.stream_id, c.field_id, c.table_name,
       c.column_name, c.value_type, c.unit_json
FROM streams AS s JOIN stream_columns AS c ON c.base_table = s.table_name
ORDER BY s.execution_id, c.table_name, c.column_name;
SELECT execution_id, relative_path, archive_path, size_bytes, sha256
FROM outputs ORDER BY execution_id, ordinal;
```

The default successful pulse has eight rows in `obc.pulse` per execution and
one `obc.run.started` and `obc.run.completed` record each. Interrupted attempts
retain their truthful outcome and whatever rows were saved. An execution bundle
taken before cycle sealing can show an Active cycle; the final cycle bundle
shows its sealed phase and all members.

Sample table names and field column names are generated. Do not assume a fixed
name. This query prints a sample-reading statement for each actual pulse table;
copy the returned statement back into SQLite:

```sql
SELECT 'SELECT m_sequence, sample_json FROM "' ||
       replace(table_name, '"', '""') || '" ORDER BY m_row LIMIT 8;'
       AS query_to_copy
FROM streams WHERE stream_id = 'obc.pulse';
```

The mapping above identifies each typed field and unit; `sample_json` retains
the complete saved sample. Wide streams use several tables joined by `m_row`.
Rows in `outputs` identify captured files by their archive paths and digests.
A pulse need not capture a separate file. Keep the manifest, database, reports
and captured files together. Use `.quit` to leave the SQLite CLI.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| No package artifact or missing installer/examples | Use the same successful candidate run and inspect Package jobs. Expired or partial artifacts are not a complete candidate; ask its maintainer for a complete set. |
| `dokime` is not found | Reopen the terminal after Windows installation; check `where dokime` on Windows or `command -v dokime` on Linux. Repair the native installation rather than moving only its CLI binary. |
| Linux package dependencies cannot resolve | Use the package manager for the matching DEB/RPM and architecture. Retain its error; the candidate does not establish compatibility with every distribution release. |
| PowerShell rejects a script or JSON argument | Run PowerShell 7.3+ (`pwsh --version`), not Windows PowerShell 5.1. Follow your organization's script-execution policy. Batch is available without PowerShell 7. |
| Status is unreachable | Keep the host running; check **Listening On**, the host IP, selected port, firewall and network route. A remote client must not use the bind address `0.0.0.0` or its own loopback address. |
| Certificate verification fails | Match the endpoint's IP/DNS name to **Certificate DNS Names And IP Addresses**. After certificate replacement, create and import a new pairing. Do not disable verification to hide a mismatch. |
| Authentication fails | Check that the paired client has not been revoked. Create and import a new pairing for that client if needed. |
| OBC operations/template are missing | Install the host-OS plugin from the same candidate, restart the host and check Plugins for loading errors. |
| Existing evidence destination | Select a new evidence directory or explicitly choose overwrite. Preserve previous results you need to retain. |
| A script timed out or disconnected | Retain its printed UUIDs; reconnect to the same host and inspect. Remote work may continue. Rerunning a script would create new attempts. |
| SQLite reports a schema error | Confirm the database was extracted fully and use SQLite 3.37+. Compare its SHA-256 with the manifest before interpreting data. |

For an institute plugin, install its matching host bundle and follow its own
settings and hardware instructions. Discover its operation IDs and templates,
then substitute those contracts in the examples. The OBC tutorial supplies no
institute secrets or hardware settings.
