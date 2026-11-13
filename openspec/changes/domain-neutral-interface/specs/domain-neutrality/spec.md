## ADDED Requirements

### Requirement: No Plugin Domain in the Product

The interface and runtime SHALL use plugin-declared choices, traceability fields and field metadata rather than domain-specific input controls, fixed traceability fields or inferred units and stream names. Plot kind and grid neutrality requirements belong to plot-kinds.

#### Scenario: Grouped Choice

- **WHEN** a parameter declares grouped options supplied by its plugin
- **THEN** the editor shows them in their groups without a control written for that parameter

#### Scenario: Declared Units

- **WHEN** a field declares a unit
- **THEN** the interface shows that unit, and shows none for a field that declares none

#### Scenario: Declared Traceability

- **WHEN** a plugin contributes traceability fields and groups
- **THEN** the interface shows those declarations without fixed host-system categories
