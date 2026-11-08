# Ingestion Storage Redesign Tasks

Status: proposed; deferred until the owner schedules it. Targets are ranges
for comparison, not budgets.

## 1. Contract

- [ ] 1.1 Confirm the in-flight write-path fixes are on `dev`: schema revision cache, decoded admission reuse, batched sample inserts, attempt-log appender and batched seal finalization; record their bench results as the baseline.
- [ ] 1.2 Resolve the open questions in design.md with the owner and record the decisions; lock the source inventory before editing.

## 2. Journal and Group Commit

- [ ] 2.1 Add per-execution journal segments with checksummed records, grouped `sync_data` and acknowledgement after the covering sync; prove with failure-injection tests that a failed sync acknowledges none of its batches, and with `storage/ingestion/samples` toward 0.2 to 1M samples/s per ingestion core from the recorded baseline.
- [ ] 2.2 Add replay with torn-tail truncation; prove that every acknowledged batch replays exactly once and no partial batch appears after simulated power loss.

## 3. Live Store and Segments

- [ ] 3.1 Make bounded typed Arrow batches the live store fed by single-decode admission; prove with `sidecar_ingest` moving from about 50k/s toward 0.2 to 1M samples/s.
- [ ] 3.2 Flush segments to Arrow IPC in the background with one catalog transaction per segment; remove sample rows from SQLite; prove catalog contents and journal reclamation only after durable publication.
- [ ] 3.3 Serve queries over live batches and catalogued files at one cutoff; prove exactly-once results during concurrent flush and with `arrow_telemetry/window_query_1000000` at or below 720 ms to 1 s per million samples.

## 4. Sealed Evidence

- [ ] 4.1 Seal executions to Parquet with row-group statistics, retaining IPC until the Parquet file is durable; prove crash recovery mid-seal and `workflow_seals_execution` with sync count bounded by batches, not members, from the 9.8 s baseline for 213 members.
- [ ] 4.2 Update evidence exports and their fixtures for Parquet; prove byte identity of exported files and readability with an independent Parquet reader.

## 5. Query Engine

- [ ] 5.1 If the owner places it in this change, evaluate documents through DataFusion over live batches and files; prove equal results to the current evaluator and `document_queries` within the current baseline range.

## 6. Documentation and Closure

- [ ] 6.1 Update `docs/product-guide.md`, the storage topic guides and README claims affected by the new layout and fresh-root requirement; review by reading.
- [ ] 6.2 Delete superseded sample tables, per-batch catalog writes and their tests; close every implementation item in the combined branch.

## 7. Validation and Delivery

- [ ] 7.1 Validate the change strictly and run the named benches once after source closure.
- [ ] 7.2 Deliver through one PR into `dev` when authorized.
