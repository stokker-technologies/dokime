## ADDED Requirements

### Requirement: Clean Kind Catalog

Dokime SHALL expose idiomatic plot kinds including TIME_SERIES, FREQUENCY_SERIES, XY, TRAJECTORY, CONTOUR and GRID_CHART. Retired names SHALL be refused with an error naming the replacement kind and option or card. Additional consolidation remains proposed until owner confirmation.

#### Scenario: Retired Kind

- **WHEN** a document declares XY_VALUE_HEATMAP after the rename
- **THEN** validation refuses it and names CONTOUR as the replacement

#### Scenario: Retired Variant

- **WHEN** the proposed XY_INTERVAL consolidation is confirmed and an author declares that retired name
- **THEN** validation refuses it and directs the author to XY with the sigma interval option

### Requirement: Numeric Contour Grid

CONTOUR SHALL accept numeric x, y and z expressions. The backend SHALL bin scattered samples to the panel resolution and send a grid with explicit empty bins. Cells and d3 filled contour bands with isolines SHALL show gaps for empty bins without interpolation.

#### Scenario: Scattered Samples Leave Gaps

- **WHEN** scattered x/y/z samples populate only some panel-resolution bins
- **THEN** the backend marks the other bins empty and both cell and contour-band views leave gaps there

#### Scenario: Zero Is a Value

- **WHEN** one populated bin reduces to zero and an adjacent bin has no samples
- **THEN** zero has a value color and the absent bin remains a gap

### Requirement: Generic Grid Chart

GRID_CHART SHALL render row-by-column cells with values for a selected instant. Data and document definitions SHALL supply row/column identities, order, labels, groups, reductions, highlights and wording. The interface SHALL require no domain-specific code or Wi-Fi constants.

#### Scenario: Another Domain Uses the Grid

- **WHEN** an author supplies sensor rows, location columns, values and instants
- **THEN** the interface draws the chart without any domain-specific interface changes

#### Scenario: Occupancy Uses Declared Cells

- **WHEN** the host document supplies bands, channel lists, values and active-cell metadata
- **THEN** live and report views draw that declared grid without inferring channel knowledge

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
