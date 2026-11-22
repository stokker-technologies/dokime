## ADDED Requirements

### Requirement: Running Execution Rendering

Running execution watches SHALL project Heat Strip columns as cells through the
same projection and renderer used by frozen reports.

#### Scenario: Rows Arrive During Execution
- **WHEN** a running execution watch delivers an initial Heat Strip snapshot and a replacement after rows arrive
- **THEN** the live document draws the initial cells and updates their values from the replacement
- **AND** neither frame is misclassified as numeric Cartesian coordinates

## MODIFIED Requirements

### Requirement: Ordered Shared Presentation

Heat Strip SHALL fill the available plot width with equal-width labelled cells
in each facet row. Each row SHALL contain only its observed categories in numeric
order for numeric identities and lexical order for text identities, preserving
exact typed identities. All rows SHALL share a colour scale, legend, minimum
cell height and wrapping labels on live and report surfaces. Absent values SHALL
remain uncoloured. Unit labels SHALL use `Name [unit]`.

#### Scenario: Facets and Empty Slots
- **WHEN** one facet contains categories 1 and 6 and another contains 36, 40 and 44
- **THEN** the first row has two equal-width cells and the second has three
- **AND** both rows span the plot width without padding with the other facet's categories

#### Scenario: Constant or Empty Values
- **WHEN** an observed category has no finite value and other facets contain equal finite values
- **THEN** that category remains uncoloured and equal values use the same colour across facets

#### Scenario: Numeric and Textual Ordering
- **WHEN** numeric categories arrive as 10, 2, 1 or text categories as c, a, b
- **THEN** each facet orders them as 1, 2, 10 or a, b, c without losing exact integer identities

#### Scenario: Mixed Numeric Cell Ordering
- **WHEN** cells include signed, unsigned, floating-point and exact fractional identities
- **THEN** they sort numerically without rounding, with deterministic typed ties and text following lexically

### Requirement: Per-Cell Reduction

Heat Strip SHALL select values at a source sample instant rather than reduce
the whole window. Multiple finite contributions to the same typed CELL and
FACET at that instant SHALL use the declared reducer, defaulting to LAST with
durable position breaking ties. Missing or nonfinite values SHALL remain empty
and SHALL NOT carry forward from another instant. Hover SHALL expose the cell,
optional facet, selected value and source instant.

#### Scenario: Late and Tied Samples
- **WHEN** a cell has values 3 and 9 at different sample instants and the first is selected
- **THEN** the cell shows 3 and its source instant, regardless of the later value
- **AND** a cell with no value at the selected instant remains uncoloured

#### Scenario: Every Reducer and Absence
- **WHEN** a cell has values 4 and 6 at one sample instant
- **THEN** LAST is 6 by durable order, MAX is 6, MEAN is 5 and COUNT is 2 at that instant; an absent cell remains empty

#### Scenario: Hover Provenance
- **WHEN** a populated cell is inspected
- **THEN** hover exposes its cell, optional facet, selected value and sample instant

### Requirement: Window and Frozen-Cut Parity

Heat Strip SHALL use the same renderer and sampled data model in live views and
reports. A full-width slider SHALL step by sample index and show the selected
source time. Live views SHALL follow the newest sample until seeking holds the
loaded selection; Latest SHALL return to incoming samples. Frozen reports SHALL
default to the last captured sample and allow seeking only within their fixed cut.

#### Scenario: Follow and Cursor Selection
- **WHEN** samples arrive, the operator selects an earlier sample and newer rows arrive
- **THEN** the initial view follows the newest sample and the held selection stays fixed after seeking
- **AND** Latest returns to the newest incoming sample

#### Scenario: Sealed Report
- **WHEN** a report captures two samples and the operator moves its slider back one step
- **THEN** it initially shows the last captured value and then shows the earlier captured value
- **AND** later execution rows cannot change either captured value

### Requirement: Bounded Grouped Selection

Each Heat Strip SHALL obtain all retained samples, cells and facets through one
grouped query with indexed source reads and bounded query/output state. Scope,
filters, selected windows and fixed cuts SHALL apply before grouping. Exceeding
the sampled-cell or memory budget SHALL refuse the selection with an action to
narrow the window or select fewer samples, without omitting samples silently.

#### Scenario: Budget and Isolation
- **WHEN** retained sampled cells exceed the output budget or evidence lies outside scope, filters or cut
- **THEN** excess cardinality is refused explicitly and excluded evidence never contributes

#### Scenario: Multi-Day Selection
- **WHEN** a multi-day selection's sampled cells fit the query and output budgets
- **THEN** one grouped query returns every selected sample without per-cell queries; selections exceeding those budgets are explicitly refused
