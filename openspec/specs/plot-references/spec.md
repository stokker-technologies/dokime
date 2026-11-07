# Plot References Specification

## Purpose
Define plot-owned Y-axis reference values, their stored-unit presentation, and validation consistently across Rust/Python authoring, live views and frozen HTML reports.

## Requirements

### Requirement: Plot-Owned Y References
Plots SHALL accept constants and existing data expression handles as reference values through reference_band, reference_tolerance and reference_line. DataExpression SHALL retain its data-only contract.

#### Scenario: Literal and Parameter Bounds
- **WHEN** a plot declares fixed or parameter bounds
- **THEN** live and frozen plots resolve both through one shared presentation path on the Y axis in stored units, applying the plot scale afterward

#### Scenario: Center and Tolerance
- **WHEN** a plot declares a constant or expression center and tolerance
- **THEN** each case uses its own captured parameters to resolve center minus and plus tolerance

#### Scenario: Invalid Values
- **WHEN** literal bounds are reversed or literal tolerance is negative
- **THEN** document validation refuses the declaration and names the plot

#### Scenario: Unresolved Dynamic Values
- **WHEN** a dynamic reference resolves to null, nonfinite values, reversed bounds or negative tolerance
- **THEN** the band is undrawn without changing captured evidence
