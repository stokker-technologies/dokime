## Why

Plots need a display-only unit conversion without changing evidence or query semantics. The owner agreed to `plot.unit("mm"); plot.scale(1e3);` for time-series y values.

## What Changes

- Add an optional positive finite display scale, defaulting to 1.0 and omitted from serialization when absent.
- Convert returned numeric time-series y points, statistics and references through the shared live/report presentation path.
- Refuse scales on other plot kinds, custom analyses and non-time x axes during document validation, naming the plot. Explicit conversion uses numeric presentation, including encoded 0/1 Boolean values.
- Harmonize Rust/Python authoring and document that units are labels and filters/repeats retain stored units.
- Preserve query plans, expressions, exports, criteria and evidence. Arithmetic expressions remain unsupported.

## Capabilities

### New Capabilities
- `plot-display-scale`: validated display conversion for numeric time series.

### Modified Capabilities

## Impact

Domain document definitions and projection, shared GUI/report plot presentation, Python SDK, focused tests and authoring/GUI documentation. The owner's required Rust check, GUI TypeScript build and focused tests replace broader local validation; no lint, starters, full lanes or simulations. Delivery is authorized through an immediately squash-merged PR to dev after the required diff scanner prints CLEAN.
