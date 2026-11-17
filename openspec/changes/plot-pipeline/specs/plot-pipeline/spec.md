## ADDED Requirements

### Requirement: Windowed Plots From Stored Buckets

A time-series plot over a time window SHALL be answered from stored time buckets, reading raw samples only for the newest edge and for ranges no bucket covers. Contour binning and grid/trajectory frame semantics belong to plot-kinds.

#### Scenario: Repeated Update

- **WHEN** a live plot over a window updates while only new samples arrive
- **THEN** the query reads the new samples and reuses the stored buckets for the rest of the window

#### Scenario: Spike Preserved

- **WHEN** a window contains a spike shorter than one bucket
- **THEN** the plot shows it, because each bucket keeps its minimum and maximum

### Requirement: Width-Aware Requests

A time-series plot request SHALL carry the visible range and the panel width, and the backend SHALL return a number of points proportional to that width. Two-dimensional contour resolution and playback frames belong to plot-kinds.

#### Scenario: Zoom

- **WHEN** the user zooms into part of a plot
- **THEN** the plot is redrawn from data bucketed for the new range, showing detail the wider view did not

#### Scenario: Irregular Timestamps

- **WHEN** samples are unevenly spaced in time
- **THEN** buckets are equal in time, not in sample count

### Requirement: Incremental Live Updates

A plot watch SHALL open with a snapshot and SHALL NOT delay drawing for exact statistics. Refresh policy SHALL follow per-kind desktop benchmark evidence: cheap bounded kinds default to invalidation/refetch of the visible window; time-series bucket deltas remain only where measured bandwidth justifies them. Both paths SHALL preserve the captured cut and late-sample correctness.

#### Scenario: Bounded Replacement Kind

- **WHEN** relevant samples change a kind assigned invalidation/refetch
- **THEN** the visible plot requests a fresh bounded result without pushed bucket state

#### Scenario: Late Sample on a Delta Path

- **WHEN** a late sample changes an already delivered time-series bucket on a measured delta path
- **THEN** the watch replaces that bucket with the correct result at the new cut

#### Scenario: Reconnect

- **WHEN** a client reconnects
- **THEN** it receives a fresh snapshot and continues using its kind's measured policy

### Requirement: Dokime-Owned Signal Kernels

The kernels stay hand-maintained and move into Dokime; `tsdownsample` was tried and dropped because it needs nightly Rust. Dokime SHALL own its product FFT/presentation kernels as private modules; generic statistics, downsampling and bucket merges SHALL compose Harmos operations under operation-boundary, with no vendored dependency or Harmos signal namespace.

#### Scenario: Stable Compiler

- **WHEN** a focused plot or signal test builds with the stable compiler
- **THEN** it uses Dokime-owned kernels without `tsdownsample` or `argminmax`
