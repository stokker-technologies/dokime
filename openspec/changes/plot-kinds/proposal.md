# Plot Kinds

## Why

The wire advertises seventeen kinds, but the interface draws ten catalog
components. Some names describe options, six kinds are rejected, and two
heatmap names draw the same interpolated numeric surface. Authors need names
that describe the input and the drawing, with parity in saved reports.

This change records the owner-approved target contract, not landed behavior.
The kind catalog, options, builders, reductions and playback semantics below
are locked for implementation.

## Locked by the Owner

- Retain `TIME_SERIES`, `FREQUENCY_SERIES`, `XY` and `TRAJECTORY`; name the
  numeric three-dimensional map `CONTOUR` and the discrete cell view `GRID_CHART`.
- `CONTOUR` takes numeric x, y and z. The backend bins scattered samples with
  panel resolution as an upper bound, like time buckets, and sends a grid with
  its actual dimensions and edges. Whole cells merge with exact reducers to fit
  unchanged display and transport limits. Empty bins stay
  empty and visible as gaps; never interpolate missing samples. Use the
  existing d3 contour and scale modules for cells or filled bands with isolines.
- `GRID_CHART` draws values in rows and columns. It replaces channel occupancy;
  bands, channel lists and wording come from data and document definitions.
  The interface contains no Wi-Fi knowledge.
- `TRAJECTORY` and `GRID_CHART` share one playback control below each plot:
  instant slider, Play, Pause and Replay. It plays already-loaded data, makes
  no data requests, and works in frozen HTML. Extract the occupancy slider
  and its follow-latest behavior into this control.
- Selection belongs to one plot. Cursor linking and shared time selection
  between plots are explicitly out of scope.

## Kind Table

Names below omit the protobuf `PLOT_KIND_` prefix. The catalog has ten kinds.
The table records actual behavior, including absent renderers, rather than
assuming every enum value works. Inline examples are Python builder
expressions; stream field handles are expressions, never parsed strings.

| Old Kind | New Kind or Card | Input, Drawing and What the Author Writes Instead |
| --- | --- | --- |
| `TIME_SERIES` | `TIME_SERIES` | Timestamp plus numeric/Boolean values; retain line/points/step styling. `dokime.plot.time_series(value)` |
| `BAR` | `BAR` | Category/value pairs; currently latest value per category, horizontal bars. `dokime.plot.bar(value).x(category)` |
| `GROUPED_BAR` | `BAR` | Currently rejected; same category/value shape plus series. Arrangement option: `dokime.plot.bar(value).x(category).series_by(group).arrangement(dokime.plot.BarArrangement.GROUPED)` |
| `STACKED_BAR` | `BAR` | Currently rejected; same inputs as grouped bars. `dokime.plot.bar(value).x(category).series_by(group).arrangement(dokime.plot.BarArrangement.STACKED)` |
| `PIE` | `PIE` | Currently rejected; categorical nonnegative magnitudes, part-to-whole sectors. `dokime.plot.pie(value).x(category)` |
| `DONUT` | `PIE` | Currently rejected; same inputs, a hole option. `dokime.plot.pie(value).x(category).hole(0.5)` |
| `PARETO` | `BAR` | Currently rejected; same category/value pairs, descending bars and cumulative percentage line. `dokime.plot.bar(value).x(category).pareto(True)` |
| `XY` | `XY` | Paired numeric coordinates; retain line, points, line-points and step. `dokime.plot.xy(value).x(position)` |
| `XY_INTERVAL` | `XY` | Currently backend x-bin mean/sigma bounds drawn as intervals, not authored low/high fields. Keep that reduction as an option on the same numeric pairs: `dokime.plot.xy(value).x(position).interval(dokime.plot.XyInterval.SIGMA)` |
| `TRAJECTORY` | `TRAJECTORY` | Ordered numeric x/y samples drawn as a line with points; retain timestamps for the new instant playback. `dokime.plot.trajectory(value).x(position)` |
| `HEATMAP` | `CONTOUR` | Current renderer consumes numeric x/y/z and interpolates; replace with backend bins and gaps. `dokime.plot.contour(value).x(position).y(height).render(dokime.plot.ContourRender.CELLS)` |
| `XY_VALUE_HEATMAP` | `CONTOUR` | Same renderer and numeric triples; no second kind. `dokime.plot.contour(value).x(position).y(height).render(dokime.plot.ContourRender.BANDS)` |
| `WIFI_CHANNEL_OCCUPANCY` | `GRID_CHART` | Currently scan frames with per-channel counts and domain-specific groups/highlights; use document-defined row/column/value/frame bindings. `dokime.plot.grid_chart(value).row(row).column(column).frame(instant).reduce(dokime.plot.Reduction.MAX)` |
| `FREQUENCY_SERIES` | `FREQUENCY_SERIES` | Numeric sampled signal plus sample rate; backend FFT, frequency/amplitude drawing. `dokime.plot.frequency_series(value).sample_rate_hz(1000)` |
| `HISTOGRAM` | `HISTOGRAM` | Numeric samples reduced into value bins/counts, with statistics. `dokime.plot.histogram(value)` |
| `BOX_SUMMARY` | `BOX_PLOT` | Numeric sample distributions per group reduced to quartiles, whiskers and outliers. `dokime.plot.box_plot(value).series_by(group)` |
| `AGGREGATE_TABLE` | Summary Card, No Plot Kind | Currently rejected; reuse aggregate expressions and summary tiles rather than invent a table renderer. `dokime.Summary().tile("Mean", value.mean())` |

