## ADDED Requirements

### Requirement: Clean Kind Catalog

Dokime SHALL expose exactly TIME_SERIES, FREQUENCY_SERIES, XY, TRAJECTORY, CONTOUR, GRID_CHART, BAR, PIE, HISTOGRAM and BOX_PLOT. Grouped, stacked and Pareto SHALL be BAR options; donut SHALL be a PIE option; interval SHALL be an XY option. Both former heatmaps SHALL use CONTOUR. Aggregate tables SHALL use document summary cards. Retired names SHALL fail with an error naming the replacement kind and option or card, without aliases.

#### Scenario: Retired Kind

- **WHEN** a document declares XY_VALUE_HEATMAP after the rename
- **THEN** validation refuses it and names CONTOUR as the replacement

#### Scenario: Retired Variant

- **WHEN** an author declares the retired XY_INTERVAL name
- **THEN** validation refuses it and directs the author to XY with the sigma interval option

### Requirement: Bar and Pie Options

Grouped and stacked BAR SHALL require series bindings. Pareto SHALL reject arrangements and series bindings, accept one finite nonnegative total per category, sort descending and draw cumulative percentage on a secondary axis. PIE SHALL accept one finite nonnegative total per category and a finite hole fraction in [0, 1), default zero. Both SHALL reject duplicate category totals. PIE with zero total SHALL show an empty state; zero-total Pareto SHALL omit the cumulative line.

#### Scenario: Invalid Totals and Options

- **WHEN** Pareto has grouped arrangement or a series binding, either Pareto or pie has duplicate categories or negative/nonfinite totals, or a pie hole is outside [0, 1) or nonfinite
- **THEN** validation refuses the definition with the invalid option or input identified

#### Scenario: Pareto Ordering

- **WHEN** Pareto receives category totals 2, 3 and 3
- **THEN** bars show 3, 3 and 2, ties retain declared order, and the cumulative line shows 37.5%, 75% and 100%

#### Scenario: Zero Totals

- **WHEN** all supplied totals are zero
- **THEN** pie shows an empty state and Pareto shows zero bars without a cumulative line

#### Scenario: Distinct Variant Rendering

- **WHEN** valid grouped, stacked, Pareto, donut and sigma interval definitions are rendered
- **THEN** they draw grouped bars, stacked bars, cumulative-percentage bars, sectors with the declared hole and backend x-bin mean/sigma intervals respectively in live and report views

### Requirement: Numeric Contour Grid

CONTOUR SHALL accept numeric x, y and z expressions. The backend SHALL bin scattered samples to the panel resolution and send a grid with explicit empty bins. Cells and d3 filled contour bands with isolines SHALL show gaps for empty bins without interpolation.

#### Scenario: Scattered Samples Leave Gaps

- **WHEN** scattered x/y/z samples populate only some panel-resolution bins
- **THEN** the backend marks the other bins empty and both cell and contour-band views leave gaps there

#### Scenario: Zero Is a Value

- **WHEN** one populated bin reduces to zero and an adjacent bin has no samples
- **THEN** zero has a value color and the absent bin remains a gap

### Requirement: Contour Reduction and Masking

CONTOUR SHALL default to mean z per bin and support explicit min/max reduction. Bin geometry SHALL follow declared axis scales and visible bounds at panel resolution, or report capture resolution. Invalid numeric x/y/z SHALL contribute no sample. The payload SHALL include extents, dimensions, values and validity. Both modes SHALL mask gaps; d3 smoothing SHALL be disabled and bands/isolines clipped to valid cells.

#### Scenario: Duplicate and Invalid Samples

- **WHEN** a bin receives z values 2 and 6 plus an invalid sample
- **THEN** default reduction yields 4, explicit min yields 2 and explicit max yields 6 without filling neighboring empty bins

### Requirement: Generic Grid Chart

GRID_CHART SHALL render row-by-column cells with values for a selected instant. Data and document definitions SHALL supply row/column identities, order, labels, groups, reductions, highlights and wording. The interface SHALL require no domain-specific code or Wi-Fi constants.

#### Scenario: Another Domain Uses the Grid

- **WHEN** an author supplies sensor rows, location columns, values and instants
- **THEN** the interface draws the chart without any domain-specific interface changes

#### Scenario: Occupancy Uses Declared Cells

- **WHEN** the host document supplies bands, channel lists, values and active-cell metadata
- **THEN** live and report views draw that declared grid without inferring channel knowledge

### Requirement: Explicit Grid Bindings

