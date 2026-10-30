# Plugin Archives And Distribution

## Locked Direction

Implement the owner relock of 2026-10-29. This change owns only Dokime product
source, declarations, tests and documentation in feat/plugin-archives-distribution.
Other parked changes and sibling worktrees are outside scope.

## Contracts

```toml
[installer]
version = "1.0.0"
formats = ["nsis"]
[dokime]
version = "0.1.3" # "dev-channel", or omitted = cargo-dokime version
target = "x86_64-pc-windows-msvc"
[[plugin]]
name = "plugin-host"
[[plugin]]
path = "plugins/acme-bench.dokime"
```

The owner's amendment supersedes the earlier plugin arrays/external_plugins tables
and organization folder discovery. Each top-level plugin entry sets exactly one
of name/path; both or neither fail with a one-based entry index before acquisition.
Paths stay inside installer.organization (resolved against the installer manifest),
or the distribution's organization directory when no override is selected. Missing
override plugins never fall back to base files. The plugins/ folder is storage only;
unlisted files never ship and plugin selection performs no filename overlay.
CLI --plugin paths resolve against the invocation directory. Names are opaque
feed/declaration names; no crate rename or plugin-prefix inference occurs.

Feed v2 contains plugins with name, kind, version, requires_dokime and targets
mapping target triples to archive/member/sha256. Sidecars use the native target;
future guest archives are target-independent (a wasm target is selected explicitly).
Guests use targets.any for target-independent selection and still obey existing
runtime admission limits; no guest execution is added.
SHA256SUMS is authoritative for archive hashes and covers plugins.json too.
Archives are verified before opening; only a requested, unique regular member is
read, then checked against its feed hash and inspected identity. No per-plugin URL
or v1 fallback remains. Native archives are ZIP on Windows, tar.gz on Unix.

## Ownership And Integration

Domain plugin release contracts own feed validation; the entry library's native
surface provides shared archive integrity/extraction used by desktop and packaging.
No separate archive feature adds another runtime feature configuration. Desktop keeps
source/connection generation fences and explicit installation. Its discovery
returns all compatible feed entries so any number of shipped plugins is usable.

Installer release resolution uses GitHub's named release download directory:
v<SemVer>, or dev-channel. Distribution metadata supplies the actual workspace
version for dev-channel. Downloads and digest-keyed per-user cache are bounded and
revalidated before reuse. Tests inject local served release roots/cache folders
through internal constructors, never production endpoint overrides or network.

Rename the product export to product/installer-distribution and its metadata to
distribution.json (schema v1). Installed distribution.json remains the existing
identity file; installer staging writes that separately.

Hub owns generated tools/python/src/dokime_tools/release and scaffold copies.
Assume its refreshed layout accepts kind=distribution/sidecars/guests asset declarations,
emits format 2 for multiple plugins, uses unversioned member names from declared
bundles, hashes members, and publishes matching SHA256SUMS. Product packaging
produces distribution and native sidecars archives from existing build bytes.
No generated release files or templates are edited here. The product declares the
sidecars archives as release assets; Hub must consume those once without emitting
duplicate names while inspecting the declared bundle paths for feed generation.

## Triage

External-bundle target, compatibility and member hash checks remain in
crates/dokime-packaging/src/installer/plugins.rs:120; duplicate checks remain at :74.
Existing --artifacts assembly and repeatable --plugin flags are retained.
Feed v2 and distribution exports landed in this change; the amendment replaces
the superseded external_plugins tables and implicit plugin-folder selection.
tools/python/src/dokime_tools/package/installer.py:83 already selects shipping
bundles through --plugin and needs no manifest conversion. Archived/sibling
change records are historical; this amendment supersedes their implicit
plugin-folder inclusion contract.

## Proof And Handoff

Only targeted mise tasks run. No full lane, CI simulation, push or release.
Source closure does not assert Hub integration or final cross-platform proof.
Specification retirement requires unchecked integration-owner task 6.1 to close;
this product-only assignment cannot perform or attest that work.
