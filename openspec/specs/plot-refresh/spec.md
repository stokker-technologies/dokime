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

### Requirement: Marker Budget
Plot markers SHALL obey the display point budget in every projection and transport reduction.

#### Scenario: Excess Markers
- **WHEN** selected markers exceed the point budget
- **THEN** the response contains at most that budget of deterministic markers
