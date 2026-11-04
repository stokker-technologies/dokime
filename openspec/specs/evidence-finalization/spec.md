# evidence-finalization Specification

## Purpose
Define bounded evidence finalization, joined drain ownership and independent durable report publication while retaining complete raw evidence.

## Requirements

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
exact summaries and complete raw evidence retained separately. Plots SHALL use first/last/min/max/count time envelopes persisted during ingestion
in the same transaction as raw samples. Tables SHALL expose bounded first/last
rows and identify full evidence. Limits SHALL come from configuration. Unsupported
history-dependent analyses SHALL be explicitly unavailable without scanning.

#### Scenario: Saved Report At Four Times The History

- **WHEN** an execution report is materialized and saved at one-hour and four-hour volume
- **THEN** the work scales with bounded report size rather than raw history
- **AND** documents, plots, summaries and evidence attribution remain consistent

### Requirement: Independent Report Publication

Stop, Cancel and shutdown SHALL finish after execution closure and evidence drain,
without joining HTML materialization. A report failure SHALL NOT fail Stop or
change the execution result. Durable report state SHALL be pending, ready or failed
and observable through GUI, CLI and Python. Report downloads and cycle saves
SHALL await readiness or expose the publication failure.

#### Scenario: Paused Renderer

- **WHEN** report rendering is paused after evidence drains
- **THEN** Stop returns while the report is pending and downloads wait
- **AND** sample follows can drain the closed evidence frontier before HTML is ready
- **AND** successful publication advances to ready or a render error advances to failed
- **AND** repeated Stop still succeeds after a report failure

#### Scenario: Shutdown With Pending Publication

- **WHEN** shutdown reaches a pending report
- **THEN** it preserves the durable intent without waiting for HTML
- **AND** startup resumes publication without replaying provider work
