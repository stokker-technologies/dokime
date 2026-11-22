## ADDED Requirements

### Requirement: Running Execution Rendering

Running execution watches SHALL project Heat Strip columns as cells through the
same projection and renderer used by frozen reports.

#### Scenario: Rows Arrive During Execution
- **WHEN** a running execution watch delivers an initial Heat Strip snapshot and a replacement after rows arrive
- **THEN** the live document draws the initial cells and updates their values from the replacement
- **AND** neither frame is misclassified as numeric Cartesian coordinates
