# Document Query Evaluation

## Why

Live documents acquire raw history and reject estimated evaluation work, so even
short runs can fail as authored documents grow. Multi-day runs need presentation
cost governed by visible output rather than retained sample count.

## What Changes

The owner has agreed this design and four independently delivered slices:

1. Static structure without telemetry reads; query-backed summaries/statistics;
   actual output limits replacing row estimates and the source-row ceiling.
2. Independent card loading in the runtime contract and GUI, using ErrorCard.
3. Linear record correlations and per-case evaluation, indexed paged distinct
   repeats, and ingest-maintained count/min/max/mean/RMS. Median remains exact.
4. Frozen reports use the same query path with complete tables/repeats; remove
   superseded acquisition and add the three-day, millions-of-samples benchmark.

## Impact

Runtime documents, query/storage owners, domain and operation contracts, generated
clients, GUI/report rendering, deterministic tests, benchmark wiring and the
product/GUI/query guides change together. Existing plot checkpoints remain.
Development storage may break cleanly at v1, without migrations or data resets.
Each slice receives focused proof and check:rust, then an immediately squash-merged
PR to dev. The owner explicitly excludes full simulation for this delivery.
