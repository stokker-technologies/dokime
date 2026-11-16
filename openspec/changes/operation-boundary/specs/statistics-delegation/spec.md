## Purpose

Compute all aggregation and statistics through Harmos query transforms while Dokime keeps its product algorithms, presentation and stored zoom envelope, cleared when an execution is sealed.

## ADDED Requirements

### Requirement: Aggregation Through Harmos Transforms

Counts, sums, means, extrema, RMS, standard deviation, percentiles, histograms, box summaries, first and last values and time buckets SHALL come from Harmos row-listing transforms. Dokime SHALL keep no implementation of these kernels.

#### Scenario: Document Percentile

- **WHEN** a document requests the median and 95th percentile of a stream field
- **THEN** Dokime issues one Harmos transform request and formats the returned values

### Requirement: Product Expressions Compile to Query Plans

Dokime expression bindings SHALL compile into Harmos query plans for filtering, projection, scaling, offsetting, containment and aggregation, keeping parameter and dimension binding in Dokime.

#### Scenario: Scaled Expression

- **WHEN** a plot expression multiplies a field by a gain and adds an offset
- **THEN** the computed values come from the Harmos transform and match the expression's definition

### Requirement: Product Algorithms Source Data Through Operations

Sigma, contour, grid and FFT calculations and document and plot presentation SHALL stay in Dokime and SHALL read their inputs only through Harmos operations.

#### Scenario: FFT Plot

- **WHEN** an FFT plot is computed for a stream window
- **THEN** its samples are read through a Harmos row operation and the transform runs in Dokime

### Requirement: Stored Zoom Envelope

Dokime SHALL keep a stored zoom envelope per numeric field, advanced by its product coarsening operation through the Harmos bucket-summary transform from the stored summary and its source cut, and persisted through Harmos object or file operations.

#### Scenario: Envelope Catches Up

- **WHEN** new samples arrive after the stored envelope's source cut
- **THEN** the coarsening operation merges them through Harmos and stores the result with the new cut

### Requirement: Sealing Clears Stored Previews

Sealing an execution SHALL clear its stored envelope and previews, including persisted summaries, occupancy grids and group counters, in the seal's compound mutation after the sealed report is computed.

#### Scenario: Plot After Seal

- **WHEN** a viewer requests a full-range plot of a sealed execution
- **THEN** no stored envelope exists for it and the plot is computed from Harmos bucket summaries over the sealed cut

#### Scenario: Report Before Clearing

- **WHEN** an execution is sealed
- **THEN** its report contains the envelope-based plots computed before the previews were cleared
