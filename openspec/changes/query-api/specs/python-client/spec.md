## ADDED Requirements

### Requirement: Lazy Finite Query Readers
The client SHALL expose the synchronous lazy query factories in `design.md` as single-consumer async readers. Factories SHALL perform no network I/O; consumption SHALL fetch on demand. Omitted filters SHALL be unconstrained within scope. Existing lookups, records/samples, owned contexts, checkpoint and lifecycle semantics SHALL remain unchanged.

#### Scenario: Factory Arguments
- **WHEN** callers build execution/cycle discovery or evidence readers
- **THEN** executions accepts template/cycle/status/since/until/archive/at and cycles accepts phase/since/until/archive/at
- **AND** execution/cycle query_records accepts record_type/where/between/versions/at; cycle queries can narrow to a member execution ID
- **AND** execution telemetry accepts stream/fields/where/between/versions/every/agg/at

#### Scenario: Historical Selection Without Follow
- **WHEN** a caller iterates `client.executions(template="hcl.demo.fraunhofer", since="2026-10-01", status="failed")`
- **THEN** it receives typed execution summaries across matching cycles and ends at the fixed snapshot
- **AND** it does not await terminal execution completion or allocate the whole history

### Requirement: Typed Predicates And Aggregate Facade
Equality maps SHALL be conjunctions; exported `Predicate(field, operator, value)` sequences SHALL express other operators independent of sequence order. Local shape checks SHALL precede transmission. Telemetry SHALL expose awaited `aggregate(field, aggregates)` with existing Aggregate objects and ordered typed results, without raw client.call.

#### Scenario: Aggregate Reader Validation
- **WHEN** a reader starts aggregate consumption
- **THEN** it returns snapshot/schema/counts and canonical-name values including percentile rank such as p95, rejects duplicates and rejects every/bucket settings
- **AND** aggregate/item/page/collect consumption is mutually exclusive; captured-schema/type validation and authorization stay server-owned

#### Scenario: Exact Statistics On A Window
- **WHEN** a caller awaits `execution.telemetry("host.system", between=(t0,t1)).aggregate("cpu.load", [Aggregate.mean(), Aggregate.percentile(95), Aggregate.max()])`
- **THEN** one canonical server request computes those statistics on raw selected samples at one snapshot
- **AND** a mean string or unsupported percentile rank is rejected with readable guidance to the existing Aggregate vocabulary

### Requirement: Typed Rows And Page Control
Readers SHALL expose async items, pages(), snapshot after the first response, and aclose()/async-context cleanup. Concurrent/mixed consumption SHALL fail. Record/Sample decoding SHALL preserve captured types; bucket/history rows SHALL use distinct models. Closing/cancelling SHALL stop further page requests without execution lifecycle calls.

#### Scenario: Page And Row Metadata
- **WHEN** a reader receives a page
- **THEN** it exposes effective limits, schema, rows, snapshot, continuation and complete, preserving original row versions, envelope identity, scalar types, nulls and units
- **AND** at accepts another reader's compatible snapshot or saved Checkpoint without fabricating terminal readiness; configurable page sizes obey server limits

#### Scenario: Reuse A Cut And Close Early
- **WHEN** a caller consumes one record page, passes its snapshot to a telemetry query and closes the record reader
- **THEN** telemetry uses the same evidence cut and the closed reader fetches no further pages
- **AND** the execution continues and server page resources are released

### Requirement: Explicit Bounded Materialization
Readers SHALL provide awaited collect(max_rows=..., max_bytes=...), to_polars(...) and to_pandas(...). Effective row/decoded-byte ceilings SHALL default to client configuration, never unlimited. Failure/incompleteness SHALL propagate atomically without a successful partial collection. Core queries SHALL require neither dataframe library.

#### Scenario: Optional Conversion Dependencies
- **WHEN** a caller invokes conversion with omitted limits or missing libraries
- **THEN** omitted limits use finite client query configuration and imports occur only for that helper
- **AND** missing libraries name optional polars/pandas extras and installation instructions; conversion remains a client cost with no server dependency

#### Scenario: Optional Library And Limit
- **WHEN** a dependency-free installation iterates samples and then requests `to_polars(max_rows=100_000)`
- **THEN** iteration works and conversion explains how to install `dokime-client[polars]`
- **AND** with Polars installed, a selection exceeding either effective materialization ceiling raises a typed limit error without returning a truncated frame

### Requirement: Tabular Fidelity
Frames SHALL preserve row/column order, exact field names, UTC timestamps, nullable scalars and int64/uint64 values without float inference. Empty/all-null columns SHALL use captured types. Unsupported nested values/dtypes SHALL fail with field-specific errors without stringification or silent casts.

#### Scenario: Frame Attribution
- **WHEN** conversion returns a frame
- **THEN** schema/units, versions, projection, snapshot and completeness are available through Pandas DataFrame.attrs["dokime"] or the Polars originating reader's metadata
- **AND** callers encountering unsupported types can narrow fields or use typed iteration

#### Scenario: Empty Frame And Large Counter
- **WHEN** a projection contains nullable integers above 2^53, UTC timestamps and an all-null field, or matches no rows
- **THEN** conversion preserves declared dtypes/order and the reader retains schema/snapshot/unit metadata
- **AND** unsupported dtype conversion fails explicitly instead of losing information
