## MODIFIED Requirements

### Requirement: Atomic Timed Watch Revisions

A timed watch SHALL tag affected plot outcomes with their captured readable cut and per-plot revision. Jointly invalidated plots SHALL use the same cut but complete independently, without a document-wide message or paint barrier. Per-plot drawing density SHALL NOT fall with plot count. Delivery SHALL retain bounded messages, per-plot errors and stale-revision rejection.

#### Scenario: Shared Stream Publication

- **WHEN** one publication affects two plots and one query is slow
- **THEN** the ready plot paints without waiting for its peer, both outcomes identify the same cut, and neither plot paints an obsolete revision

#### Scenario: Opening and Keepalive

- **WHEN** a timed watch opens
- **THEN** visible plots receive independent opening snapshots and payload-free pending notices do not replay delivered data

#### Scenario: Saved Handoff

- **WHEN** an accepted-live revision becomes durable after delivery
- **THEN** the plot receives no duplicate drawing result for that revision

#### Scenario: Revision Budget and Plot Failure

- **WHEN** one plot exceeds its frame budget
- **THEN** its own reduction or typed error does not reduce another plot's density or prevent another plot from completing

## ADDED Requirements

### Requirement: Desktop Round-Trip Measurement

The desktop SHALL own a system benchmark reusing its native harness and feeding Nightly benchmarks.json on Windows and Linux. Before optimization it SHALL measure all query-to-presentation stages for 30/100 mixed plots over millions of samples. Later steps SHALL record comparable before/after results. The integration-plots bench SHALL remain.

#### Scenario: Complete Measurement Matrix

- **WHEN** the benchmark runs cold and warm during ingest, late samples, zoom, resize and navigation over multi-day history
- **THEN** it reports backend query, encoding/transport/native bridge, protobuf decode, retained-frame update, canvas draw and presentation p50/p95, first-visible/all-visible paint, CPU, peak memory, queue depth and bytes
- **AND** refusals or unavailable stages are explicit rather than reported as successful paint measurements

### Requirement: Plot-Local Live Interests

Timed viewport, width and settings changes SHALL update only the affected plot's query interest. Resize SHALL be debounced. Scheduling SHALL prioritize on-screen cards and release off-screen live interests. Documents SHALL support more than 32 plots with bounded concurrency; unchanged panel geometry SHALL retain density independently of plot count.

#### Scenario: One Plot Changes

- **WHEN** one plot is zoomed, resized or edited while another remains unchanged
- **THEN** only the changed plot is requeried and the other's watch, statistics and retained frame are not restarted

#### Scenario: One Hundred Plots

- **WHEN** a document containing 100 mixed plots opens
- **THEN** it is admitted, on-screen cards load first and equal-width plots retain the same drawing density as in a smaller document
- **AND** queued work and live interests remain bounded as navigation changes visibility

#### Scenario: Off-Screen and Return

- **WHEN** a timed plot leaves the screen and later returns after telemetry advances
- **THEN** it has no off-screen live query/statistics work and resumes with a fresh bounded snapshot at a readable cut

### Requirement: Scoped Recoverable Changes

Plots and live cards SHALL share one coalescing change source built on Harmos progress/follow facilities. Changes SHALL identify scope, stream/schema or record dependency, affected time range and readable frontier, separating accepted-live from durable progress. Consumers SHALL match scope, fields, filters and range before requerying and recover notification gaps by frontier comparison.

#### Scenario: Unrelated Scope

- **WHEN** a sample arrives on a selected stream name outside a plot's execution scope
- **THEN** that plot is not requeried

#### Scenario: Unchanged Selection

- **WHEN** a change affects only unselected fields, excluded filter values or times outside a fixed plot window
- **THEN** that plot is not requeried

#### Scenario: Schema or Record Dependency

- **WHEN** a schema or record change affects a visible plot's resolved dependencies
- **THEN** that plot is reevaluated even without a new telemetry sample and unrelated plots stay unchanged

#### Scenario: Lost Notification

- **WHEN** a consumer misses coalesced notifications or reconnects
- **THEN** it compares readable frontiers and obtains a correct snapshot without requiring every intermediate event

### Requirement: Measured Per-Kind Refresh

Cheap bounded frequency, XY, trajectory, contour, grid, aggregate and statistics results SHALL default to invalidation followed by visible-window refetch. Every kind SHALL have a benchmark-recorded policy. Time-series pushed bucket deltas SHALL remain only where measured bandwidth justifies them. Refresh policy SHALL preserve exactness, revision provenance, cancellation and backpressure.

#### Scenario: Bounded Result Changes

- **WHEN** relevant evidence changes for a visible cheap bounded plot
- **THEN** coalesced invalidation triggers its current visible-window request rather than retaining a second incremental query model

#### Scenario: Time-Series Delta Chosen

- **WHEN** benchmark evidence justifies bucket deltas for time series
- **THEN** opening/reconnect snapshots and late-sample bucket replacement retain their existing correctness

### Requirement: Independent Affected Statistics

Statistics SHALL be exact, coalesced independently of drawing and scheduled only for affected visible plots. Statistics results SHALL identify their plot revision and cut; obsolete results SHALL NOT overwrite newer results. Drawing SHALL NOT await statistics.

#### Scenario: Unchanged Plot

- **WHEN** one visible plot changes while another visible plot and an off-screen plot do not
- **THEN** only the affected visible plot's statistics are queried and its drawing can complete first
