# Query Evaluation Design

## Agreed Ownership

Domain document contracts describe static authored structure and query results.
Runtime document ownership binds a fixed evidence cut to each card query, reusing
finite-query exact statistics and existing plots. Storage owns incremental numeric
moments and distinct-value indexes, committed with the evidence they describe.
GUI renders structure first and owns independent card request lifetimes and
ErrorCard presentation. Frozen reports resolve every page at one sealed cut.

## Evaluation

Building structure reads definitions/context only. A binding is a finite query;
simple unfiltered numeric folds use maintained moments when the cut matches.
Other statistics, including median, execute exact bounded-memory queries. Card
failures remain local. No definition-size-times-rows or rows-squared admission
estimate remains. Limits measure actual encoded output and expanded sections;
failures identify budget, limit, reached amount and card.

Record TimeBetween uses ordered correlation rather than one scan per record.
PerCase partitions once before folding. Repeated sections enumerate distinct
indexed values through continuation pages instead of replaying raw rows.
Record readers order bounded pages by execution/attempt and durable position;
correlation state retains only the selected predecessor types for the current
attempt. Distinct dictionaries share typed values with durable row postings,
preserving literal field paths, row-local filters and capture order. SQLite sort
scratch spills to disk. Nested repeats traverse actual card output and prune
selectors found to be empty, without enumerating an empty Cartesian product.

Storage keeps typed numeric prefix checkpoints per captured execution, stream and
field at saved batch frontiers. A query selects the prefix at its cut; a partial
batch uses the exact bounded reader. Checkpoints include nullable fields and
retain integer extrema and totals without f64 rounding. Stored-format replacement
retains v1 and requires a fresh root rather than a migration or reset.
Exact document median walks the ordered distinct dictionary with occurrence-index
lookups; the dictionary owns the outer join order, avoiding a dictionary scan per
occurrence and sorting of duplicate-value groups. Both walks use covering indexes;
only the two selected values are decoded. Nullable telemetry roots derive their
presence from each file's saved contract rather than storing a null occurrence
per sample. Alias precedence and null-versus-missing predicates stay exact,
including nested fields and schema changes.

Repeat continuations reuse the persisted finite-query cut/context and authenticated
cursor ownership. Their context contains authored definitions and selected scalar
context, never captured raw rows or an in-memory document registry. Pages resume
actual expanded output. Frozen materialization drains every page at its sealed cut;
GUI card identities remain independent of changing page/evidence frontiers.
Previous Page and Next Page replace visible output at that cut; Latest explicitly
returns to live refresh. Default generated repeat identities hash the canonical
typed value, so equal labels cannot collide. Numeric repeat equality retains the
existing exact cross-type comparison; nonnumeric ordering is deterministic by
type and value. The earliest qualifying capture supplies explicit ordering.

## Card Delivery

Reuse the existing document stream operation for structure and selected-card
queries. Each GUI binding has an independent query lifecycle, so live refreshes
continue while another binding is still loading. Publish structure before the
saved-evidence wait and commit each card after its complete result arrives. A later card or stream failure keeps
already completed cards available. Avoid a persistent capture registry or a second
query scheduler. Cancellation belongs to the existing request/session owner.
Typed selector queries reuse bounded storage readers and scalar query accumulators
so integer values remain lossless; live and frozen contexts compose that same
query owner. Median scratch remains bounded and may spill. Summary reads project
only their selected Arrow columns; decoded buffers are bounded by the card payload
budget. Buffer admission charges encoded and declared decoded bytes, with no
per-source-row object estimate. Document scans retain cancellation, deadlines and bounded buffers without a
source-row admission ceiling. Public finite-query policies retain their limits.

## Replacement and Delivery

Remove raw live acquisition, array summary evaluation in runtime delivery and
superseded preflight estimates as their replacements land. Keep pure authored
definition validation, exact evidence attribution and the existing plot path.
Frozen reports retain every authored table row and distinct repeated section.
Schema changes retain v1 and require an explicitly fresh root when incompatible.

Slices follow the owner's order. Targeted tests and check:rust prove each slice;
GUI tests and prepare:bindings apply when their owners change. Review README
claims and update affected guides with each user-facing change. The final bench
records structure latency, card-fill latency and peak memory for three days,
several million samples and 30 plots at increasing row counts; timing is never a
correctness assertion.
