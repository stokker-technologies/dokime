## ADDED Requirements

### Requirement: Exact Coarser Preview Resolutions

The existing preview owner SHALL maintain coarser time-bucket resolutions for wide plot windows. Reads SHALL choose covered summaries at the requested cut, with exact partial/uncovered edges. Late samples SHALL update affected resolutions without exposing newer evidence at older cuts. Explicit telemetry every intervals SHALL retain their requested resolution and exact reducers.

#### Scenario: Wide Window

- **WHEN** a multi-day plot window needs panel-width detail
- **THEN** the query reads a suitable coarser preview instead of decoding every base-resolution bucket and preserves the defined first/last/min/max and moments

#### Scenario: Late Sample and Older Cut

- **WHEN** a late sample changes an interior bucket after a snapshot cut
- **THEN** a fresh query includes it at every applicable resolution and a query at the older cut excludes it using valid summaries or exact evidence reads

#### Scenario: Partial Edges

- **WHEN** a window clips summary buckets or newer evidence is not yet covered
- **THEN** exact edge/uncovered reads produce the same defined reductions as evidence at that cut

### Requirement: Bounded Fallback Frame Reduction

Fallback plot projections SHALL reduce incrementally without collecting the full selected point population. Peak working memory SHALL be bounded by the frame and admitted decode/reducer/spill buffers, independently of selected rows. Exact operations SHALL retain exactness through bounded spill or fail explicitly on resource exhaustion, never approximate or return partial results.

#### Scenario: Growing Selection at Fixed Frame

- **WHEN** selected rows increase from millions upward with fixed output frame and working budgets
- **THEN** reduction peak memory is bounded by the frame and working buffers rather than selection size, while results match the specified reduction at the captured cut

#### Scenario: Exact Reduction Exhausts Resources

- **WHEN** exact fallback reduction exhausts admitted memory, spill or work
- **THEN** it returns a typed failure without a partial or approximate statistic and cancellation releases owned resources
