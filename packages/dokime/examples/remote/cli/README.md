# Remote CLI Workflows

Control the simulated OBC device, run independent attempts with live observations,
and answer a confirmation using an installed Dokime CLI on another computer.
Start with the [shared candidate-artifact walkthrough](../README.md) to obtain
the matching CLI and host plugin, configure the host and transfer credentials.
A matching published release combination is not claimed; native execution and
interruption behavior still require platform validation.

## Bootstrap Windows Tools

From the extracted `cli/` directory, run the batch launcher in a normal,
unelevated Command Prompt:

```bat
bootstrap.bat
```

Or start the same setup from built-in Windows PowerShell:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bootstrap.ps1
```

Both use `bootstrap.ps1`. It installs PowerShell **7.6.7** from Microsoft's
official self-contained ZIP, verifies its pinned SHA-256, installs mise using
its official Windows installer, then runs `mise use -g uv` and `mise reshim`.
It needs no administrator rights, WinGet, Git, existing `pwsh`, or repository
checkout. The execution-policy option applies only to that process; it does not
change the machine's policy or override organization-enforced Group Policy.

PowerShell lives under `%LOCALAPPDATA%\Programs\PowerShell\7.6.7-<architecture>\`.
Mise and `mise-shim.exe` stay together under `%USERPROFILE%\.local\bin\`.
The installer adds mise's bin and shims directories to your **user PATH**,
preserving existing entries. Shims default to `%LOCALAPPDATA%\mise\shims`;
existing mise directory overrides are respected. The script also adds the
PowerShell directory and appends mise activation to PowerShell 7's
`$PROFILE.CurrentUserAllHosts`, preserving the rest of that file. This loads
project environment variables at the prompt; shims serve Command Prompt and
editors. No `MISE_DATA_DIR` or other directory override is needed for defaults.

Close and reopen your terminal and editor after setup, then verify:

```powershell
pwsh --version
mise doctor
uv --version
```

If PowerShell refuses to load your profile under the default Restricted policy,
Microsoft's usual personal-machine setting is
`Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`. Review your organization's
policy before applying it. The bootstrap leaves this persistent choice to you.
Use `mise exec -- <command>` when shell activation cannot be enabled.

Re-running setup reuses the installed PowerShell version, lets the official mise
installer retain or update mise, and updates the global uv selection. It does
not automatically trust project configurations or install Dokime. To develop
from a checkout, review its `mise.toml`, then run `mise trust` and `mise install`.

Use Windows PowerShell **5.1** on Windows 10 **1607 or newer**, or Windows 11,
with x64 or ARM64 Windows and HTTPS access to GitHub, mise and uv's downloads.
TLS 1.2 is enabled for the bootstrap process so older PowerShell defaults do
not prevent downloads; certificate validation remains enabled. Older Windows
images may still need Windows updates, root certificates or runtime
prerequisites that a user-level script cannot install. Microsoft's current
Windows 10 support is edition/lifecycle dependent; this bootstrap does not
extend OS support. The Dokime candidates in this walkthrough remain x86-64.

The setup follows [Microsoft's ZIP installation guidance](https://learn.microsoft.com/powershell/scripting/install/install-powershell-on-windows#install-the-zip-package),
[mise's Windows installer guidance](https://mise.jdx.dev/installing-mise.html#windows)
and [mise's shell activation guidance](https://mise.jdx.dev/shell-setup.html).
See also [Microsoft's PowerShell execution policies](https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_execution_policies)
and [supported Windows releases](https://learn.microsoft.com/dotnet/core/install/windows#supported-versions).

## Configure A Script

Install Dokime on `PATH`, restart the host after admitting the matching OBC
plugin, and copy the host's public certificate and downloaded access key to the
remote computer. Keep the server's private key on the host.

Edit the globals at the top of each script before running it. Names, grouping,
order and meanings match the [Python examples](../python/README.md):

| Group | Parameters |
| --- | --- |
| Connection Parameters | `ENDPOINT`, `CA_CERTIFICATE`, `KEY_FILE`, `ENV_FILE` |
| Case And Execution Parameters | `TEMPLATE`, `CYCLE_LABEL`, then `SAMPLE_COUNT`, `EXECUTION_COUNT`, `WAIT_SECONDS` for tests; `ANSWER`, `WAIT_SECONDS` for prompts |
| Observation Filters | `RECORD_TYPES`, `STREAMS` for tests |
| Evidence Files | `OUTPUT_DIRECTORY`, `OVERWRITE` |
| Response Files | `RESPONSE_DIRECTORY` for device control |

Set `ENDPOINT` to the host's actual gRPC endpoint, such as
`https://lab-host.example:8766`. Its hostname must match the public certificate.
Use an empty certificate path only for an explicitly configured plaintext
isolated lab endpoint. An empty key or dotenv path omits that file.

