## ADDED Requirements

### Requirement: Marker Kinds Share The Plot Legend

The plot SHALL list each marker kind once after traces, with its matching glyph,
label and count in the shown x window. Marker statistics SHALL be empty.

#### Scenario: Independent Marker Visibility
- **WHEN** a user toggles a kind's row
- **THEN** only that kind hides and its row remains available to restore it
- **AND** visibility follows the same component ownership as trace visibility

#### Scenario: Prompt Lifecycle And Reports
- **WHEN** a plot includes prompt markers in the live viewer or frozen report
- **THEN** asked and answered kinds appear as legend rows instead of a separate control
- **AND** marks retain hover labels and distinct palette colors in both themes

#### Scenario: No Markers
- **WHEN** a plot has no markers
- **THEN** its existing legend presentation is retained

### Requirement: Sparse Samples Remain Visible

Every finite sample without a connecting segment SHALL have a visible mark in
every line style. Point styles SHALL draw finite coordinate pairs, including
alternating aligned traces, independently of hover.

#### Scenario: Gaps And Optional Coordinates
- **WHEN** aligned traces alternate samples or faceted rows have missing coordinates
- **THEN** valid samples draw while missing coordinate pairs remain gaps

#### Scenario: Window Reduction And Frozen Rendering
- **WHEN** a window is reduced or rendered from frozen evidence
- **THEN** retained samples and gaps remain correctly represented on canvas
