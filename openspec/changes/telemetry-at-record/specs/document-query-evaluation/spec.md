# Document Query Evaluation Spec Delta (Proposed)

## ADDED Requirements

### Requirement: Telemetry Sampled at Records

A telemetry expression SHALL accept an `at` transform naming a record type. For each record of that type the expression SHALL yield the stream value of the row with the greatest timestamp at or before the record's timestamp, or, when a tolerance is given, the nearest row within the tolerance with ties resolved to the earlier row. A record with no qualifying row SHALL yield no value. The result SHALL compose with aggregates, record details and per-case grouping like a per-record series.

#### Scenario: Value at Each Event

- **WHEN** a histogram's X selects `telemetry(stream, field).at(record_type)`
- **THEN** it receives one value per record, each the stream's last value at or before that record

#### Scenario: Tolerance

- **WHEN** the expression adds `within_s(t)` and a record has rows on both sides within `t`
- **THEN** the nearest row's value is used, the earlier one on a tie, and a record with no row within `t` yields no value

#### Scenario: Repeat Selector

- **WHEN** a repeated section's value, order or filter uses `at`
- **THEN** validating the document fails with an error naming that section

#### Scenario: Non-Telemetry Source

- **WHEN** `at` is applied to a record, parameter, dimension or artifact expression
- **THEN** validation fails naming the expression

### Requirement: Linear Sampled Evaluation

Evaluating `at` SHALL merge the stream rows and the records in one ordered pass with memory bounded independently of the row count, for live and sealed evidence alike.

#### Scenario: Sealed Execution

- **WHEN** a sealed execution's document evaluates `at` over a million-row stream
- **THEN** the evaluation reads the rows once and the result equals the live evaluation of the same evidence
