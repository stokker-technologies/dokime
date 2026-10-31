## MODIFIED Requirements

### Requirement: Isolated Parallel Workers
The runners SHALL parallelize isolated tests while preserving resource ownership.

#### Scenario: Shared Inputs And Exclusive Desktop
- **WHEN** concurrent lanes consume wheels, generated contracts and fixture artifacts
- **THEN** preparation finishes before readers start and no lane rewrites those inputs
- **AND** independent native desktop workflows run concurrently with their own roots and ports inside one exclusively owned desktop lane

#### Scenario: Standalone Lanes
- **WHEN** an individual lane task is selected
- **THEN** it remains independently usable with its normal preparation and reports

## ADDED Requirements

### Requirement: Event-Driven Test Progress
Dokime fixtures SHALL use short cadences from shared configuration and plugin
settings, await observable conditions, and preserve product correctness policies.

#### Scenario: Progress Is Available
- **WHEN** the product already exposes a progress signal
- **THEN** consumers observe it without waiting for a polling interval to expire
- **AND** configured intervals remain upper bounds where periodic work is needed

#### Scenario: Comparable Evidence
- **WHEN** the integration and acceptance lanes are measured before and after
- **THEN** every baseline story over ten seconds has an attributed wait and comparison
- **AND** remaining Harmos costs are reported without modifying Harmos
