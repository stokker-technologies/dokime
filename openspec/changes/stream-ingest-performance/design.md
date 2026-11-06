# Stream Ingest Design

## Ownership and Contracts

The runtime storage writer owns durable Arrow batches and atomic catalog cuts. Plugin publication retains bounded ordered batches. Shared numeric field summaries replace row-by-plot folding for eligible projections; filters, correlations and exact-input analysis keep their correctness and caps. Static projection definitions are stored once and only changed state is checkpointed. Raw Arrow rows remain unchanged. Development storage stays v1; incompatible roots are reported, never reset.

## Implemented Shape

Runtime publication settings travel in the execution request: 4,096 rows and 1 MiB by default, capped to the declared stream window. A configurable 2 ms background flush delay coalesces partial batches; full batches and explicit evidence barriers publish immediately. Native sidecar and embedded publishers move owned encoded batches. Typed push maps are built only for consumers that require them; subscriptions retain their values. One oversized schema-admitted row travels alone. Host admission moves uniquely owned samples into the live tail after encoding, retaining a clone only when another capture still owns the sample.

The writer normalizes each admitted sample once. Arrow batches construct columns from borrowed admissions, without per-row metadata maps or cloned value maps. Identities use multi-row inserts; unchanged contiguous scopes reuse serialization. Numeric moments and adaptive extrema envelopes update once per captured execution/stream/field, using borrowed map lookups. Shared numeric checkpoints use Postcard rather than repeated JSON/deflate work. The shared projection retains typed exact inputs through its cap, then clears that prefix; a larger later cap cannot recreate discarded inputs. Nulls preserve the input count and exact representation. Stream context merges the latest non-null value per field.

Plain numeric time-series plots read those shared fields. Other projections retain their specialized semantics, deduplicated by canonical request. Plot IDs bind to projections; immutable definitions and compressed mutable checkpoints have separate columns. Only changed specialized checkpoints are updated. Shared projections are reconstructed when read, outside ingestion, and identical views share their loaded checkpoint. Historical cuts retain the existing refusal/replay rules.

Persistent numeric rank tries are deleted. Exact percentiles use the complete typed prefix when available; otherwise the report reads bounded numeric Arrow pages and orders lossless floating-point keys in a disposable SQLite file. Document summary medians use the same replay; dev's moment-based integer fast path keeps its lossless-range guards and typed-query fallback. Current-cut counts are checked against durable moments. This moves work from every ingestion flush to the reader that needs exact ranks, with temporary disk proportional to selected raw values and bounded page/cache memory.

## Validation

A native OBC benchmark fixture publishes scalar and wide rows through the sidecar transport to the saved barrier, including 0, 1 and 64 plots. Record wall and CPU time, rows and logical MiB per second, raw Arrow/catalog/WAL bytes and cost attribution, repeating measurements on this potentially busy machine. Focused tests verify evidence, ordering, shared state and report results. Review README impact; performance facts belong in docs/product-guide.md.

## Measurement Follow-Up

The ingest2 follow-up keeps production cadence, durability points, deadlines and budgets. Benchmark-only SQLite VFS observation delegates the original xSync unchanged and counts successful sync requests, separating WAL requests. Arrow sync spans and writer-call counts cover the host durability boundary; post-commit VFS notifications count committed catalog write transactions, including autocommit. Explicit catalog commit-call counters remain separate, including no-op commits, and neither is inferred from VFS sync counts. Live process CPU is reported per PID/name rather than attributing process-tree CPU to the writer. Inclusive span time is active thread/poll wall time, including preemption; lock-wait events include suspension. These quantities overlap and are not a serial budget.

Payload fixtures carry an f64 plus row-varying printable text totaling exactly 4 KiB or 64 KiB per row. Durable Arrow replay verifies byte length outside the timed interval. They measure logical admitted payload throughput, with compressed Arrow bytes reported separately.

Trim eligibility requires measured benefit and exact existing semantics. Tail normalization applies to a separate in-memory representation after original-frame encoding; the writer decodes that original frame, so removing its normalization is not inherently safe. Whole-file SHA-256 and independently verifiable batch SHA-256 have different hash states; hashing each byte only once cannot preserve both digests. TelemetrySampleId preserves UUID letter case; blindly packing to 16 bytes would collapse currently distinct IDs. No identity, digest or normalization contract changes are authorized by this follow-up.

Benchmark-feature plugin builds retain the steady suite used by report benches, the recovery suite used by server workflow benches and the benchmark suite, excluding unused product example suites. Shipping builds retain the complete product catalog. Ingest fixtures keep static 0/1/64-plot documents, and the benchmark checks durable plot bindings as well as row counts. Payload cases select zero plots. This keeps the fixture within unchanged state-record budgets.
