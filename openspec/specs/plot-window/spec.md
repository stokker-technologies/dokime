# Plot Window Specification

## Purpose
Keep time-window selection in live viewers while reports cover the full captured evidence range.

## Requirements

### Requirement: Viewer-Owned Time Window

Plot definitions, stored document contracts and report presentation SHALL contain no authored time window. Live viewers SHALL select one window for all plots in the view, defaulting to All with presets of 1 min, 2 min, 5 min, 10 min, 30 min and 1 h.

#### Scenario: Live Selection

- **WHEN** a live view opens and its operator changes the window
- **THEN** its initial query covers all history, each plot receives the selected range, and the view retains that selection across rerenders

#### Scenario: Frozen Report

- **WHEN** a report is captured after a live viewer selects a window
- **THEN** the report includes the full captured sample range independently of the viewer selection

#### Scenario: Multi-Day History

- **WHEN** All is selected for a long-running plot
- **THEN** the existing bounded preview and binning paths remain in use, and any path loading unbounded raw history is identified without adding limits

#### Scenario: Clean Contract Break

- **WHEN** plot definitions and clients are serialized or generated
- **THEN** no authored window field or compatibility fallback remains, the storage baseline stays v1, and existing storage is not reset
