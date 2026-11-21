# Installer Packaging

## Owner Decisions

The release and organization installer have one producer: the CLI built with the
kit runs `dokime installer package` with a checked-in default manifest and
`--artifacts`. Packaging never downloads in this path. Delete the Python native
producer, its configuration and tests only after updater and native payload parity
is established. Retain the CI graph and job names; change packaging steps only.
The installer is never executed during this delivery.

Architecture invariant 14 states that "shells never redeclare contracts"
(`docs/project/architecture.md:363`). The packaging library owns assembly and its narrow
step events; the CLI owns rendering and does not introduce a second event model
or callback into the whole runtime.

The default `installer.toml` omits the installer version, which resolves to the
built CLI's package version. `signing.updater.artifact_base_url_env` is mutually
exclusive with `artifact_base_url`; it resolves an HTTPS base URL from the named
environment variable. Release dispatch sets it from the kit's recorded channel
and version, preserving an explicit publishing override. Only signed Windows
dispatch passes `--sign`. Exclusions apply to defaults; an explicit named entry
can add the same identity again, once.

The release key environment keeps Tauri's path-or-content semantics: a path,
raw minisign key, or its base64 encoding resolves through the library's one
signing implementation. An omitted release key password is empty, as in Tauri.
Signatures bind the installer version in their authenticated `version:` trusted
comment and are verified against the compiled public key before publication.
This matches the [Tauri updater contract](https://v2.tauri.app/plugin/updater/)
and the pinned updater 2.14.1's signed-version parser; verification is never disabled.

## Runtime Plugin Inventory

Plugin bundles exclude the following development and private inputs at every
directory depth, matching ASCII names without case sensitivity. Required launch
inputs matching these rules are refused with an action to move them; Git-ignored
required inputs retain the Unignore refusal. They are never silently dropped.
Build and launch scripts and their runtime metadata remain ordinary inputs unless
excluded.

```text
mise.toml .mise.toml mise.lock .tool-versions
.env .env.*
.git .gitignore .gitattributes .gitmodules .github .gitlab .gitlab-ci.yml
.hg .hgignore .hgtags .svn
test tests __tests__ .pytest_cache .mypy_cache .ruff_cache .cache
.idea .vscode .vs .DS_Store Thumbs.db
.venv __pycache__ _input _output credentials credentials.json
.netrc .pypirc id_rsa id_ed25519 *.pair *.pem *.key
```

Native bundles already select executable, assets and referenced metadata only.
Source bundles keep build and launch code, language manifests and lock files.
The Rust starter inventory has only the executable, icon and bundle metadata.
Python still includes the separate client entrypoint
`src/starter_project/__main__.py`; C# includes `src/Example.Client/`,
`Example.slnx` and `Directory.Build.props` alongside its launch payload and source.
Removing these requires an owner decision on source-project build inputs. C++
also includes its selected executable through `assets/plugin/`; that duplicate
is retained for the overlap decision below. The deterministic starter audit copies
project inputs and fabricates native build outputs; it does not build or execute
the four SDK starters.

## Kit Identity and Default Plugins

`distribution.json` remains schema 1 and requires `source_revision`, `channel`
and `plugins` in addition to version, target and files. Build/Package writes the
same `DOKIME_SOURCE_REVISION` and `DOKIME_BUILD_CHANNEL` consumed by executable
build scripts; no second revision/channel inference or old-shape reader exists.
The exporter reads `dokime version --json` from that build's CLI to obtain those
embedded values, rather than asking Git or interpreting workflow names again.
The kit lists default plugin identities from the first-party built bundles. Local
kits store those bundles in `plugins/`; downloaded kits select them from the
verified release plugins archive. Every installer includes defaults unless
explicitly excluded. Additional `[[plugin]]` name/path entries remain supported.

```toml
[installer]
version = "1.2.3"
formats = ["nsis"]
exclude_plugins = ["obc"]

[dokime]
version = "nightly"

[[plugin]]
path = "plugins/bench.dokime"
```

There is one ZIP regular-member writer: `package/assembly.py::write_file`.
Distribution and release plugin archives delegate permissions and file-type bits
to it, retaining their existing integrity and path checks.

Default names are inspected through the built CLI's `plugin inspect --json`,
not inferred from archive filenames. The first-party `plugin-obc.dokime` declares
`obc`, so `obc` is its manifest exclusion name and progress identity. The archive
filename remains unchanged; no alias or second plugin registry is introduced.

## Release Parity

Before deleting the Python producer, reconcile installer names, product label and
identifier, current-user Windows hooks, installed resource locations, Linux
dependencies and desktop template, Authenticode, updater signatures and metadata,
and release verification expectations. The single updater filename is
`release.json`, consumed by desktop updater endpoints. Release staging now reads
that same name instead of the former intermediate `latest.json`.
Release filenames stay `dokime-<version>-<target>-setup.exe`, `.deb` and `.rpm`.
Organization filenames use the distribution slug. Keep signature verification,
archive bounds and operation deadlines unchanged. Stable Windows publication
requires updater signing. No fallback producer or compatibility alias remains.

### Parity Audit

The Python references below name the removed producer at the starting revision
`92f0db8f2`; the Rust references name its replacement responsibilities. Python
paths are relative to `tools/python/src/dokime_tools/`, installer Rust/NSIS paths
to `crates/dokime-packaging/src/`, and `src-tauri/` to `crates/dokime-gui/`.

| Concern | Previous Producer Evidence | Unified Producer and Outcome |
| --- | --- | --- |
| Output Names | `package/installer.py:147` | `installer/native.rs:566`: release prefix `dokime`, version and target for every format; closed. |
| Label and Identifier | `package/installer.py:64`, `src-tauri/tauri.conf.json:44` | `installer/native.rs:487` and manifest identity: default pack label and compiled identifier; organization overrides retained; closed. |
| Windows Scope and Hooks | `package/installer.py:89`, `src-tauri/installer-hooks.nsh:5` | `installer/installer.nsi:59` and `installer/hooks.nsh:2`: current user, no elevation, machine-install refusal in both registry views; CLI PATH hook shared; closed. |
| Resource Layout | `package/installer.py:80` | `installer/native.rs:110`: Windows executable siblings, organization and plugins folders; Linux real executables/resources under `/usr/lib/<slug>/` with `/usr/bin/` launchers; closed by the existing distribution-identity contract. |
| Linux Packages | `package/installer.py:119`, `src-tauri/tauri.conf.json:27` | `installer/native.rs:269`, `installer/native.rs:279` and `installer/native.rs:508`: explicit WebKit/GTK/AppIndicator/audio/Soup/JS/SVG/OpenSSL dependencies and one desktop template; closed. |
| Authenticode | `package/installer.py:95` | `installer/signing.rs:50`: same PowerShell provider for payloads and installer, signing verification retained; closed. |
| Updater Signature | `package/installer.py:153` | `installer/signing.rs:63`: base64 minisign sidecar, verification against compiled trust before publication; closed. |
| Updater Metadata | `package/installer.py:164`, `release/staging.py:145` | `installer/signing.rs:150`, `release/staging.py:145`: `release.json` everywhere, version and `platforms.windows-x86_64.{signature,url}` retained; staging adds release sections under the same root; closed. |
| Release Verification | `package/verification.py:91`, `release/verify.py:45` | Native archives execute declared CLI verification, published assets match SHA256SUMS and configured signature checks; no installer execution added locally; closed. |

Default Windows packaging preserves the built icon and executable payload before signing; a supplied
organization pack patches its icon before signing. Installation folders and
registry keys follow distribution identity. Older development installers using
the product label as their folder/registry key are not migrated. The obsolete
desktop hook file has no remaining producer consumer, but belongs to the desktop
owner and must be retired there. Linux package names use `dokime` for the default
distribution and the identifier slug for organizations; file names use version
and target uniformly. Hosted native packaging and signed publication remain
required proof; source parity is not a claim that those jobs ran locally. The
updater sidecar remains a package output; release staging publishes its signature
inside `release.json`, rather than declaring the sidecar as a separate asset.

## Progress Observer

One small caller-supplied observer receives completed steps and their durations:
Manifest, Kit, Appearance, Plugins, Bundle, Sign and Output. The packaging library
prints nothing. Failures identify the active step after prior completed events.
The CLI writes progress to stderr, paths to stdout, and keeps JSON unchanged and
silent on stderr. Nonterminal output has no color or cursor control. Decimal
numbers use dots without grouping. Kit reports version, short revision, channel,
target, acquisition and verified inventory. Plugin entries distinguish defaults
from additions. Output reports paths and sizes; the final line reports installer
count and total duration. The header uses the CLI's shared build identity when
available, with one call site. Signing notices use the same observer.
The Python package dispatcher forwards all three standard streams explicitly so
the built CLI's output survives headless Windows invocation.

## Proof and Ownership

Do not edit `docs/`, CLI version ownership or desktop sources. Report documentation
impact by file and line. Validate this change strictly; keep its task checkboxes
current. Run format, scoped test-target compilation, focused packaging tests,
installer acceptance, affected Python tooling tests and repository checks once at
the end. Shared test contracts additionally require workspace test-target
compilation with desktop `testing`. Never run an installer. End-to-end Windows
packaging uses a local kit only if project Build/Package can produce it; compare
produced names and sizes with published Nightly assets. Hosted Nightly must prove
native Windows/Linux packaging and release verification. Commit locally and never
push from this worktree.

Packaging has no external 7-Zip requirement: the shared Python writer owns ZIP
outputs and the native bundler owns installer assembly. Remove the unused package
task tool and packaging-job allowlist entries. One-off archive inspection does
not add a project dependency.


## Retained Overlaps for Owner Decision

No duplicate below is removed without the owner's decision. The explicitly
requested Python producer and ZIP member writers are removed in this delivery.

| Locations | Responsibilities and Callers | Recommended Deletion |
| --- | --- | --- |
| `crates/dokime-packaging/src/installer/{pack,policy}.rs` and `crates/dokime-gui/src-tauri/src/app/{organization/pack,updates/policy}.rs` | Both validate appearance and update policy; installer preparation calls the former, desktop pack loading and updates call the latter. | Delete the packaging copies after choosing a lightweight shared contract owner. |
| `crates/dokime-packaging/src/installer/distribution.rs::Identity` and `crates/dokime-gui/src-tauri/src/paths.rs::DistributionIdentity` | Both validate reverse-DNS distribution identity; CLI packaging and desktop installation resolution call them respectively. | Delete the desktop validator after exposing the shared identity contract without bundler dependencies. |
| `crates/dokime-packaging/src/installer/{native.rs,hooks.nsh,installer.nsi}` and `crates/dokime-gui/src-tauri/{tauri.conf.json,installer-hooks.nsh}` | Both declare NSIS policy/hooks and Linux package dependencies. The active CLI producer uses the native settings and hook; desktop development/context generation still reads the GUI configuration, but its old installer producer is deleted and the old hook has no remaining producer. | Delete the GUI hook and unused installer-only bundle settings in the desktop owner's delivery, preserving fields still needed by desktop context/icon generation. |
| `crates/dokime-packaging/src/bundle/mod.rs::{create_project,create}` | Both construct native plugin inventories and freeze them through the library writer; CLI bundling calls the former, runtime `host.rs` and test fixtures call the latter. | Delete the raw-executable helper after migrating the runtime convenience API and fixtures to the project/bundle boundary. |
| `crates/dokime-packaging/src/bundle/mod.rs::create_project` entrypoint insertion and assets traversal; `sdk/cpp/examples/starter/dokime.toml` | C++ selects an executable below `assets/plugin/`, so that same file also enters the assets inventory; CLI project bundling calls both blocks. | Delete the selected executable's assets copy, retaining its `bin/` entrypoint. |
| `tools/python/src/dokime_tools/ci/signing.py:70` and `crates/dokime-packaging/src/installer/native.rs::prepare` | Release CI signs/verifies exported executables; native preparation signs their staged copies with the same provider. | Delete repeat signing of already verified, unchanged payloads in the native path while preserving verification and re-signing patched artwork. |
| `tools/python/src/dokime_tools/release/layout.py::plugin_feed` and `crates/dokime-packaging/src/installer/plugins.rs::select` | Release staging publishes package-based plugin names, while installer selection uses bundle identities; `plugin-obc` versus `obc` exposes the mismatch. The new kit declaration also uses inspected bundle identities. | Delete the package-name projection after deciding whether the release catalog still has a consumer; any retained catalog should use the existing bundle inspector's identities. |
| `tools/python/src/dokime_tools/ci/tasks.py::main` focused-package command construction and `tools/python/src/dokime_tools/test/focused.py::{selections,changed}` | Both construct Rust test commands; `mise run test -- --package` calls the former and `--changed` calls the latter. The changed-path branch returns before filter validation and does not forward `--filterset`, so the combination silently drops its filter. | Delete the inline package command builder after routing both scopes through the existing selection owner, preserving explicit filters or rejecting unsupported combinations. This tooling is outside the installer worker's ownership. |
| `crates/dokime-packaging/src/installer/distribution.rs::digest`, `crates/dokime-packaging/src/bundle/mod.rs::tests::digest` and Harmos `crates/harmos-runtime/src/bundle/mod.rs::digest` | All compute lowercase hexadecimal SHA-256 for bytes. Installer verification/payload indexing and bundle fixture construction call the local copies; library bundle integrity and the product's installation fixtures use the library facility. | Delete both local digest functions and use the existing public library function; review the remaining direct hash dependency before removing it. |
