# Remote CLI Workflows

Scripts: `packages/dokime-client/examples/remote/cli/`. Commands below run from that folder.

Control the simulated OBC device, run independent attempts with live observations,
and answer a confirmation using an installed Dokime CLI on another computer.
Start with the [shared candidate-artifact walkthrough](overview.md) to obtain
the matching CLI and host plugin, configure the host and transfer secrets.
A matching published release combination is not claimed; native execution and
interruption behavior still require platform validation.

## Bootstrap Windows Tools

The default setup installs the released Dokime CLI from its official GitHub release asset using mise.
Edit `DOKIME_VERSION` at the top of `bootstrap.ps1` to select the exact release.
Its official Windows binary must already be published;
an unavailable release fails setup without compiling from source or fetching
third-party builds.

**For this candidate walkthrough**, set `$INSTALL_DOKIME = $false` first and
install the matching candidate with the native installer described in the
[shared guide](overview.md#install-dokime). This also provides a tools-only
setup for Python users. Both launchers read that same editable option.

From the extracted `cli/` directory, run the batch launcher in a normal,
unelevated Command Prompt:

```bat
bootstrap.bat
```

Or start the same setup from built-in Windows PowerShell:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\bootstrap.ps1
```

Both use `bootstrap.ps1`, whose installation commands and PATH updates are
written in order without helper functions. It installs PowerShell **7.6.7** from
Microsoft's official self-contained ZIP, verifies its pinned SHA-256, installs mise using
its official Windows installer, then runs `mise use -g uv` and `mise reshim`.
With `INSTALL_DOKIME` enabled on Windows x64, mise installs the selected version's official `dokime-cli` release asset and exposes its `dokime.exe` command. This uses the GitHub release backend with an exact asset name; registry and source-build fallbacks are absent.
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
The CLI installs under `%LOCALAPPDATA%\Programs\DokimeCLI\bin\`, which is added
to user PATH after the installed `dokime.exe --version` succeeds. This installs
the remote client; `dokime server`, `dokime gui` and the host PC still require
the matching desktop/server installation.

Close and reopen your terminal and editor after setup, then verify:

```powershell
pwsh --version
mise doctor
uv --version
```

After the default CLI installation, also check:

```powershell
dokime --version
```

If PowerShell refuses to load your profile under the default Restricted policy,
Microsoft's usual personal-machine setting is
`Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`. Review your organization's
policy before applying it. The bootstrap leaves this persistent choice to you.
Use `mise exec -- <command>` when shell activation cannot be enabled.

Re-running setup reuses the installed PowerShell version, lets the official mise
installer retain or update mise, updates the global uv selection and ensures
the selected CLI version is installed when enabled. It does not automatically
trust project configurations. To develop from a checkout, review its
`mise.toml`, then run `mise trust` and `mise install`.

Use Windows PowerShell **5.1** on Windows 10 **1607 or newer**, or Windows 11,
with x64 or ARM64 Windows and HTTPS access to GitHub, mise and uv's
downloads. The CLI also needs a published native asset for that architecture.
TLS 1.2 is enabled for the bootstrap process so older PowerShell defaults do
not prevent downloads; certificate validation remains enabled. Older Windows
images may still need Windows updates, root certificates or runtime
prerequisites that a user-level script cannot install. Microsoft's current
Windows 10 support is edition/lifecycle dependent; this bootstrap does not
extend OS support. The Dokime candidates in this walkthrough remain x86-64.

The setup follows [Microsoft's ZIP installation guidance](https://learn.microsoft.com/powershell/scripting/install/install-powershell-on-windows#install-the-zip-package),
[mise's Windows installer guidance](https://mise.jdx.dev/installing-mise.html#windows)
and [mise's shell activation guidance](https://mise.jdx.dev/shell-setup.html).
CLI installation follows [mise's GitHub backend](https://mise.jdx.dev/dev-tools/backends/github.html).
See also [Microsoft's PowerShell execution policies](https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_execution_policies)
and [supported Windows releases](https://learn.microsoft.com/dotnet/core/install/windows#supported-versions).

## Configure A Script

Install Dokime on `PATH`, restart the host after admitting the matching OBC
plugin, and import the host-created pairing once:

```sh
dokime backend add --remote ../lab.dokime-remote --identifier lab
```

Edit the globals at the top of each script before running it. Names, grouping,
order and meanings match the [Python examples](python.md):

| Group | Parameters |
| --- | --- |
| Connection Parameters | `BACKEND_IDENTIFIER` |
| Case And Execution Parameters | `TEMPLATE`, `CYCLE_LABEL`, then `SAMPLE_COUNT`, `EXECUTION_COUNT`, `WAIT_SECONDS` for tests; `ANSWER`, `WAIT_SECONDS` for prompts |
| Observation Filters | `RECORD_TYPES`, `STREAMS` for tests |
| Evidence Files | `OUTPUT_DIRECTORY`, `OVERWRITE` |

Set `BACKEND_IDENTIFIER` to the imported profile identifier, normally `lab`.
Its label, ordered endpoints, expected identity and persistent CA come from the remote file; the
native client resolves its access key from the OS keyring.

POSIX globals use `NAME="value"`; PowerShell uses `$NAME = 'value'`; batch uses
`set "NAME=value"`. Each test call shows one record filter and
one stream filter. Quote globs such as `obc.run.*`; add explicit
`--record-type` or `--stream` arguments to teach additional selections.
`EXECUTION_COUNT` selects one or both of the two explicitly written attempts;
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

Each device call names its response file directly: `before.json`, `power-on.json`,
`running.json`, `power-off.json` and `after.json` in the current working directory.
The native `--output` option refuses an existing response file unless that call
explicitly adds `--overwrite`. Responses are ordered event arrays ending in completion.
Ctrl+C stops waiting; read device state before resending an unconfirmed command.

## Run Independent Attempts

Read each script from top to bottom: every `dokime` call shows the command
and arguments, with no workflow functions or loops over steps. The test script
checks output destinations, materializes one `TEMPLATE` case with `sample_count`,
creates a cycle and explicitly appends two independent attempts without starting
work. Use `EXECUTION_COUNT=1` to study only the first attempt; larger stories
need additional explicit calls and evidence destinations.
The create call saves `cycle.json` in the current directory; its `cycle_id`
becomes the cycle UUID. Append and prompt discovery use literal `execution.id`
and `prompt.id` files for native identity transport. They remain uncaptured so
PowerShell does not terminate their Ctrl+C recovery. Each call is one line,
and no script interprets native exit codes.
These cycles use `continue_automatically=false`. Explicit execution Stop
durably sets that policy to false; a later explicit Start runs only the selected
attempt until the user explicitly re-enables the policy.

Each execution then runs sequentially through the blocking `execution run`
command. Record and stream filters select live observations; the command returns
after the durable outcome and final listener drain. The script saves
`execution-1.zip` and `execution-2.zip` after their respective runs.
It seals its cycle explicitly and saves `cycle.zip`. POSIX `set -e` and
PowerShell native Stop mode can end the script on a nonzero command before later
exports; Command Prompt continues subsequent calls. Inspect the actual outcomes
and printed identities rather than treating script completion as a verdict gate.
No mutation is retried automatically.

## Answer One Confirmation

The prompt script materializes one case with `require_confirmation=true`,
appends its attempt and explicitly calls `execution start`. The blocking
`execution prompt next` discovers its prompt; the following calls display and
answer the pending confirmation using both the execution and
prompt UUIDs. The read-only prompt display is saved as `prompt.json` in the
current directory. Sampling begins after confirmation. The script waits for the
durable result, saves `execution.zip`, seals its cycle and saves `cycle.zip`.

## Preserve And Inspect Evidence

Choose a separate `OUTPUT_DIRECTORY` for each workflow. Existing bundle
destinations fail preflight unless `OVERWRITE` is enabled. CLI downloads verify
checksums and replace files atomically; download retries never rerun hardware.
The fixed `cycle.json` and `prompt.json` response files are replaced by ordinary
shell redirection. The `.id` files retain confirmed native identities.

Bundles contain `manifest.json`, `evidence.sqlite`, captured `artifacts/`,
Arrow IPC files of produced or captured samples under `telemetry/` and saved `reports/`.
Complete finalized producer files retain their original bytes; shared observer
files contribute only the execution's captured ranges at the saved frontier.
Cycle bundles
place artifacts and telemetry under `executions/<execution-id>/`. Join the
SQLite catalogs and verify manifest checksums.
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
connections or a nonzero command during cleanup can leave the result unconfirmed.
Retain printed identities,
reconnect to the same server and inspect before resubmitting. Do not retry an
uncertain mutation automatically. Existing evidence is preserved.

## Institute Plugins

Keep the same lifecycle for institute plugins, replacing operation IDs,
template and parameters with their discovered contracts. The OBC voltage and
fault behavior apply only to this demonstration.
