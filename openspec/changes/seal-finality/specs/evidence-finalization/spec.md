## ADDED Requirements

### Requirement: Seal Is the Terminal State

Executions and cycles SHALL have one terminal state named sealed. An execution
SHALL become sealed when its run completes or is cancelled. Messages, phases,
capabilities and client surfaces SHALL name that state sealed and SHALL NOT
describe it as ended or finished.

#### Scenario: Execution Completes

- **WHEN** an execution's run completes
- **THEN** the execution is reported as sealed with its outcome

#### Scenario: Refusal Names the State

- **WHEN** an operation requires a terminal execution and receives a running one
- **THEN** its error asks the caller to wait until the execution is sealed

### Requirement: Sealed Executions Are Final

A sealed execution SHALL refuse new notes and new attached files with an
actionable error. Reads of a sealed execution SHALL return the same evidence on
every call.

#### Scenario: Note After Seal

- **WHEN** a client adds a note to a sealed execution
- **THEN** the host refuses it and names the sealed execution

#### Scenario: Note Before Seal

- **WHEN** a client adds a note to a running execution
- **THEN** the note is stored and appears in the execution's evidence

### Requirement: Cycle Seal Materializes Evidence

Sealing a cycle SHALL refuse while any of its executions is running. Otherwise
it SHALL materialize one report covering every execution and one evidence
archive containing that report, and retain both with the sealed cycle.

#### Scenario: Running Execution

- **WHEN** a cycle is sealed while one of its executions is running
- **THEN** the seal is refused and names the running execution

#### Scenario: Sealed Cycle Has Its Evidence

- **WHEN** a cycle whose executions are all sealed is sealed
- **THEN** its report and its evidence archive exist and the archive contains the report

### Requirement: Retained Evidence Is Returned Unchanged

Every download of a sealed cycle's report or evidence archive SHALL return the
bytes retained at seal.

#### Scenario: Repeated Download

- **WHEN** a sealed cycle's evidence archive is downloaded twice
- **THEN** both downloads have the same digest

### Requirement: Seal Progress Is Observable

A cycle SHALL report the Sealing phase while its evidence is materialized, the
Sealed phase when both artifacts are retained, and Seal Failed with the cause
when materialization fails. A failed seal SHALL leave the cycle unsealed and
sealable again.

#### Scenario: Materialization Fails

- **WHEN** report creation fails during a cycle seal
- **THEN** the cycle reports Seal Failed with the cause and can be sealed again
