# Report Previews Specification

## Purpose
Preserve renderable report plots with bounded incremental evidence projections.

## Requirements

### Requirement: Bounded Plot Rendering

Reports SHALL emit the existing Cartesian frame contract and retain units, labels and context.
Within-cap summary calculations SHALL retain typed integer inputs until result conversion.

#### Scenario: Time Series

- **WHEN** a report includes one or more envelope series, including fewer than four points
- **THEN** the frontend can render their aligned frame with declared units and metadata subtitles

#### Scenario: Large Integer Summary

- **WHEN** a within-cap series contains integers beyond the floating-point safe-integer range
- **THEN** its mean and median are calculated before converting the final result for display

### Requirement: Incremental Plot Previews

Plots SHALL apply filters and scope at ingest, retain exact inputs within configured caps, and maintain bounded time envelopes, paired scatter occupancy grids and per-group counters during execution. Record durations SHALL retain correlation state rather than subtracting downsampled timestamps.

#### Scenario: Owner Plot Kinds

- **WHEN** an execution records XY positions, reconnect start/end records or network channel counts
- **THEN** its report contains bounded paired coordinates, measured durations and expected counts

#### Scenario: Scatter Extents Grow

- **WHEN** scatter inputs exceed the exact cap and expand the coordinate range
- **THEN** the configured grid re-bins existing cells and retains their occupancy

#### Scenario: Frozen Evidence Cut

- **WHEN** a saved preview advances beyond a report's captured evidence cut
- **THEN** the report reconstructs its bounded preview through that cut without including newer evidence

### Requirement: Specific Unavailability

A preview that cannot represent an analysis beyond its exact cap SHALL state one specific reason in plain words.

#### Scenario: Unsupported History Analysis

- **WHEN** an analysis needs history that cannot be summarized by its incremental state
- **THEN** only that plot identifies the unavailable analysis and retains the full evidence link
