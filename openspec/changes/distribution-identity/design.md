# Design

Locked 2026-10-29; implementation scheduled.

## Identity Source

```toml
# installer.toml (distributor-owned)
[installer]
version = "0.1.0"
organization = "organization"
identifier = "com.example.testhub"     # storage + install identity for this distribution
```

- Validated like a reverse-DNS application identifier; the slug used in file and package names derives from it.
- Fixed per installation. Changing it later is a new installation: data under the old identifier stays where it is and is never moved or deleted silently (development storage policy: no migrations).

## Installed Distribution File

`cargo dokime installer package` writes `distribution.json` beside the installed executables:

```json
{ "schema": 1, "identifier": "com.example.testhub" }
```

It is hashed in `installer.json`, the verified installed-file inventory used by bundled-plugin startup (the previous full `distribution.json` role is removed). Absent identity file: the compiled identifier applies; invalid files fail startup.

Linux installs binaries and resources together under `/usr/lib/<slug>/`, with identifier-derived `/usr/bin/<slug>` and `<slug>-gui` launchers; dots become hyphens in explicit identifiers. With no override, the payload slug stays `dokime`. NSIS uses the identifier for its install folder and registry keys. Its product name, installer pages, shortcuts and Add/Remove Programs display name use the resolved organization manifest's `[app] label`. Linux package descriptions and desktop entries use that label; package names keep the slug. NSIS output filenames are `<slug>-<version>-<Rust-target>-setup.exe`, preserving the release layout.

## Resolution At Startup

The desktop reads the distribution file before the organization pack, logging, storage or any window, and derives every directory from it:

| Directory | Windows | Unix |
|---|---|---|
| Application | `%LOCALAPPDATA%\<identifier>` | XDG data home `/<identifier>` |
| Data | `…\<identifier>\data` | `…/<identifier>/data` |
| Plugins | `…\<identifier>\plugins` | `…/<identifier>/plugins` |
| Logs | `…\<identifier>\logs` | XDG state home `/<identifier>/logs` |
| WebView profile | `…\<identifier>\EBWebView` (set as the webview data directory) | engine default under the application directory |

`DOKIME_APPLICATION_ROOT` keeps overriding the application root for tests. CLI and server defaults use the same resolver. Launch/runtime configuration and settings retain its `ApplicationPaths` snapshot (root and log directory), including diagnostics and log streams; they never reconstruct defaults from the root.

## Install Identity

The NSIS install folder and uninstall registry key, and the DEB/RPM package names, use the distribution identifier, so a branded distribution installs beside plain Dokime.

## Still Compiled (Out Of Scope)

Deep-link scheme, updater trust and code-signing identity remain per build. Two branded distributions of one build still share those compiled mechanisms; making them per-distribution is a separate decision. Every desktop launch opens its own window without a desktop lease or focus forwarding; only the backend exclusively leases its resolved data folder.
