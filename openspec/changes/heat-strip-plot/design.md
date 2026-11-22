# Heat Strip Design

## Context

Proposed design; see `proposal.md` for scope.
`docs/project/architecture.md:646` states: "Plot/document readers consume
projected batches." Its purpose is one typed evaluation path through the library.
Invariant 19 states: "Rendering Does Not Publish." Heat Strip preserves both.
`Reports.Create` embeds protobuf into the shared React report template; direct-HTML
ownership at architecture lines 653-655 and `docs/guides/gui.md:8` is outdated.

## Goals and Boundaries

One grouped read supplies every cell/facet at one evidence cut. Existing filters,
expression evaluation, query limits, cancellation and viewer windows apply.
Do not change Watch/notification cadence or `crates/dokime-gui/src/plots/live-data.ts`.
No waterfall, new registry, report renderer, cache, dependency or data migration.

## Authoring

```python
plot = heat_strip(WIFI_NETWORK.field("network_count"), label="Wi-Fi Channel Occupancy")
plot.cells(WIFI_NETWORK.field("wifi_channel"))
plot.facet(WIFI_NETWORK.field("wifi_band"))
plot.reduce(Reduction.LAST)
```

```rust
let plot = Plot::heat_strip(value_expression)
    .label("Wi-Fi Channel Occupancy")
    .cells(cell_expression)
    .facet(facet_expression)
    .reduce(Reduction::Last);
```

Extend the existing protocol-owned Rust authoring value and C#/C++ plot builders;
all four SDKs serialize one model.

## Wire Mapping

Proposed additions preserve all existing tags and reserved names:

```proto
// PlotKind
PLOT_KIND_HEAT_STRIP = 21;
// Reduction: reuse MEAN and MAX; add LAST and COUNT without changing MIN.
REDUCTION_LAST = 4;
REDUCTION_COUNT = 5;
// Plot: y contains exactly one VALUE; existing reduction is reused.
.dokime.model.v1.DataExpression cell = 26;
.dokime.model.v1.DataExpression facet = 27;
// PlotRequest: same expressions, no second definition or conversion type.
.dokime.model.v1.DataExpression cell_expression = 40;
.dokime.model.v1.DataExpression facet_expression = 41;
```

Reuse `PlotFrame`/`PlotColumn`: one column per facet, typed CELL in `x`, nullable
reduced VALUE in `y`, contribution time in `instants`, and existing series labels.
Build the union of cells once and align every facet to it, leaving missing slots
null. No Heat Strip-specific payload family is needed. Validate required CELL,
one numeric VALUE and supported reducers at the existing model/query boundaries.

## Evaluation and Presentation

Project VALUE, CELL and optional FACET together through the existing expression
owner. Traverse selected batches once, grouping by typed facet/cell identity;
reuse library accumulators for MAX, MEAN and COUNT. LAST compares source time then
durable position, independently of ingestion order. Ignore absent/nonfinite
VALUE contributions; retain observed CELL identities even if all values are absent.
COUNT counts present finite contributions and an all-absent group stays empty.
MAX hover time identifies its latest tied contributor; MEAN/COUNT use the latest
contributor time. Missing CELL or requested FACET supplies no group.
Sort numeric cells by numeric value without f64 identity loss; sort textual cells
lexically, with deterministic numeric-before-text ordering for mixed inputs.
Apply stable facet ordering and the same shared cell order to every strip.
Bound group/output cardinality by existing query budgets and page through indexed
source batches, retaining accumulator state rather than raw history. Refuse excess
groups with guidance to narrow the window; multi-day duration alone is no refusal.
The shared renderer draws equal squares, cell labels, facet labels and one legend;
one colour domain covers all finite values. A constant domain remains renderable.
Empty cells are uncoloured. Hover exposes cell, facet, reduced value and its time.
Formatting uses `.` without grouping and labels use `Name [unit]`.
Frozen captures use the same snapshot and renderer for the whole execution at the
sealed cut, independently of the live selection. No interpolation is involved.

## Risks and Migration

High cardinality can exceed output budgets; deterministic refusal preserves bounds.
Typed identity and missing values need explicit tests to prevent merging or zero-fill.
Migrate every dispatch/consumer together; retain tracked bindings, reserved retired
kind tags and existing kinds.
If the pinned library lacks a required facility, stop that item and report the gap.

## Ownership Findings

`GridChart` and `GridDefinition` in `proto/dokime/model/v1/plots.proto` have identical
fields. Authored `Plot.grid` uses the former; `PlotRequest.grid` uses the
latter through `host/documents/plot.rs`. Recommend reusing `GridChart` and deleting
`GridDefinition` in owner-approved work; do not merge either mechanism here.
Documentation: `docs/guides/query-api.md:22`, `docs/guides/gui.md:459` need Heat Strip; architecture line 646 needs grouped reads.
