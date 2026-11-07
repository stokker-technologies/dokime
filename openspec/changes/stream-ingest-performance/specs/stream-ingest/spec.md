# Stream Ingest Specification

## ADDED Requirements

### Requirement: Measured Durable Sidecar Ingest

The benchmark SHALL measure durable scalar, wide, 4 KiB and 64 KiB ingestion through a real native plugin and sidecar transport, reporting payload throughput, process CPU, observed durability operations, lock waits and storage bytes.

#### Scenario: Repeated Busy-Machine Measurement

- **WHEN** the benchmark measures the before and after candidates
- **THEN** repeated scalar/wide and 0/1/64-plot results identify machine provenance and separate unprofiled throughput from inclusive instrumented stage timing
- **AND** results include rows/s, logical numeric MiB/s, CPU by host/sidecar process and per 100,000 rows, and Arrow/catalog/WAL bytes

#### Scenario: Observe Durability Without Changing It

- **WHEN** a native fixture reaches its durable barrier
- **THEN** the benchmark reports observed host Arrow file/Unix parent-directory and SQLite VFS sync requests, telemetry writer calls and explicit catalog commits inside that interval, preserving the existing cadence and durability points

#### Scenario: Measure Payload and Contention

- **WHEN** the benchmark measures 4 KiB and 64 KiB rows on a busy machine
- **THEN** it verifies durable payload lengths, separates host and live sidecar CPU, records admission/SQLite/store/tail lock waits and distinguishes inclusive active-stage timing from elapsed waits
- **AND** payload MiB/s uses bytes reread from durable Arrow batches rather than a numeric-row-rate multiplier

### Requirement: Shared Bounded Plot Projections

Eligible plots of the same stream field SHALL read one shared bounded projection, preserving an exact typed input prefix within the configured cap, and raw rows SHALL remain unchanged. Static plot definitions SHALL be stored once, and only dirty compact state SHALL be checkpointed. Filters, correlations and analyses SHALL retain their correctness and exactness refusals.

#### Scenario: Many Plots of One Field

- **WHEN** 64 time-series plots select one field over a multi-day kHz stream
- **THEN** ingestion and checkpoint bytes scale with the field and bounded state rather than rows multiplied by plots

### Requirement: Bounded Numeric Indexing

Numeric ingestion SHALL keep bounded state per field. Exact percentile results SHALL remain available through saved-evidence replay, without a historical rank trie written for every value. Ordering, sample identity, scope visibility, durable evidence cuts and admission memory bounds SHALL be preserved.

#### Scenario: Long Exact Percentile

- **WHEN** a report requests a percentile beyond the retained exact-input cap
- **THEN** the result is computed exactly from its saved raw evidence cut with bounded memory
