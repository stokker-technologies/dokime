# System Story Reports

## ADDED Requirements

### Requirement: Shared Final Report Finalization

Every report-producing system story SHALL end by saving its final sealed execution report and reopening it with its seal/checkpoint identity and artifacts. Shared runtime fixture teardown, native process teardown and desktop journey finalization SHALL perform this automatically. Individual stories SHALL NOT add report-saving boilerplate or a separate report-saving story.

#### Scenario: A Story Produces Executions

- **WHEN** a report-producing system story reaches its ordinary teardown
- **THEN** the shared finalizer saves its sealed execution reports
- **AND** the saved HTML reopens with the same execution identity
- **AND** its seal/checkpoint remains unchanged
- **AND** every artifact in the seal appears in the report and reopens at the report's relative link with the expected length and digest

#### Scenario: A Story Has No Execution Report

- **WHEN** a system story intentionally ends without an execution report
- **THEN** the shared exemption inventory records that story and its reason
- **AND** a report-producing story without a saved execution report fails finalization

#### Scenario: A Story Exercises A Stop Boundary

- **WHEN** a shutdown story intentionally stops a running host and recovers it
- **THEN** finalization runs at the recovered session's ordinary teardown
- **AND** it does not seal or render before the intentional stop
- **AND** if a desktop journey has already closed its recovered session, the shared finalizer reopens the same isolated application home and closes that owner after saving its reports
