## ADDED Requirements

### Requirement: Bounded Finalization Work

The runtime SHALL persist raw evidence and report reductions incrementally with
bounded memory. Stop and Cancel SHALL seal only a bounded pending tail, without
changing existing operation or shutdown deadlines.

#### Scenario: Long Execution Stops

- **WHEN** otherwise identical one-hour and four-hour executions stop
- **THEN** Stop-to-seal and Cancel-to-settled work is independent of saved history
- **AND** all acknowledged evidence remains durable and available

### Requirement: Joined Drain Ownership

Each execution-end drain SHALL have one owner. Concurrent callers and retries
SHALL await the original in-flight drain, including after a waiter is cancelled.
Completed stages SHALL NOT execute again.

#### Scenario: Stop Overlaps Shutdown

- **WHEN** shutdown or retry enters while execution Stop is draining
- **THEN** both await the same work and each physical drain runs once
- **AND** failure preserves unfinished ownership for safe retry

### Requirement: Bounded Report Materialization

Reports SHALL materialize from incrementally maintained, bounded inputs with
exact summaries and complete raw evidence retained separately. Existing
full-history document semantics require the recorded owner decision before cutover.

#### Scenario: Saved Report At Four Times The History

- **WHEN** an execution report is materialized and saved at one-hour and four-hour volume
- **THEN** the work scales with bounded report size rather than raw history
- **AND** documents, plots, summaries and evidence attribution remain consistent
