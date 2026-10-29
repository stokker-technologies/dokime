# Design

Locked 2026-10-29; implementation deferred.

## Identity Source

```toml
# installer.toml (distributor-owned)
[installer]
version = "0.1.0"
organization = "organization"
identifier = "com.hilti.dokime"     # storage + install identity for this distribution
```

- Validated like a reverse-DNS application identifier; the slug used in file and package names derives from it.
- Fixed per installation. Changing it later is a new installation: data under the old identifier stays where it is and is never moved or deleted silently (development storage policy: no migrations).

## Installed Distribution File

`cargo dokime installer package` writes `distribution.json` beside the installed executables:

```json
{ "schema": 1, "identifier": "com.hilti.dokime" }
```

It is part of the verified payload inventory. Absent file: the compiled identifier applies.

## Resolution At Startup

The desktop reads the distribution file before the organization pack, logging, storage or any window, and derives every directory from it:

| Directory | Windows | Unix |
|---|---|---|
| Application | `%LOCALAPPDATA%\<identifier>` | XDG data home `/<identifier>` |
| Data | `…\<identifier>\data` | `…/<identifier>/data` |
| Plugins | `…\<identifier>\plugins` | `…/<identifier>/plugins` |
| Logs | `…\<identifier>\logs` | XDG state home `/<identifier>/logs` |
| WebView profile | `…\<identifier>\EBWebView` (set as the webview data directory) | engine default under the application directory |

`DOKIME_APPLICATION_ROOT` keeps overriding the application root for tests. CLI and server defaults use the same resolver.

## Install Identity

The NSIS install folder and uninstall registry key, and the DEB/RPM package names, use the distribution identifier, so a branded distribution installs beside plain Dokime.

## Still Compiled (Out Of Scope)

Deep-link scheme, single-instance key, updater trust and code-signing identity remain per build. Two branded distributions of one build still share them; making those per-distribution is a separate decision.
