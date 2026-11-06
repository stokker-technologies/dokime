## MODIFIED Requirements

### Requirement: Bounded Exact Reduction Plans

Requests SHALL share native projected batch decoding across outputs within working, scan and time budgets. Moments and rank indexes MAY answer only matching scope, selection and frontier. Other percentiles SHALL use bounded exact numeric selection. Accumulators SHALL NOT retain unbounded decoded raw history. Numeric spools SHALL respect caller memory budgets. Deadline or resource exhaustion SHALL fail without approximation or partial results; cancellation SHALL stop selection cooperatively.

#### Scenario: Completed Or Exhausted Reduction

- **WHEN** an exact reduction finishes or exhausts its admitted work
- **THEN** success identifies output types/units, selected row counts, excluded-value counts and snapshot
- **AND** exhaustion returns a typed failure, never current-frontier statistics, partial estimates or preview aggregates

#### Scenario: Older Cut With Newer Numeric Index

- **WHEN** samples arrive after a query snapshot and its mutable numeric summary has advanced
- **THEN** the query computes from the original Arrow cut or a matching retained index
- **AND** it never reads a newer summary as if it belonged to that cut

#### Scenario: Shared Decode And Numeric Spool

- **WHEN** one request selects several statistics or bucket fields
- **THEN** borrowed projected columns feed the existing sequential reducers after version, capture, predicate, window and frontier selection
- **AND** compact finite numeric projections may spill to temporary storage within admitted query budgets, including payload, spill buffering and rank scratch within caller-owned working memory
- **AND** nonempty interior percentiles retain the existing logical nine-pass scan charge even when evidence is decoded once
- **AND** exact percentile bits agree across memory and spill paths without weakening scan, memory or policy deadlines