GRID_CHART SHALL carry row, column, numeric value and timestamp frame expressions, typed key/label/order/optional-group entries, highlights and an explicit mean/min/max reducer with no default. Unknown keys or absent reducers SHALL fail validation. The backend SHALL group samples by frame/row/column and reduce duplicate cells, excluding invalid numeric values. Missing cells SHALL remain distinct from zero; contour coordinate fields SHALL NOT substitute for discrete bindings.

#### Scenario: Required Reducer

- **WHEN** a grid omits its reducer or uses an undeclared row or column key
- **THEN** validation refuses it and identifies the missing reducer or unknown key

#### Scenario: Duplicate Cells

- **WHEN** one frame supplies values 2 and 6 for one declared cell with max reduction and no samples for another cell
- **THEN** the first cell shows 6 and the other remains absent

### Requirement: Shared Plot-Local Playback

TRAJECTORY and GRID_CHART SHALL share one below-plot instant slider with Play, Pause, Replay and the occupancy slider's follow-latest behavior. Playback SHALL use only already-loaded data and issue no data requests. Each plot SHALL own its selection; playback SHALL NOT link cursors or time selections across plots.

#### Scenario: Playback Requests No Data

- **WHEN** a user seeks, plays, pauses and replays either kind over its loaded instants
- **THEN** only its selected instant changes and no query, watch restart or data fetch is issued by playback

#### Scenario: Independent Selection

- **WHEN** a user seeks one trajectory while a grid chart is visible
- **THEN** the grid chart's selection and every other plot's cursor remain unchanged

#### Scenario: Follow Latest

- **WHEN** a plot is following latest and receives an instant through normal live delivery
- **THEN** its slider selects that latest instant without playback requesting data

### Requirement: Playback Selection and Endpoints

Playback SHALL initially follow latest. Seeking earlier or starting playback SHALL suspend follow-latest; Latest or committing the final instant SHALL restore it. Play SHALL advance from selection, Pause SHALL hold, and Replay SHALL start at the first instant. Playback SHALL stop at the loaded end without restoring follow-latest. Empty/single-instant data SHALL disable animation. Trajectory SHALL show the ordered path through selection and mark its position; grid SHALL show only that frame.

#### Scenario: Selection and Live Arrivals

- **WHEN** a user seeks earlier and a new live instant arrives
- **THEN** selection stays fixed until Latest or committing the final instant restores follow-latest

#### Scenario: Replay to the Loaded End

- **WHEN** Replay starts with multiple loaded instants
- **THEN** selection starts at the first, advances in timestamp order without synthetic instants, stops at the loaded end and stays there on later live arrivals

#### Scenario: Pause and Sparse Instants

- **WHEN** playback is paused, or the payload contains fewer than two instants
- **THEN** Pause holds selection; empty and single-instant payloads disable animation, and no frame is synthesized

#### Scenario: Kind-Specific Selection

- **WHEN** a user selects an available instant
- **THEN** trajectory shows its path through that instant and marks the current position, while grid shows only that instant's frame

### Requirement: Frozen Report Playback

Saved HTML reports SHALL embed the plot data, available instants and shared playback assets for TRAJECTORY and GRID_CHART. Their playback SHALL work offline with the same controls and plot-local selection as the interface, within the captured payload.

#### Scenario: Backend Unavailable

- **WHEN** a saved report is opened with networking disabled and its backend stopped
- **THEN** both kinds can seek, play, pause and replay their embedded instants without requesting data

### Requirement: Language-Neutral Builders and Complete Consumers

Plot definitions SHALL remain dokime.v1 messages built by thin SDK builders with method-chained expressions. Every advertised kind and option SHALL render in the interface and report. Plugin documents, bench documents, screenshot scenes and guides SHALL use the cleaned definitions without retired aliases.

#### Scenario: Equivalent Authoring Languages

- **WHEN** Python and Rust authors declare the same contour or grid bindings and options
- **THEN** their builders produce equivalent dokime.v1 definitions rendered consistently live and in a report

### Requirement: Typed Builder Options

Flat builders SHALL use the constructor and method spellings in proposal.md. Options SHALL accept typed Reduction (mean/min/max), ContourRender (cells/bands), BarArrangement (grouped/stacked) and XyInterval (sigma) enums, with uppercase snake case Python members and PascalCase Rust variants. Strings SHALL NOT stand in for enums. Absent arrangement/interval SHALL mean ordinary BAR/XY. Pareto SHALL use a Boolean and pie hole a numeric fraction.

#### Scenario: Typed Options Across Languages

- **WHEN** Python uses .reduce(dokime.plot.Reduction.MEAN) and Rust uses .reduce(Reduction::Mean)
- **THEN** both serialize the same typed reducer; a string argument is rejected by the binding
