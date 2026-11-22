# heat-strip Specification

## Purpose
Compare reduced measurements across distinct categories and facets using one
labelled square per category in live documents and frozen execution reports.

## Requirements

### Requirement: Generic Heat Strip Authoring

The product SHALL accept a Heat Strip with one numeric VALUE expression, a required CELL expression, an optional FACET expression and LAST, MAX, MEAN or COUNT reduction. Omitted reduction SHALL mean LAST. Python, Rust, C# and C++ plot authoring SHALL serialize the same declaration.

#### Scenario: Builder Round-Trip
- **WHEN** an author builds a faceted Heat Strip and serializes and decodes its document
- **THEN** kind, VALUE, CELL, FACET, label and explicit reduction remain unchanged
- **AND** an omitted reduction evaluates as LAST

#### Scenario: Invalid Declaration
- **WHEN** CELL is missing, VALUE is not a single numeric expression or reduction is unsupported
- **THEN** the plot is refused with guidance to correct the declaration

### Requirement: Per-Cell Reduction

The product SHALL reduce selected finite VALUE contributions independently per typed CELL and FACET identity. LAST SHALL use latest source time with durable position breaking ties. Missing or nonfinite contributions SHALL NOT become zero, and groups with no contributions SHALL remain empty for every reducer.

#### Scenario: Every Reducer and Absence
- **WHEN** one cell has values 2, 6 and 4 in source-time order and another has only absent values
- **THEN** LAST is 4, MAX is 6, MEAN is 4 and COUNT is 3 for the first cell
- **AND** the second cell remains empty for LAST, MAX, MEAN and COUNT

#### Scenario: Late and Tied Samples
- **WHEN** an earlier-source-time value arrives after a later one and two latest samples share source time
- **THEN** LAST selects the greatest source time and then the greatest durable position

#### Scenario: Hover Provenance
- **WHEN** a populated square is inspected
- **THEN** hover exposes its cell, optional facet, reduced value and contribution time
- **AND** LAST uses its selected sample, MAX its latest tied maximum, and MEAN/COUNT their latest contributor

### Requirement: Ordered Shared Presentation

Heat Strip SHALL display equal labelled squares in numeric CELL order for numeric identities and lexical order for textual identities. Optional facets SHALL form separate strips with one shared CELL order, colour scale and legend. Missing slots SHALL remain uncoloured. Number formatting SHALL use a decimal point without grouping and unit labels SHALL use `Name [unit]`.

#### Scenario: Numeric and Textual Ordering
- **WHEN** numeric cells arrive as 10, 2, 1 or textual cells arrive as c, a, b
- **THEN** their labels appear as 1, 2, 10 or a, b, c respectively
- **AND** distinct integers beyond floating-point exact precision retain their identities

#### Scenario: Mixed Numeric Cell Ordering
- **WHEN** CELL identities include signed integers, unsigned integers, floating-point values and exact fractions
- **THEN** Dokime storage orders them by numeric value, comparing fractions without rounding identities to floating point
- **AND** equal numeric values retain deterministic typed identities and textual cells follow numeric cells lexically

#### Scenario: Facets and Empty Slots
- **WHEN** two facets contain overlapping cell sets and different value ranges
- **THEN** strips use the sorted union of cells and equal square sizes
- **AND** a cell absent in a facet is empty and one legend covers both value ranges

#### Scenario: Constant or Empty Values
- **WHEN** all populated cells have the same value or every observed cell has absent values
- **THEN** constant values render consistently and all-absent cells remain empty without an invented zero

### Requirement: Window and Frozen-Cut Parity

Live Heat Strips SHALL use the viewer-selected window and existing refresh policy. Frozen reports SHALL reduce the whole execution through their captured cut with the declared reducer, defaulting to LAST, and SHALL use the same presentation as live documents.

#### Scenario: Follow and Cursor Selection
- **WHEN** Follow advances the selected window or the operator moves the cursor/window
- **THEN** the snapshot recomputes cells and values within that selected window
- **AND** cells occurring only outside it do not remain from the prior snapshot

#### Scenario: Sealed Report
- **WHEN** a report is captured after a narrow live selection and newer evidence subsequently arrives
- **THEN** it contains the full captured execution's reduced cells at its fixed cut
- **AND** later evidence and the live window do not change it

### Requirement: Bounded Grouped Selection

Each Heat Strip SHALL obtain all selected cells and facets in one grouped query, with bounded accumulator/output state and indexed source reads. Existing filters, scope and fixed cuts SHALL apply before reduction. Exceeding a group/output budget SHALL produce actionable refusal without silently omitting cells; execution duration alone SHALL NOT cause refusal.

#### Scenario: Multi-Day Selection
- **WHEN** a multi-day execution has many rows but its distinct cells and facets fit the query budget
- **THEN** one grouped query computes the complete reduction without retaining raw history or querying separately per cell/facet

#### Scenario: Budget and Isolation
- **WHEN** selected group cardinality exceeds the output budget or evidence lies outside scope, filters or cut
- **THEN** excess cardinality is explicitly refused and excluded evidence never contributes
