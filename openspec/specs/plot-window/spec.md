# Plot Window Specification

## Purpose
Keep time-window selection in live viewers while reports cover the full captured evidence range.

## Requirements

### Requirement: Viewer-Owned Time Window

Plot definitions, stored document contracts and report presentation SHALL contain no authored time window. Live viewers SHALL select one window for all plots in the view, defaulting to 5 min with options ordered 1 min, 2 min, 5 min, 10 min, 30 min, 1 h, All.

#### Scenario: Live Selection

- **WHEN** a live view opens and its operator changes the window
- **THEN** its initial query covers the last five minutes, each plot receives the selected range, and the view retains that selection across rerenders

#### Scenario: Frozen Report

- **WHEN** a report is captured after a live viewer selects a window
- **THEN** the report includes the full captured sample range independently of the viewer selection, renders once and offers no live refresh controls

#### Scenario: Clean Contract Break

- **WHEN** plot definitions and clients are serialized or generated
- **THEN** no authored window field or compatibility fallback remains, the storage baseline stays v1, and existing storage is not reset

#### Scenario: Multi-Day History

- **WHEN** All is selected for a long-running plot
- **THEN** the existing bounded preview and binning paths remain in use, and any path loading unbounded raw history is identified without adding limits

### Requirement: View Refresh Ownership

Live viewers SHALL offer Refresh to the left of the window slider in every window mode. One refresh SHALL request every plot in the view, show progress with the button disabled until completion, display the last successful refresh as Updated hh:mm:ss and surface failures through existing notifications.

#### Scenario: Manual All

- **WHEN** All is entered and runtime, visibility, zoom or document updates subsequently arrive
- **THEN** one snapshot batch loads on entry, no watch or automatic full-history prefetch runs, and each subsequent button press requests exactly one further snapshot batch

#### Scenario: Timed Refresh

- **WHEN** a timed window is selected and Refresh is pressed
- **THEN** the existing automatic watch behavior continues and every plot receives an immediate fresh initial snapshot

#### Scenario: Refresh Completion

- **WHEN** a refresh is pending, succeeds or fails
- **THEN** pending work disables the button with progress, success advances the completion time, and failure notifies the operator and permits retry without advancing that time
