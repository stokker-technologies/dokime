# query-selection Specification

## Purpose
Define the small typed predicate language over declared fields, its schema validation and UTC time windows.

## Requirements

### Requirement: Small Typed Predicate Language
New evidence queries SHALL accept a flat conjunction of typed declared-field predicates: `eq`, `lt`, `lte`, `gt`, `gte`, `in` and `exists`. Values SHALL be bound parameters and identifiers allowlisted. An empty conjunction SHALL match all rows. OR/NOT trees, regex, functions, expressions, joins, user code and SQL SHALL be rejected.

#### Scenario: Predicate Operand Validation
- **WHEN** a query contains equality/range, IN and exists predicates
- **THEN** equality/range requires one typed scalar, IN a nonempty bounded compatible list, and exists one boolean
- **AND** two comparisons express a range and IN expresses alternatives for a single field

#### Scenario: Conjunctive Selection
- **WHEN** predicates require `status eq "failed"`, `voltage gte 10` and `voltage lt 12`
- **THEN** only rows satisfying all three are returned
- **AND** a predicate containing a SQL fragment or unknown operator fails before reading evidence

### Requirement: Declared Fields And Exact Types
Record predicates SHALL address captured `details` fields; samples SHALL address captured stream column IDs. IDs SHALL be literal, including dots. Validation SHALL precede scanning, even on empty data. Numeric comparisons SHALL preserve mathematical value without lossy integer-to-float conversion or string/boolean coercion.

#### Scenario: Envelope And Value Types
- **WHEN** a caller selects envelope fields or nonnumeric values
- **THEN** only `@id`, `@timestamp`, `@received_at`, `@source`, `@schema_version`, and records' `@severity` are reserved envelope IDs, using declared string/time/enum types
- **AND** strings compare case-sensitively with locale-independent ordering; booleans permit eq/IN/exists and arrays/objects permit only exists
- **AND** nested traversal, array indexing, undeclared extras and non-finite numeric operands fail even if ingestion allows extras

#### Scenario: Dotted Column And Large Integer
- **WHEN** the contract declares `cpu.load` and an unsigned counter above 2^53
- **THEN** `cpu.load` selects that exact ID and counter equality preserves integer distinctions
- **AND** `"1"` as a numeric operand fails with a typed mismatch

### Requirement: Missing And Null Semantics
`exists(true)` SHALL mean the field is present and non-null; `exists(false)` SHALL match absent or null values. Missing/null fields SHALL not match equality, IN or ordered comparisons. Null operands SHALL be rejected with guidance to use exists. Ingestion defaults SHALL NOT be applied again during querying. Raw results SHALL preserve absent versus explicit null values where the captured contract distinguishes them.

#### Scenario: Optional Measurement
- **WHEN** one record omits a field, another stores null and another stores zero
- **THEN** exists(false) returns the first two and equality to zero returns only the third

### Requirement: Captured Schema Selection
Records SHALL select one concrete type and execution/cycle scope; samples one concrete stream and execution. Optional versions SHALL resolve from contracts at the snapshot. Omitted versions SHALL include compatible captured versions; incompatible types/units SHALL fail with available versions and narrowing guidance. Replies SHALL retain typed schema and attribution.

#### Scenario: Compatible Versions With No Matches
- **WHEN** selected compatible versions have optional missing fields or no matching rows
- **THEN** absent fields are nullable and a known contract returns a typed empty result; an unknown contract fails
- **AND** replies carry schema identity/version, scalar types, nullability and units, while raw rows retain original versions and execution/cycle attribution

#### Scenario: Plugin Updated Since Capture
- **WHEN** an old execution is queried after its plugin changes a field's type
- **THEN** predicates and returned values use captured contracts, not the new plugin
- **AND** incompatible selected historical versions require explicit narrowing rather than silent casting

### Requirement: Half-Open UTC Windows
New query windows SHALL be `[start, end)` with optional bounds, using authored record time or measured sample time. A bounded window SHALL exclude missing times without receive-time fallback. Existing inclusive list/browse windows SHALL remain unchanged. UTC normalization SHALL preserve stored precision and reject invalid bounds.

#### Scenario: Time Input Normalization
- **WHEN** bounds are offset-aware RFC3339 strings, aware Python datetimes or date-only Python strings
- **THEN** they normalize to UTC and date-only means midnight UTC
- **AND** naive datetimes, invalid precision, reversed/equal bounds fail; omitted windows include missing timestamps
- **AND** explicit predicates on `@received_at` can select capture time separately

#### Scenario: Adjacent Windows And Late Arrival
- **WHEN** adjacent queries use `[t0,t1)` and `[t1,t2)` at the same snapshot
- **THEN** a sample at t1 appears only in the second query
- **AND** late-arriving evidence after the snapshot is absent even if its occurrence time falls inside either window

### Requirement: Cycle Work Status Selection
Cycle selection SHALL accept exact alternatives of declared cycle work statuses. Empty alternatives SHALL impose no status constraint; multiple statuses SHALL be disjunctive and combine conjunctively with phase and other existing filters. Unspecified SHALL fail as invalid selection; over-budget alternatives SHALL use the existing configured resource-limit failure.

#### Scenario: Busy Cycles
- **WHEN** selection requests Running, Paused and Stopping
- **THEN** only matching cycles at the fixed cut are returned
- **AND** phase, archive, time, ordering and continuation constraints still apply

#### Scenario: Invalid Status
- **WHEN** selection includes Unspecified
- **THEN** discovery returns an invalid-selection failure
