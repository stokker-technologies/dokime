# Plot Refresh Specification

## Purpose

Deliver manually refreshed All plots independently from bounded committed previews, preserving exact-value requirements and timed-window behavior.

## Requirements

### Requirement: Independent All Delivery
The GUI SHALL deliver All snapshots independently per plot within existing transport and request budgets.

#### Scenario: One Plot Fails
- **WHEN** one plot snapshot fails or exceeds the transport budget
- **THEN** other plots complete and only the failed plot displays an error

#### Scenario: Manual Refresh
- **WHEN** All is selected and runtime, summary or visibility inputs change
- **THEN** no additional query starts until explicit Refresh or view entry

### Requirement: Bounded All Projection
Declared All plots SHALL use write-time bounded report previews and preserve exactness refusals rather than approximate exact analyses.

#### Scenario: Multi-Hour History
- **WHEN** a declared plot covers four hours of samples
- **THEN** its response is point-budgeted and its query reads bounded preview state instead of raw history

#### Scenario: Exact Analysis Refusal
- **WHEN** an exact analysis exceeds the preview's exact sample capacity
- **THEN** it reports an analysis error within the requested display point budget

#### Scenario: Concurrent Save
- **WHEN** telemetry is saved while a current All snapshot opens
- **THEN** the preview and its physical frontier are captured in one SQLite read snapshot without falling back to raw history because the writer advanced
- **AND** explicitly historical queries continue reading their requested cut exactly

### Requirement: Stored Field Expressions
Plain same-stream telemetry field expressions SHALL use the stored numeric projection when otherwise eligible.

#### Scenario: XY Fields
- **WHEN** both axes select fields without transforms
- **THEN** stored and expression representations produce equivalent values, labels and formats

#### Scenario: Exact Integer Statistics
- **WHEN** a field expression contains integers beyond the numeric projection's safe range
- **THEN** the typed exact path preserves aggregation before conversion to floating point

### Requirement: Marker Budget
Plot markers SHALL obey the display point budget in every projection and transport reduction.

#### Scenario: Excess Markers
- **WHEN** selected markers exceed the point budget
- **THEN** the response contains at most that budget of deterministic markers