Grouped and stacked arrangements and the Pareto option require real rendering,
not aliases to the current bars. Pareto accepts one finite nonnegative value
per category, sorts descending, and shows cumulative percentage on a secondary
axis; it is mutually exclusive with grouped/stacked arrangement and series
bindings. Equal values retain declared category order; a zero total shows bars
without a cumulative line. Pie accepts one finite nonnegative magnitude per
category, rejects negative or nonfinite values, and shows an empty state for a
zero total. Its hole fraction defaults to zero and must be finite in [0, 1).
Duplicate category totals are refused for Pareto and pie; authors aggregate
samples explicitly before supplying totals. Summary tiles replace an unsupported promise, not an existing working table. The
ordinary typed Table card stays for tabular data.

Keep histogram and box plot distinct from bar: their input is a distribution
of samples, not category totals. Keep frequency distinct from time series:
it requires sampled-signal/FFT semantics. Trajectory keeps time order and instant
selection rather than becoming an XY style. A categorical grid does not become
a numeric surface. XY intervals keep the existing sigma reduction semantics;
they do not introduce a new endpoint input shape.

## Contour Definition and Builders

`CONTOUR` maps numeric x/y positions to a numeric z value, not a perspective
3D mesh. Requests carry the visible x/y bounds and panel width and height;
the backend returns bin extents, dimensions, z values and validity. Empty
is distinct from zero. The interface never constructs a scattered-point surface.
The saved report uses its capture resolution and the same validity mask.

Reduce multiple samples in a bin by mean z by default, with explicit max/min
reduction options. Bin geometry follows the declared axis scales. Invalid numeric coordinates/values contribute
no sample. Both rendering modes mask missing cells; contour paths and fills
must not bridge holes. Disable d3 contour smoothing for the binned field and
clip bands and isolines to valid cells; never fill absent bins to feed d3.

These flat builders are thin conveniences over `dokime.v1.Plot`: the
constructor takes z and sets the kind; `.x` and `.y` set coordinate expressions.
Presentation and reduction options become typed protobuf fields, not another DSL.
`scan` below is an already-declared stream; no nested axis or surface builders.

```python
x = scan.field("position_x")
y = scan.field("position_y")
z = scan.field("temperature").scale(1.0)
surface = (dokime.plot.contour(z).x(x).y(y)
           .label("Temperature Map").unit("°C")
           .reduce(dokime.plot.Reduction.MEAN)
           .render(dokime.plot.ContourRender.BANDS))
document = dokime.Document("Scan").section(dokime.Section("Map").plot(surface))
```

```rust
let x = scan.field("position_x");
let y = scan.field("position_y");
let z = scan.field("temperature").scale(1.0);
let surface = Plot::contour(z).x(x).y(y)
    .label("Temperature Map").unit("°C")
    .reduce(Reduction::Mean).render(ContourRender::Bands);
let document = Document::new("Scan").section(Section::new("Map").plot(surface));
```

Use the typed `ContourRender` option `CELLS` for colored bins and `BANDS` for
filled bands with isolines. Builders accept enum values, not strings:
Python uses `dokime.plot.Reduction.MEAN` and `dokime.plot.ContourRender.BANDS`;
Rust uses `Reduction::Mean` and `ContourRender::Bands`. `Reduction` contains
mean, min and max; `BarArrangement` contains grouped and stacked;
`XyInterval` contains sigma, with absent interval meaning ordinary XY.
Python enum members use uppercase snake case and Rust variants use PascalCase.
The constructor and method names in these examples are the contract in both
languages, with idiomatic enum casing. Ordinary BAR has no arrangement option
set; grouped/stacked require a series binding. `.pareto(True)` in Python and
`.pareto(true)` in Rust enable Pareto; `.hole(fraction)` sets the pie hole.
Expression chaining assembles `DataExpression` messages as required by `grpc-plugins`.

## Grid Chart Definition and Builders

`GRID_CHART` takes row key, column key, cell value and instant expressions.
Rows and columns are discrete identities, not continuous x/y coordinates.
Definitions supply ordered keys/labels, optional groups, units, highlights and
summary wording. The backend produces generic frames, applying the declared
cell reduction within each frame. Missing cells stay distinct from zero.
For occupancy the host plugin declares bands, channel encoding/lists, active
channel, scan frames and count wording; none is inferred by the product.

