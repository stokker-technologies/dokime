## ADDED Requirements

### Requirement: No Plugin Domain in the Product

The interface and runtime SHALL contain no stream identifier, field name, unit rule, plot or input control that belongs to one plugin's domain.

#### Scenario: Occupancy as a Grid

- **WHEN** the host plugin declares a grid plot over its Wi-Fi scan stream with its channel cells and bands
- **THEN** the live view and the report show channel occupancy per band for the selected scan, with the active channel highlighted

#### Scenario: Another Plugin Uses the Grid

- **WHEN** a different plugin declares a grid plot over its own stream and cells
- **THEN** it renders with no change to the product

#### Scenario: Grouped Choice

- **WHEN** a parameter declares grouped options supplied by its plugin
- **THEN** the editor shows them in their groups without a control written for that parameter

#### Scenario: Declared Units

- **WHEN** a field declares a unit
- **THEN** the interface shows that unit, and shows none for a field that declares none

### Requirement: Supported Plot Kinds Only

Every plot kind a plugin can declare SHALL be drawn by the interface.

#### Scenario: Declaring a Kind

- **WHEN** an author lists the available plot kinds
- **THEN** each one can be declared and rendered
