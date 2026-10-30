# Long-Execution Finalization Tasks

## Implementation

- [ ] 1.1 Runtime benchmarks: deterministic one-hour/four-hour fixture, separate Stop/Cancel/report-save baseline measurements and dominant-cost instrumentation. Benchmarks explicitly authorized during investigation.
- [x] 1.2 Execution lifecycle/captures/storage/log tails: join each physical drain, preserve work across waiter cancellation and retries, fix cleanup before a log source opens, and prove shutdown joins an in-flight stop with invocation counts.
- [x] 1.3 Telemetry/storage: incrementally persist exact aggregates and eliminate full-history completion scans with bounded memory and durable recovery.
- [x] 1.3.1 SQLite evidence indexes: index both foreign-key child position columns; prove metadata retirement does not scan accumulated telemetry signals or record captures.
- [ ] 1.4 SQLite telemetry/report owners: persist configurable adaptive time buckets with first/last/min/max/count in the sample transaction; replace report history scans with indexed bucket and bounded first/last row reads. Raw evidence and exact numeric indexes remain complete; storage v1 clean break.
- [ ] 1.4.1 Report/document/plot consumers: render bounded extrema envelopes; use exact numeric aggregates for supported summaries; visibly identify unsupported history-dependent analyses, filters and transforms without scanning or approximating them. Bound record/table previews and point to full evidence.
- [ ] 1.4.2 Execution lifecycle/storage owners: close and drain before returning Stop/Cancel; durably schedule background report publication independently of provider completion. Preserve cancellation, retry and restart ownership; report errors never fail Stop. Shutdown preserves pending work without waiting for HTML.
- [ ] 1.4.3 Domain/server/generated client/GUI/Python owners: expose pending/ready/failed report state and causes; report downloads and cycle.save await publication. Regenerate affected contracts and migrate existing consumers.
- [ ] 1.5 Focused tests: preserve evidence, exact summaries, cancellation safety, joined finalization and bounded output; migrate affected old expectations and repair the simulation launch Clone requirement found by focused compilation.
- [ ] 1.6 Documentation: review README impact and update affected product/storage/report contracts and benchmark instructions.

## Final Validation And Delivery

- [ ] 2.1 Run one-hour/four-hour after measurements and verify flat end-of-run cost; record native benchmark evidence under target.
- [ ] 2.2 After source closure run requested workspace check, format and focused tests; report actual platform limitations without claiming unrun proof.
- [ ] 2.3 Commit logical fixes only, retain the updated change for integration and hand back at most eight lines with numbers. No push or PR.
