# Plot Display Scale Specification

## Purpose
Convert plotted time-series values into display units while preserving stored evidence and query semantics.

## Requirements

### Requirement: Validate Display Scale
Documents SHALL accept only positive finite scales on numeric time-series y axes, and SHALL identify the plot when refusing a scale. An absent scale SHALL preserve existing serialization and display values.

#### Scenario: Invalid Scale
- **WHEN** a document plot declares zero, a negative value, NaN or infinity, or an unsupported plot kind or axis
- **THEN** document validation refuses it with a diagnostic naming the plot

#### Scenario: Default Scale
- **WHEN** no scale is declared
- **THEN** serialization omits it and presentation uses 1.0

### Requirement: Convert Presentation Only
Live plots and frozen HTML SHALL share conversion of returned y points, axis/cursor/tooltip values, numeric statistics and y reference lines/bands. Count SHALL remain unchanged. All data/query/evidence/export/criterion semantics SHALL retain stored units.

#### Scenario: Millimetre Display
- **WHEN** a numeric time series uses scale 1000 and unit mm
- **THEN** its points, min/max/mean/median/percentiles/std/RMS and y references multiply by 1000, with unchanged count and timestamps

#### Scenario: Stored-Unit Selection
- **WHEN** filters or repeated sections select data for a scaled plot
- **THEN** they use stored values and conversion occurs only after the query
