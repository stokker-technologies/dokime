# Release Layout

## Why

Release v0.1.5 publishes 66 flat assets. 46 of them are software bills of materials, one per source
crate and platform and one per shipped artifact. A release page has no folders, so a person looking
for an installer has to find it among them. Nightly publishes the same flat set, and its benchmark
and coverage results are not published at all: they exist only as workflow run artifacts that expire
after 7 or 13 days, and the Nightly release notes carry only a pass or fail line per suite.

## What Changes

One layout for stable releases and for Nightly. Nightly is a release run without the tag and the
registry publication, plus benchmarks and coverage.

```text
dokime-<version>-x86_64-pc-windows-msvc-setup.exe
dokime_<version>_amd64.deb
dokime-<version>-1.x86_64.rpm
dokime-cli-<version>-<platform>.zip|tar.gz            one per platform
dokime-distribution-<version>-<platform>.zip          one per platform
dokime-plugins-<version>-<platform>.zip|tar.gz        one per platform
dokime-proto-<version>.zip                            dokime.v1 and harmos.v1 definitions
dokime-compliance-<version>.zip                       every SBOM, NOTICE, THIRD-PARTY-NOTICES
release.json                                          one index: latest, downloads, plugins
SHA256SUMS
dokime-reports-<version>.zip                          Nightly only: benchmark and coverage results
benchmarks.json                                       Nightly only: latest benchmark results, fixed name
```

- `dokime-compliance-<version>.zip` holds the bills of materials in folders: `artifacts/` (one per
  shipped file) and `crates/<platform>/` (one per source crate), with `NOTICE` and
  `THIRD-PARTY-NOTICES` at its root.
- The changelog is the release body, not an asset.
- `release.json` replaces `latest.json`, `downloads.json` and `plugins.json`. The updater, the
  download page and the plugin feed read their section of it. This changes what those consumers
  read; they move in the same change.
- `dokime-plugins` replaces `dokime-sidecars`.
- `cargo-dokime` archives stop when the single command-line tool of `grpc-plugins` lands.
- The pure Python SDK of `grpc-plugins` ships one `dokime-<version>-py3-none-any.whl`
  and one `dokime-<version>.tar.gz` per release, shared by all platforms. Both are
  release downloads and are uploaded together once to the registry.
- Nightly attaches `dokime-reports-<version>.zip`: benchmark results per suite with the baseline
  they were compared to, and coverage per language as a summary and the raw report. The Nightly
  release notes gain a short table: coverage per language, benchmarks that moved beyond their
  threshold, and suite results, with the run link. Stable releases do not run or attach them.

## Nightly and Release Share Jobs

Nightly is the release run with `channel=nightly`: no version tag, no registry publication, and
benchmarks and coverage added. The validation jobs of CI and the publish and verify jobs of
Release are reusable workflows that Nightly calls; it adds validation planning, publication
freshness, benchmarks, coverage and the report. Nightly validation always plans from the current
tree, independently of previous releases, tags and repository history. The owner decision of
2026-11-19 confines freshness (tag comparison, duplicate revision detection, lookup errors fail
closed) to publication; it cannot gate checks, builds or tests. Dokime and Harmos retain their
triggers and timezone, non-cancelling run, stage and job names, named artifact interface, publish entry point, digest
verification, and an idempotent result report guarded by the publication identity. Products,
languages, matrices and asset inventories stay each project's own.

### Earlier Release and History Inventory

The owner removed dependencies of Nightly tests on earlier releases or history. Fabricated
fixtures remain deterministic tests; release publication and verification retain their own
provider operations. No test that fetches a real previous product release was found in
`tools/python`; none of the fixture-based release tests is removed.