The message shape uses explicit row/column/frame bindings and typed axis
entries, rather than overloading numeric contour axes. The example's `rows`
and `columns` are sequences of declared key/label/order/group entries; `active`
is an expression identifying a highlighted cell. All are protobuf data, with
no plugin UI callbacks. The reducer is required and explicit (max in this example),
with no implicit default. Row, column, value and frame are `DataExpression`
bindings; row and column keys match the declared entries, value is numeric,
and frame is a timestamp. Each entry carries key, label, order and optional
group; highlights bind row/column identities. Unknown keys and an absent
reducer are validation errors. The backend groups by frame, row and column
and applies the declared mean, min or max to duplicate cells. Invalid numeric
values contribute no sample; an empty cell remains absent. These bindings,
entries, reducer and highlights are typed fields on the grid definition,
separate from numeric contour coordinates.

```python
grid = (dokime.plot.grid_chart(readings.field("value"))
        .row(readings.field("row_key")).column(readings.field("column_key"))
        .frame(readings.field("timestamp")).rows(rows).columns(columns)
        .reduce(dokime.plot.Reduction.MAX).highlight(active).label("Sensor Grid"))
document = dokime.Document("Survey").section(dokime.Section("Cells").plot(grid))
```

```rust
let grid = Plot::grid_chart(readings.field("value"))
    .row(readings.field("row_key")).column(readings.field("column_key"))
    .frame(readings.field("timestamp")).rows(rows).columns(columns)
    .reduce(Reduction::Max).highlight(active).label("Sensor Grid");
let document = Document::new("Survey").section(Section::new("Cells").plot(grid));
```

## Shared Playback

Extract the occupancy slider, keyboard/pointer interaction and follow-latest
behavior into one shared implementation used by trajectory and grid, including
the report bundle. The instant list comes from each plot's already-loaded
payload, in timestamp order. Seeking, Play, Pause and Replay cause no query,
watch restart, window change or report fetch. Normal live delivery may continue
independently; follow-latest applies to those incoming frames.

Initial selection follows latest; seeking an earlier instant or starting playback suspends follow-latest;
Latest or committing the last instant restores it. Playback reaching the end
stops without restoring follow-latest; only that explicit user selection does.
Play advances from the selection, Pause holds it, Replay starts at the first available instant, and
playback stops at the loaded end. Empty/single-instant data disables animation.
Trajectory shows the ordered path through the selected instant and marks the
current position; grid shows only the selected frame. No synthetic frames or
interpolated instants. Every plot owns its selection and playback state.

The frozen report embeds the available instants and their plot data, plus all
control code/assets. Follow-latest means its fixed final instant there. It
must work with the backend stopped and networking disabled. Existing bounded
report previews remain; playback promises the instants present in the payload,
not retrieval of raw evidence or every retained sample.

## What Stays and What It Removes

Keep raw evidence, filters, grouping, units, references, numeric/Boolean time
series, FFT options, histogram statistics, XY styling, viewport zoom, existing
time buckets and normal live transport. Keep unrelated existing time-series
cursor behavior; this change adds no cross-plot playback linkage.

Remove retired kind constructors/names, duplicate heatmap branches, client
inverse-distance interpolation, Wi-Fi render/analysis/disk-backed special cases
and the private occupancy slider. No compatibility aliases or automatic data
reset. Retired names fail validation naming their replacement kind and option
or card, for example `XY_INTERVAL: use XY with interval=sigma`. Reserve retired
protobuf names/numbers; never reinterpret an old numeric tag as another kind.
Development schemas stay at v1; incompatible definitions require an explicitly
selected fresh root while preserving existing storage. Already-exported HTML
retains its bundled renderer; new reports use the new catalog.

## Ownership and Relation to Other Changes

- `grpc-plugins` owns the language-neutral document/message architecture and
  expression chains. This change extends that contract; no parallel document API.
- `plot-pipeline` retains stored time buckets, width-aware time queries, watch
  deltas, columnar transport and private signal kernels. This change owns the
  two-dimensional grid payload/binning and kind-specific interface/report use
  of that transport. Its generic point-budget wording is narrowed to avoid
  prescribing one-dimensional reduction for contours or playback frames.
- `domain-neutral-interface` retains grouped choices, traceability, declared
  units and removal of stream-name inference. Its grid replacement, occupancy
  migration/removal, supported-kind cleanup and grid/report proof move here.
  That change depends on this one for plot domain neutrality.
- `screenshots-task` retains the capture task and scene framework. This change
  updates the bench documents and its `plots`/`report` scenes for the new catalog,
  sparse contour gaps and per-plot playback; no duplicate capture workflow.

## Impact

Protocol `proto/dokime/v1/documents.proto` and generated contracts; plot validation,
query/preview/frame payloads in the runtime and their transport consumers;
`crates/dokime-gui/src/plots` and document dispatch/adapters; the self-contained
runtime HTML report template (`crates/dokime-runtime/dist-report/report.html`),
its capture/embedding owner (`crates/dokime-runtime/src/report/html.rs`) and
the bundled interface modules in `crates/dokime-gui/src/report`; SDK builders in all languages
and generated examples/starters; host and bench plugin document definitions;
screenshots task scenes and fixtures; product guide, plot authoring guidance
and any affected README claims. Every advertised kind/option must actually draw
in both live and report views. This specification edits no implementation files.
