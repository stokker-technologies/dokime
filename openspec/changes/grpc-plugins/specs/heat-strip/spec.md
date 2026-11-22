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

#### Scenario: Disjoint Facet Categories
- **WHEN** one facet contains categories 1 and 6 and another contains 36, 40 and 44
- **THEN** the first row has two equal-width cells and the second has three
- **AND** both rows span the plot width without padding with the other facet's categories

#### Scenario: Missing Values and Shared Colours
- **WHEN** an observed category has no finite value and other facets contain equal finite values
- **THEN** that category remains uncoloured and equal values use the same colour across facets