| What and Location | Protection and Lanes | Decision |
| --- | --- | --- |
| `.github/workflows/nightly.yml` freshness dependency edges | Avoid unchanged/duplicate publication; previously gated every Nightly validation job, absent from PR | Remove edges from validation; current-tree plan supplies matrices; freshness gates only Publish and its Report |
| `tools/python/src/dokime_tools/ci/nightly_freshness.py`; `test_ci_workflow.py::test_nightly_freshness_compares_tags_and_only_earlier_duplicates` | Nightly tag lookup, duplicate run suppression; fixture test in PR/Nightly tooling | Keep publication policy and mocked test; provider failure no longer prevents validation |
| `ci/impact.py::changed_paths`; `test_ci_impact.py` | PR/push changed-path selection through merge-base/diff, fail-open on missing history | Keep; Nightly plans all eligible jobs without calling history comparison |
| Workflow `fetch-depth: 0` checkouts | Makes history available to release notes and PR planning; PR/Nightly/release | Keep; checkout alone does not compare prior release assets or make tests require them |
| `release/nightly.py`; `test_ci_gates_channel.py`; `ci_gates_fixtures.py` | Channel draft/swap/rollback, stale publication refusal, prior notes/history; Nightly publication, fixture tests in PR/Nightly | Keep; release flow, deterministic in-memory provider tests |
| `release/verify.py`, `release/registry/provider.py`, `package/installation.py`; `verify.yml` | Downloads and installs the exact just-published release, checks digests, signatures and revision; release/Nightly post-publication, no PR test call | Keep; verifies this run's publication rather than an earlier release |
| `test_ci_gates_channel.py::test_shared_verifier_preserves_channel_and_signing_policy` and installer test | Channel/signature/installer policy; PR/Nightly tooling | Keep; `v0.1.5`, remote tags, downloads and subprocesses are fabricated/mocked |
| `release/layout.py`; `test_release_downloads_download_manifest.py`; `release_downloads_fixtures.py` | Classifies assets and constructs tagged URLs; PR/Nightly tooling | Keep; fabricated inventories and checksums, no network |
| `release/dry.py`, `release/staging.py`, `package/verification.py`; `test_release_downloads_stage.py`, `test_release_downloads_python_packaging.py` | Rehearses current staged packages and manifests; PR/Nightly Release / Dry | Keep; reads current packages, not a prior publication |
| `package/changelog.py`, `package/assembly.py`, `release/staging.py`; release-note fixtures | Renders current release notes from git-cliff for Package and Release / Dry in PR/Nightly, plus release publication | Keep release artifact production; deterministic notes tests use fabricated commands, no prior asset validation |
| `ci/review.py`, `release/review.py`, `release/guard.py`, `release/tag.py`, `release/registry`, release/publish workflows | Release branch/tag identity, required checks, registry and publication safety | Keep; release-time operations, mocked gate/provider tests in PR/Nightly |
| `docs/image.py::download`; `test_docs_site.py` | Assembles released documentation versions; documentation publication only, fixture test in PR/Nightly | Keep; image publication is not a test of prior release contents |
| `ci/native.py` prerequisite URLs | Installs pinned third-party build tools in PR/Nightly | Keep; tool provisioning, not previous product release verification |
| `test/benchmark.py`, `test/desktop_bench.py` local baseline reads; benchmark fixture tests | Compare measurements with optional local `dev` output; Nightly Bench and fabricated PR/Nightly tooling tests | Keep; no release download or repository history; Nightly seeds missing baselines |
| `test/archive.py`, `test/benchmark.py`, `docs/site.py` revision reads | Labels current artifacts/results with HEAD | Keep; no previous history needed |
| `build/cli_fixtures.py::build` | Generated starter proof; Rust candidate override, Python current wheel, C++ current local archive plus SHA-256 | Keep; explicit C++ coordinates already avoid a prior release |
| `crates/dokime-cli/src/commands/plugin/mod.rs::created_starters_resolve_native_metadata_and_asset_inputs`, `tests/system/scaffold.rs` | Starter shape proof including C++; PR/Nightly | Keep; C++ scaffolding passes explicit fabricated archive coordinates rather than resolving a release |

The C++ default in `crates/dokime-cli/src/commands/new.rs` resolves a checksum from the package
version's published asset. Version 0.1.5 lacks that asset. Its owning branch must make candidate
proof pass the archive built from this checkout and its computed SHA-256 (or a local fixture
server with fabricated bytes); tests must never resolve the implicit published-version URL.
`build/cli_fixtures.py` already uses explicit local coordinates. The production default should
resolve only an explicitly selected published SDK version, and otherwise ask for `--sdk-url`
and `--sdk-sha256` when that release does not exist. This worker does not change `new.rs`.

## Published Numbers

`benchmarks.json` is attached to every Nightly under that fixed name, so a site can read the
latest results from the Nightly release without expiry. It carries, per benchmark: name, suite,
unit, value, the baseline value and the revision. No history is kept in the repository.

## Screenshots

The committed product screenshots move to `docs/guides/screenshots/` (change `screenshots-task`). They
are not release assets.

## What Stays

Signing, checksums, the embedded installer signature, the asset names of installers and archives
other than the ones listed, and the rule that Nightly publication depends only on build and package
while tests, benchmarks and coverage may fail.

## Impact

Release and Nightly tooling and workflows, the updater, the download index consumer, the plugin
feed consumer, the release rehearsal, the product guide and README download instructions. Harmos
mirrors the layout in its own change of the same name.
