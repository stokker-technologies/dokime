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
- Python wheels stop being release assets when the pure Python SDK of `grpc-plugins` is published
  to the registry. Until then they remain, two files.
- Nightly attaches `dokime-reports-<version>.zip`: benchmark results per suite with the baseline
  they were compared to, and coverage per language as a summary and the raw report. The Nightly
  release notes gain a short table: coverage per language, benchmarks that moved beyond their
  threshold, and suite results, with the run link. Stable releases do not run or attach them.

## What Stays

Signing, checksums, the embedded installer signature, the asset names of installers and archives
other than the ones listed, and the rule that Nightly publication depends only on build and package
while tests, benchmarks and coverage may fail.

## Open Points

- Whether benchmark history across Nightly runs is kept anywhere beyond the last report and the
  baseline cache. Not in this change unless the owner asks.

## Impact

Release and Nightly tooling and workflows, the updater, the download index consumer, the plugin
feed consumer, the release rehearsal, the product guide and README download instructions. Harmos
mirrors the layout in its own change of the same name.
