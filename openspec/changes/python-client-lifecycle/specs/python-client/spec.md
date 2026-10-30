## ADDED Requirements

### Requirement: Owned Cycle Lifecycle
The client SHALL expose `new_cycle(label, **options)` as an async context.
#### Scenario: Any Context Exit
- **WHEN** normal completion, an exception or cancellation exits the context
- **THEN** owned work stops, the cycle seals and final outcomes drain
- **AND** cleanup failures remain grouped with any original failure

### Requirement: Template Start And Awaitable Execution
The cycle SHALL resolve, materialize, append and start fresh executions.
#### Scenario: Single Case Or Expansion
- **WHEN** a template materializes one or several cases
- **THEN** start returns one handle or an ordered tuple of all handles
- **AND** expanded cases are admitted in order to the server queue
- **AND** each handle is awaitable and retains its saved outcome

### Requirement: Terminal Evidence Iterators
Executions SHALL expose positional record/stream filters and JSON serialization.
#### Scenario: Execution Seals
- **WHEN** records or samples follow an execution
- **THEN** they drain final evidence and terminate at its saved seal

### Requirement: Outcome Exit Codes
Outcomes SHALL retain status and result and expose conventional exit codes.
#### Scenario: Terminal Verdict
- **WHEN** the outcome is passed, failed/errored or interrupted/cancelled
- **THEN** its exit code is 0, 1 or 130 respectively

### Requirement: Saved Cycle Files
Cycles SHALL save one execution ZIP per member, cycle.zip and optional report.html.
#### Scenario: Save Completes
- **WHEN** a sealed cycle is saved to a directory
- **THEN** manifest.json and the returned manifest contain each file's relative path, size and SHA-256
- **AND** no local JSONL capture is offered

### Requirement: Clean Consumer Migration
The repository SHALL remove callback execution.run and obsolete composition.
#### Scenario: Repository Usage
- **WHEN** examples, documentation and Python clients use executions
- **THEN** they use starts, awaitable handles and terminal async iterators
