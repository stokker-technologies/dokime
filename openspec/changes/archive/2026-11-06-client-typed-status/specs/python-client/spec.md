## ADDED Requirements

### Requirement: Typed Work Status
The client SHALL export `CycleStatus` and `ExecutionStatus` string enums mirroring all server statuses with stable lowercase values. Summaries SHALL expose those enums, and cycle summaries SHALL expose `running_count`. Named status filters SHALL accept one enum or a sequence; existing string filters SHALL remain supported. Protocol representation SHALL remain private.

#### Scenario: Inspect Busy Work
- **WHEN** a caller iterates cycles filtered by Running, Paused and Stopping
- **THEN** requests select those statuses and summaries retain typed status and running count without lifecycle calls

#### Scenario: Decode Statuses
- **WHEN** any declared cycle or execution status is returned
- **THEN** the summary exposes the corresponding exported enum, including Unspecified
- **AND** unsupported status replies fail with a client-facing protocol error

### Requirement: Typed Record Declarations
`client.record_schemas()` SHALL lazily yield immutable record schemas with record type, version and ordered declared fields. Fields SHALL expose identifier, typed value type, optional unit symbol/label and required/nullable flags. Discovery SHALL page existing catalog operations and reject invalid continuations, revisions and definition identities.

#### Scenario: Discover Declared CSV Columns
- **WHEN** a consumer iterates record schemas across several catalog pages
- **THEN** requests are made on demand and fields retain declared order, literal identifiers, types, labels and units without record-value inference

#### Scenario: Catalog Changes
- **WHEN** the catalog revision or listed definition identity changes during paging
- **THEN** discovery raises a protocol error rather than returning a successful mixed catalog
