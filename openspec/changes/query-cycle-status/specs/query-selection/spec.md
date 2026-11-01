## ADDED Requirements

### Requirement: Cycle Work Status Selection
Cycle selection SHALL accept exact alternatives of declared cycle work statuses. Empty alternatives SHALL impose no status constraint; multiple statuses SHALL be disjunctive and combine conjunctively with phase and other existing filters. Unspecified SHALL fail as invalid selection; over-budget alternatives SHALL use the existing configured resource-limit failure.

#### Scenario: Busy Cycles
- **WHEN** selection requests Running, Paused and Stopping
- **THEN** only matching cycles at the fixed cut are returned
- **AND** phase, archive, time, ordering and continuation constraints still apply

#### Scenario: Invalid Status
- **WHEN** selection includes Unspecified
- **THEN** discovery returns an invalid-selection failure
