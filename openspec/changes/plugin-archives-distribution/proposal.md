# Plugin Archives And Distribution

## Why

The owner locked release archives and feed format 2 on 2026-10-29. Dokime still
downloads individual plugins and requires distributor-supplied release URLs.

## What Changes

- Consume only plugins.json format 2 with archive and member integrity checks.
- Resolve installer versions and named plugins from release assets, caching by digest.
- Rename installer payload exports to distribution and package sidecar archives.
- Retain local build-tree assembly and checked external bundles.
- Select plugins only through top-level [[plugin]] name/path entries or equivalent
  CLI inputs; remove implicit organization plugin discovery and overlays.
- Update product guides and declarations; leave Hub-rendered release code to Hub.

## Capabilities

### New Capabilities

- `plugin-distribution`: verified archive-based plugin and distribution consumption.

### Modified Capabilities

Installer plugin selection replaces implicit folder inclusion; appearance merging
and plugin activation behavior are retained.

## Impact

Clean break for feed v1, release_url/sha256 and prior [dokime] plugin selection keys.
No storage migration,
crate renaming, release publication or push. All data format baselines remain v1.
