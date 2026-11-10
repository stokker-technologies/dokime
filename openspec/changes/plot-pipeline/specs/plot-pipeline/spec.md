## ADDED Requirements

### Requirement: Windowed Plots From Stored Buckets

A plot over a time window SHALL be answered from stored time buckets, reading raw samples only for the newest edge and for ranges no bucket covers.

#### Scenario: Repeated Update

- **WHEN** a live plot over a window updates while only new samples arrive
- **THEN** the query reads the new samples and reuses the stored buckets for the rest of the window

#### Scenario: Spike Preserved

- **WHEN** a window contains a spike shorter than one bucket
- **THEN** the plot shows it, because each bucket keeps its minimum and maximum

### Requirement: Width-Aware Requests

A plot request SHALL carry the visible range and the panel width, and the backend SHALL return a number of points proportional to that width.

#### Scenario: Zoom

- **WHEN** the user zooms into part of a plot
- **THEN** the plot is redrawn from data bucketed for the new range, showing detail the wider view did not

#### Scenario: Irregular Timestamps

- **WHEN** samples are unevenly spaced in time
- **THEN** buckets are equal in time, not in sample count

### Requirement: Incremental Live Updates

A plot watch SHALL send a snapshot when it opens and changed buckets afterwards, and SHALL NOT delay a redraw for exact statistics.

#### Scenario: New Samples

- **WHEN** new samples arrive for a watched plot
- **THEN** the watch sends only the buckets that changed

#### Scenario: Late Sample

- **WHEN** a sample arrives with a timestamp inside an already sent bucket
- **THEN** the watch sends that bucket again with the sample included

#### Scenario: Reconnect

- **WHEN** a client reconnects to a watch
- **THEN** it receives a fresh snapshot and continues with changed buckets
