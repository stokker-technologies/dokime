# Long-Execution Finalization Tasks

## Implementation

- [ ] 1.1 Runtime benchmarks: deterministic one-hour/four-hour fixture, separate Stop/Cancel/report-save baseline measurements and dominant-cost instrumentation. Benchmarks explicitly authorized during investigation.
- [x] 1.2 Execution lifecycle/captures/storage/log tails: join each physical drain, preserve work across waiter cancellation and retries, fix cleanup before a log source opens, and prove shutdown joins an in-flight stop with invocation counts.
- [x] 1.3 Telemetry/storage: incrementally persist exact aggregates and eliminate full-history completion scans with bounded memory and durable recovery.
- [x] 1.3.1 SQLite evidence indexes: index both foreign-key child position columns; prove metadata retirement does not scan accumulated telemetry signals or record captures.
- [ ] 1.4 Report/document/plot owners: incrementally persist bounded report inputs and replace full-history materialization; resolve rendered-data contract first.
- [ ] 1.5 Focused tests: preserve evidence, exact summaries, cancellation safety, joined finalization and bounded output; migrate affected old expectations.
- [ ] 1.6 Documentation: review README impact and update affected product/storage/report contracts and benchmark instructions.

## Final Validation And Delivery

- [ ] 2.1 Run one-hour/four-hour after measurements and verify flat end-of-run cost; record native benchmark evidence under target.
- [ ] 2.2 After source closure run requested workspace check, format and focused tests; report actual platform limitations without claiming unrun proof.
- [ ] 2.3 Commit logical fixes only, finish the specification when all obligations are closed and hand back at most ten lines with numbers. No push or PR.
