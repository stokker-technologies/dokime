# Document Query Evaluation Specification

## Purpose

Provide exact document queries at fixed evidence cuts, with independent cards,
bounded working memory and complete frozen reports.

## Requirements

### Requirement: Static Document Structure

Document structure SHALL cost only the declared definition and SHALL read no
telemetry. Every data binding SHALL resolve through a query at a fixed cut.

#### Scenario: Large Document on a Short Run

- **WHEN** a large document with many plots and summaries opens on a short run
- **THEN** its structure is available without telemetry acquisition
- **AND** estimated row work cannot reject capture

### Requirement: Exact Independently Loading Cards

Cards SHALL load independently. Summary count/min/max/mean/RMS SHALL use maintained
aggregates where applicable, with exact statistics queries otherwise. Median SHALL
remain exact. Existing plots SHALL retain their bounded query path.

#### Scenario: Slow or Oversized Card

- **WHEN** one card is delayed or exceeds its actual output budget
- **THEN** other cards become available independently
- **AND** the failing card exposes ErrorCard with budget, limit, reached amount and card identity

#### Scenario: Exact Statistics Beyond Former Source Limits

- **WHEN** retained evidence exceeds former source-row and source-byte capture limits
- **THEN** document summary queries complete with bounded working memory
- **AND** statistics match exact values independently computed from the raw rows

### Requirement: Output Based Limits and Paging

Limits SHALL bound real encoded payload and actually expanded sections, with
paging. Source row counts and estimated evaluation work SHALL NOT reject a
document. Repeated sections SHALL page an index of every distinct selected value.

#### Scenario: Distinct Value in Interior History

- **WHEN** a repeat value occurs only within interior retained evidence
- **THEN** indexed paging includes it once without loading raw history

#### Scenario: Nested Pages During Ingest

- **WHEN** nested repeated sections span pages and later samples arrive
- **THEN** every continuation and its card queries use the original evidence cut and scalar context
- **AND** every selected distinct value and authored child appears across those pages
- **AND** later values are excluded

#### Scenario: Nullable Typed Prefix Within a Saved Batch

- **WHEN** a summary cut lies between aggregate checkpoints
- **THEN** the maintained prefix and bounded remaining values match raw-row statistics
- **AND** nulls, missing fields and integers beyond f64 precision preserve their semantics

#### Scenario: Nullable Aliases Across Schemas

- **WHEN** a declared canonical field is null in one schema and absent in another
- **THEN** indexed repeats and filters preserve canonical-field precedence, null predicates and alias fallback at the captured cut
- **AND** a nested null remains distinguishable from a missing detail

### Requirement: Linear Record Evaluation

TimeBetween and PerCase SHALL use sorted merge or single partition passes rather
than nested scans over the full record population.

#### Scenario: Correlated Records Across Cases

- **WHEN** records from multiple cases interleave
- **THEN** linear evaluation preserves correlation keys, ordering and case boundaries

### Requirement: Shared Frozen Evaluation

Frozen reports SHALL use the same query evaluation at their sealed cut, retaining
all authored table rows and repeated sections. Superseded raw acquisition SHALL
be removed. Development formats MAY change at v1 without migration or reset.

#### Scenario: Sealed Report Parity

- **WHEN** a document is sealed and rendered after later evidence arrives
- **THEN** its complete report matches the document queries at the sealed cut
- **AND** later evidence is excluded

### Requirement: Representative Scaling Benchmark

The existing bench pipeline SHALL measure structure latency, card-fill latency
and peak memory for three days, several million samples and 30 plots at increasing
row counts. Working memory SHALL remain flat as retained rows grow.

#### Scenario: Increasing History

- **WHEN** the benchmark compares equal visible documents over increasing history
- **THEN** it records structure/card-fill times and peak memory without timing assertions
