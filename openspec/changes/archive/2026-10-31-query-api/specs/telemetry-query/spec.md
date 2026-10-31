## ADDED Requirements

### Requirement: Arrow Projection And Filtering
Telemetry queries SHALL prune Arrow batches through SQLite execution/stream/schema/time indexes and decode needed columns where compression permits. Predicates/windows SHALL precede reductions. Raw results SHALL preserve typed identity, time, sequence, scope, selected values and schema/units. Unsupported layouts SHALL fail without unbounded fallback.

#### Scenario: Server Execution Plan
- **WHEN** the server plans a raw or reduced telemetry selection
- **THEN** omitted fields selects all declared columns within limits, and projection includes predicate/time columns
- **AND** it uses no per-sample RPC, full-history Python decode, JSON round-trip or plot-preview substitution

#### Scenario: Narrow Window And Projection
- **WHEN** a query selects two fields and a short time window from a million-sample execution
- **THEN** catalog time indexes skip irrelevant batches and only admitted projected batches are processed
- **AND** raw selected values agree with the stored contract and original evidence at that cut

### Requirement: Exact Server Statistics
`aggregate_samples` SHALL accept bounded ordered unique alias/field/Harmos Aggregate outputs: count, sum, mean, min, max, median, std, rms and integral percentile ranks 0..100. Arithmetic SHALL follow Harmos f64 projection/population-std semantics; non-finite results SHALL fail. Percentiles SHALL interpolate exactly at `(n-1)*rank/100` without sketches.

#### Scenario: Reduction Types And Empty Data
- **WHEN** outputs reduce a numeric selection containing absent, null or non-finite values
- **THEN** empty-field count counts rows, field count counts finite numbers, other outputs require declared numeric fields, and excluded values are counted in quality metadata without becoming zero
- **AND** empty numeric selections return count=0 and null for other statistics; raw int64/uint64 values stay lossless
- **AND** count is uint64/dimensionless, other numeric outputs are float64 with the input unit; aliases preserve request order

#### Scenario: Small Known Population
- **WHEN** the selected finite values are 0, 10, 20 and 30 plus one null
- **THEN** row count is 5, field count is 4, mean is 15 and p95 is 28.5
- **AND** quality metadata identifies the excluded null and std uses the population denominator

### Requirement: Bounded Exact Reduction Plans
Requests SHALL share selection/scans across outputs within working/scan/time budgets. Moments/rank indexes MAY answer only matching scope, selection and frontier. Other percentiles SHALL use bounded exact selection, charging every pass to work budgets. No accumulator SHALL retain every sample; exhaustion SHALL fail without approximation.

#### Scenario: Completed Or Exhausted Reduction
- **WHEN** an exact reduction finishes or exhausts its admitted work
- **THEN** success identifies output types/units, selected row counts, excluded-value counts and snapshot
- **AND** exhaustion returns a typed failure, never current-frontier statistics, partial estimates or preview aggregates

#### Scenario: Older Cut With Newer Numeric Index
- **WHEN** samples arrive after a query snapshot and its mutable numeric summary has advanced
- **THEN** the query computes from the original Arrow cut or a matching retained index
- **AND** it never reads a newer summary as if it belonged to that cut

### Requirement: Explicit Time Buckets
`every` SHALL select fixed positive duration buckets anchored at Unix epoch UTC using floor(timestamp/every), with `agg` required. Numeric bucket reducers SHALL be count/sum/mean/min/max. Windows SHALL clip inputs, not labeled bucket bounds. Buckets SHALL reduce each execution/stream/schema-compatible partition separately and omit empty buckets.

#### Scenario: Bucket Request And Output Contract
- **WHEN** a caller requests buckets or supplies agg without every
- **THEN** agg without every fails; percentile/median/std/rms require unbucketed aggregate_samples
- **AND** buckets use no calendar/DST alignment, cross-execution/source grouping, interpolation or gap filling
- **AND** a populated bucket without finite values returns field count zero and null reduction (count zero); typed replies identify bounds, sample/field counts, units and reducer, not authored sample IDs

#### Scenario: Boundary And Missing Interval
- **WHEN** every is one second and samples occur at 00:00:00.999, 00:00:01.000 and 00:00:03.000 UTC
- **THEN** they populate buckets starting at seconds 0, 1 and 3 and no second-2 bucket is invented
- **AND** paging preserves complete buckets and ascending bucket order

### Requirement: Bounded Bucket State
The planner SHALL bound selected partitions, active bucket accumulators and output rows/bytes from configuration. Late/out-of-order timestamps SHALL yield the same result as ordered input at the same snapshot. It MAY rescan bounded windows or retain a bounded bucket map, but SHALL NOT assume time-ordered ingestion or silently coarsen the interval. Excessive bucket cardinality SHALL fail with guidance to narrow the window or increase every.

#### Scenario: Excessive Resolution
- **WHEN** a multi-day selection at millisecond resolution exceeds configured bucket state
- **THEN** it fails with a typed bucket-limit error without returning coarser or incomplete data

### Requirement: Measurable Interactive Target
The engineering target SHALL be approximately one second for a warm local scan/reduction over 1,000,000 samples under the fixture below. Native Windows/Linux measurements SHALL report latency, memory and work. This target SHALL NOT be a universal SLA or justify changing deadlines; check-only specification proof SHALL establish no performance claim.

#### Scenario: Reference Measurement Workload
- **WHEN** the future implementation measures the target
- **THEN** it uses one execution/stream/schema, UTC timestamp, two float64 fields, one numeric comparison and one-second mean buckets within output limits; shared mean/max/count statistics are measured separately
- **AND** native release builds use recorded Windows/Linux machines, local SSD, fixed fixture/seed, warmup and repeated trials, reporting median/p95 latency, peak working memory, decoded/scanned rows/bytes and output size
- **AND** server time includes planning/decode/filter/reduction/encoding, excludes fixture creation/transport/dataframes, and cold-cache/percentile costs are reported separately; no benchmark runs in this spec-only change

#### Scenario: Performance Evaluation
- **WHEN** the implementation is evaluated on both reference platforms
- **THEN** results demonstrate scaling from 100,000 to 1,000,000 samples and bounded memory at fixed budgets
- **AND** any missed target is reported with measured causes rather than claimed as proven by check-only validation
