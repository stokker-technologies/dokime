## Why

The owner approved replacing quantity-associated plot references with Y-axis references owned by the plot.

## What Changes

- Introduce ReferenceValue for f64 and existing expression handles, without adding constants to DataExpression.
- Replace Band/Line builders and quantity/axis selection with reference_band(label, min, max), reference_tolerance(label, center, tolerance), and reference_line(label, value).
- Resolve constants and expressions on the shared live/frozen presentation path, including per-case parameters and display scaling in stored units.
- Refuse reversed literal bounds and negative literal tolerance during plot validation. Unresolved or reversed dynamic references remain undrawn.
- Migrate domain, storage and wire contracts (development v1 clean break), GUI, starter, Python mappings and maintained guidance. Do not edit private plugins or the untracked favicon.

## Capabilities

### New Capabilities
- `plot-references`: plot-owned Y references with literal or expression values.

### Modified Capabilities

## Impact

Domain, runtime storage, server wire projections, shared GUI/report rendering, Python document mappings and public authoring guidance. README reviewed; its general claims remain valid. The owner overrides normal publication sequencing: commit coherent steps and push an early ready PR with an empty body; inspect relevant hosted results before immediate squash. Only the specified Rust check, GUI TypeScript build and focused deterministic tests run locally; no broad checks, lint, starters, lanes or simulation.