An explicit `KEY_FILE` takes precedence over process `DOKIME_KEY`, which takes
precedence over an explicitly selected `ENV_FILE`. Copy
[`../.env.example`](../.env.example) when using dotenv, replace its placeholder,
and clear `KEY_FILE`. No parent-directory search occurs. Keep credentials private.

POSIX globals use `NAME="value"`; PowerShell uses `$NAME = 'value'`; batch uses
`set "NAME=value"`. POSIX filter lists are space-separated strings, PowerShell
filter lists are arrays, and batch lists contain individually quoted values.
Quote globs such as `obc.run.*`. Each matching filter is an alternative; empty
filter lists observe all records or streams.
`EXECUTION_COUNT` repeats every materialized case that many times;
`SAMPLE_COUNT` controls samples per pulse. `WAIT_SECONDS` is the positive
deadline for each blocking wait. `ANSWER` is `yes` or `no`.

The scripts call literal `dokime` from `PATH`. Run them without arguments:

```sh
sh ./control-device.sh
sh ./control-test.sh
sh ./control-prompt.sh
```

PowerShell 7.3 or newer preserves inline JSON through Standard native argument
passing:

```powershell
.\control-device.ps1
.\control-test.ps1
.\control-prompt.ps1
```

Command Prompt needs only Dokime and cmd.exe:

```bat
control-device.bat
control-test.bat
control-prompt.bat
```

## Control The Device

The device script lists operations, reads the manual device, powers it on,
reads it again, powers it off and performs a final read. Powered state reports
230 V RMS; off state reports zero. Manual control is independent of execution
devices and is not execution evidence.

Optionally set `RESPONSE_DIRECTORY` to a new directory. The script reserves it
before connecting and refuses an existing directory before device work.
It saves `before.json`, `power-on.json`, `running.json`, `power-off.json`
and `after.json`. Each native response is an ordered event array ending in
completion. Empty `RESPONSE_DIRECTORY` prints responses without files.
Ctrl+C stops waiting; read state before resending an unconfirmed command.

## Run Independent Attempts

The test script materializes `TEMPLATE` with `sample_count`, checks all output
destinations, creates a cycle and appends every attempt without starting work.
It prints exact cycle and execution UUIDs captured with `--id-only`.
These cycles use `continue_automatically=false`. Explicit execution Stop
durably sets that policy to false; a later explicit Start runs only the selected
attempt until the user explicitly re-enables the policy.

Each execution then runs sequentially through the blocking `execution run`
command. Record and stream filters select live observations; the command returns
after the durable outcome and final listener drain. The script saves
`execution-1.zip`, `execution-2.zip` and further numbered bundles as configured.
It seals its cycle explicitly and saves `cycle.zip`. A nonpassing durable outcome
still saves evidence and returns nonzero. A failed connection stops the workflow
without retrying mutations.

## Answer One Confirmation

The prompt script materializes one case with `require_confirmation=true`,
appends its attempt and explicitly calls `execution start`. It discovers,
displays and answers the pending confirmation using both the execution and
prompt UUIDs. Sampling begins after confirmation. The script waits for the
durable result, saves `execution.zip`, seals its cycle and saves `cycle.zip`.

## Preserve And Inspect Evidence

Choose a separate `OUTPUT_DIRECTORY` for each workflow. Existing bundle
destinations fail preflight unless `OVERWRITE` is enabled. CLI downloads verify
checksums and replace files atomically; download retries never rerun hardware.
Batch scripts also retain confirmed identities in `.id` files.

Bundles contain `manifest.json`, `evidence.sqlite`, captured `files/` and
saved `reports/`. Inspect the SQLite database and verify manifest checksums.
An execution bundle saved before the cycle is sealed records an Active cycle;
the final cycle bundle includes all members and its sealed phase.

## Interruption And Recovery

Append, Start, blocking run, execution wait/export and prompt discovery,
display/answer pass `--cancel-on-interrupt`, `--owned-cycle` and
`--recovery-directory` to the native CLI on every platform.
Ctrl+C stops the selected execution, seals that script-owned cycle, waits for
durable evidence and saves `execution-<UUID>.zip` and `cycle-<UUID>.zip` under
`recovery/`. Sealing cancels that cycle's unstarted members. Other cycles and
the shared server remain available. Batch delegates recovery to the native CLI
because cmd.exe cannot reliably own Ctrl+C cleanup.

PowerShell and POSIX shell also attempt scoped cleanup when interrupted between
commands after cycle creation is confirmed. Batch cannot reliably handle Ctrl+C
between native commands or before their signal listener is installed, including
connection setup. If cycle creation's reply is lost, no script can infer its
unknown cycle UUID. These cases, another interruption during recovery, and lost
connections can leave the result unconfirmed. Retain printed identities,
reconnect to the same server and inspect before resubmitting. Do not retry an
uncertain mutation automatically. Existing evidence is preserved.

## Institute Plugins

Keep the same lifecycle for institute plugins, replacing operation IDs,
template and parameters with their discovered contracts. The OBC voltage and
fault behavior apply only to this demonstration.
