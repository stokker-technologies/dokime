## ADDED Requirements

### Requirement: Cycle Work At The Snapshot
Cycle query rows SHALL expose the declared cycle work status and running execution count from the query cut. Status SHALL be captured from the cycle revision; running count SHALL count only Running member executions at that same cut. Existing phase and paging semantics SHALL remain unchanged.

#### Scenario: Work Changes After Capture
- **WHEN** cycle lifecycle or execution status changes after the first page
- **THEN** continuation and snapshot reuse retain the captured status and running count
- **AND** a fresh query observes the newer work state

#### Scenario: Native And Remote Discovery
- **WHEN** native and remote callers query the same snapshot
- **THEN** their cycle status and running count agree

#### Scenario: Admitted Work Before A Running Checkpoint
- **WHEN** the admitted cycle header is Running and the durable execution revision is still Pending
- **THEN** discovery reports Running with a zero running count
- **AND** a fresh query after a Running execution checkpoint counts that revision while the earlier snapshot remains unchanged
