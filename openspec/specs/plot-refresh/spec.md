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

### Requirement: Atomic Timed Watch Revisions
A document watch SHALL evaluate its dirty plots together at one captured evidence cut, reusing query deduplication and bounded storage reads. It SHALL send each completed dirty set in one message, with per-plot outcomes in request order. The GUI SHALL commit the decoded revision through one shared animation-frame update.

#### Scenario: Shared Stream Publication
- **WHEN** one publication changes several plots in a document watch
- **THEN** one message carries every affected plot's outcome at the captured cut
- **AND** the frontend cannot paint a partial revision as individual plot messages arrive

#### Scenario: Opening and Keepalive
- **WHEN** a watch opens
- **THEN** it acknowledges its plots and delivers their opening outcomes together
- **AND** keepalives contain only pending notices, without replaying delivered data

#### Scenario: Saved Handoff
- **WHEN** an admitted telemetry revision moves into saved storage after delivery
- **THEN** the watch sends no duplicate snapshot for that plot and revision

#### Scenario: Revision Budget and Plot Failure
- **WHEN** a dirty set exceeds one transport message's budget
- **THEN** its plots share the available budget and reduce retained payloads without rereading evidence
- **AND** a refused plot reports its own error while healthy plots remain in the same completed dirty set
- **AND** finite All requests retain independent per-plot delivery and budgets
