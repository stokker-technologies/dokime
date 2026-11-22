# Heat Strip Plot

## Why

Operators need to compare one reduced measurement per distinct category within a
selected window. A generic Heat Strip expresses this directly without requiring
an authored grid, interpolation or a domain-specific plot kind.

## What Changes

- Add `HEAT_STRIP`: equal labelled squares ordered by CELL, optionally one strip
  per FACET, coloured by a reduced VALUE with one shared scale and legend.
- Support LAST by default, MAX, MEAN and COUNT; absent values remain empty.
- Read all selected cells and facets in one bounded grouped query per plot.
- Use the viewer's selected window for live snapshots and the complete captured
  execution for sealed reports, preserving fixed evidence cuts.
- Add Python, Rust, C# and C++ authoring and one existing SDK plot example.
- Regenerate tracked bindings through project tasks. Preserve reserved retired
  kind tags and the current v1 development storage/evidence baseline.
- Keep `over_time()` and changes to Watch cadence outside this change.

## Capabilities

### New Capabilities

- `heat-strip`: category/facet reduction, presentation and authoring across live
  documents and frozen reports.

### Modified Capabilities

None. Existing `plot-window`, `plot-refresh` and `document-query-evaluation`
contracts continue to govern window ownership, refresh and fixed cuts.

## Impact

Protocol model and snapshot contracts; storage projection and kind reduction;
document-to-request mapping; snapshot kind dispatch; shared plot projection,
grouping, rendering and report consumption; four SDK builders and focused tests.
Rust builders belong to `crates/dokime-protocol/src/builders.rs`, consumed by the
SDK, rather than a new SDK-owned implementation.

Required consumer edits extend beyond `packages/plots/src/catalog/`: see the
source inventory in `tasks.md`. No dependency, manifest, lockfile, CI or `docs/`
edits are planned. The documentation owner must update the query/GUI contracts
and stale report ownership statements identified in `design.md`.

## Agreement and Activation

The owner has locked the feature behaviour in the task. The owner agreed the proposal, wire mapping, scenarios and inventory and activated
`heat-strip-plot` on `feat/heat-strip-plot`. The activation marker was verified
before implementation. The worker hands back local commits without pushing.
