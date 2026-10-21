---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-architecture/SKILL.md; sha256:a7b2cd4d6cfb2911e538d486b71cf06ead0d0cc0b551ce66116617d54d5154b6 -->

# Dokime Architecture

Apply the shared architecture and Rust rules. The entry library is `dokime`: domain definitions are under `dokime::domain`, runtime functionality is re-exported at its root, and remote access is under `dokime::client` with feature `client`. Shipped interfaces, plugins and cargo tools consume these surfaces through `dokime`. The `dokime-cli` package owns command parsing and commands and installs the `dokime` executable. Internal `dokime-domain`, `dokime-runtime` and `dokime-client` crates retain implementation ownership; `dokime-packaging` owns build and packaging utilities. During an in-progress cutover verify landed paths before citing an API.

Connection, installation and version helpers belong to the entry library. Connection helpers use `dokime::client_error::Error`; the CLI reuses that error instead of wrapping it. Root `dokime::Error` and `dokime::Result` retain runtime identity across feature selections.

The domain crate contains every operation, transaction, state entry, message and value definition. It performs no I/O, async work, storage or network access. Runtime crates implement the resource-bound behaviour and depend on domain crates; domain crates never depend on runtimes. Sidecars, plugins, clients and tools use the portable domain surface.

Organize declarations kind-first: `model/`, `operations/`, `transactions/`, `state/`. Each kind's `mod.rs` is its closed catalog, with folder modules by domain. Runtime implementations mirror the same domain names and paths. A plugin's private definitions live in its `domain/` module; move a declaration to a shared domain crate when another party consumes it typed.

Records select by envelope identity: scope is the cycle, item is the execution (or cycle for cycle context), source is the provider/process epoch. Use typed reads for records and retain columnar queries for plots and telemetry derivations. Lifecycle capture references the durable frozen execution revision rather than duplicating it.

Packaging uses the entry library and Harmos packaging without enabling host services. Build scripts use `dokime::domain` contracts and request only enabled Harmos generators (`generate-proto`, `generate-python`, `generate-typescript`); `grpc` implies proto. The thin cargo tool uses the entry library and packaging instead of embedding a second host runtime.

Storage is development v1. A clean schema break may require a fresh storage root; never silently reset existing data. Plugin installation and enablement become active after a full app restart; development reload remains an explicit separate lifecycle.

## Accepted Remote Cutover

The remote-streamline implementation is migrating to explicit container phases: Active, Sealing, Sealed. Several cycles may be Active; one server-wide execution lane admits each execution identity at most once. Templates materialize cases without starting work; Execution(case) constructs an unstarted attempt and cycle append durably registers it. Start is idempotent by execution identity without caller request keys. Blocking Run composes Start with durable completion and requested listener drain. Cycles have no Start/Run/Stop operations; they carry optional server-owned ordered continuation. Natural completion and sealing may advance to the next unstarted member; explicit execution Stop halts continuation and durably seals that execution. Failure follows declared policy, and appending to an idle cycle does not dispatch hardware. Execution completion leaves its cycle Active. Cycle Seal closes admission, stops unfinished members and waits for durable execution and cycle seals before removing it from Active while retaining history. Saved seals and truthful outcomes are independent. Startup seals previous-session cycles without replaying device work; server SIGINT/shutdown seals all active containers. A GUI reconnecting to a surviving server sees its Active cycles; closing a GUI-managed server seals them. Aggregate outcomes do not control admission. Verify landed source before using these accepted contracts as available APIs.

Record-type and stream observation filters use vectors of case-sensitive whole-identifier globs, with OR within each vector. Star matches any sequence including dots, question mark one character, and plain identifiers match exactly. Empty vectors select all within the selected execution when listening is enabled. The canonical server read/watch boundary owns matching; reject unsupported syntax before Start and preserve replay, pagination and final drain. Filters/callbacks do not change captured evidence or exports. Regex syntax and client-owned scheduling are excluded.

The accepted Python shape returns a bound Cycle directly from create_cycle. Resolve a Template, materialize cases on the server, construct synchronous Execution(test_case) objects locally, append them atomically, then run them in a separate explicit loop. Materialization captures immutable parameters and authoritative definition/provider revision references without execution attempts or hardware. Registration checks that captured revision atomically; append rejects stale definitions or providers instead of rematerializing. Append preserves client-supplied execution UUIDs and binds the same Python objects only after confirmation. Identical retries do not duplicate attempts, conflicting bindings or order are refused, and retired attempt identities cannot be reused. A script that owns its sequential Run loop disables automatic cycle continuation.

```python
template = await client.template("obc.checks.pulse")
cases = await template.materialize(parameters={"sample_count": SAMPLE_COUNT})
executions = [Execution(test_case) for test_case in cases]
cycle = await client.create_cycle(label="Remote Pulse Demonstration")
await cycle.append(executions)
for execution in executions:
    outcome = await execution.run(record_types=RECORD_TYPES, streams=STREAMS)
await cycle.seal()
```

This is the accepted complete cutover shape; check source readiness before treating the whole example as implemented.

Native local and remote listeners have independent ownership. Receiving listener identity controls local-only operation admission and host-path disclosure; a remote call through loopback never inherits native trust. The server shell owns listener settings, TLS material, key verification and listener restart/rollback. The runtime owns admitted executions, so listener changes and key rotation cannot cancel them. Persist the certificate/private key and key verifier under the application root; expose a generated full-control access key once and the public certificate separately. TLS and authentication are independent explicit choices. Retained certificate identities change only through explicit regeneration, with clients updating trust. A valid key grants full control over remote product and plugin operations, without per-user roles or permission files. Authentication-off listeners have the same operation authority. Original Harmos provider provenance distinguishes plugins from internal host providers, which remain native-only. Preserve independent plugin-to-plugin capability requirements and same-operation grant identity. Run receipts use stable listener attribution rather than key-specific ownership; a rotated key can recover and control admitted work without replaying it.

## Accepted Evidence Export

The remote cutover's exporter is runtime-owned under `domains/exports/`. A sealed execution can be exported from an Active cycle; a cycle export pins its sealed header and every exact execution revision named by its seal. Read captured context, records, typed samples and files through those frozen views. Never reconstruct evidence from client observations or materialize a new report as an export side effect. Missing required evidence fails the whole export.

The version-1 ZIP contains `manifest.json`, `evidence.sqlite`, `files/` and `reports/`. The manifest names the subject, node/cycle/execution identities, exact saved revisions, captured boundaries, completeness, row counts, stream/table mappings and each payload's length/SHA-256; it excludes its own checksum. Transfer metadata and terminal completion carry the archive length/SHA-256. Files use an execution/output-index namespace and retain their original logical path in the inventory; reports are the saved immutable HTML. Never traverse application settings, remote security material or credential stores.

SQLite uses explicit cycle, execution, definition, input, step, criterion, traceability, record, stream, stream-column and output tables. Complete canonical JSON accompanies the convenient scalar columns, preserving heterogeneous and nested values. Declared stream schemas receive deterministic tables and distinct indexed column names; `stream_columns` maps them to original field identities, value types and units. Schemas wider than 512 fields use partitions joined by `m_row`; the manifest lists every partition. Boolean values use INTEGER, text/datetime TEXT and structured cells JSON TEXT. Numeric cells use STRICT-table ANY to preserve the captured variant: floating values are REAL, signed-range integers INTEGER and larger unsigned integers exact decimal TEXT, with no float conversion. Raw canonical values retain their original signedness. Missing cells remain SQL NULL. Extra sample fields remain queryable in sample JSON without guessed types.

```sql
SELECT execution_id, status FROM executions;
SELECT execution_id, stream_id, table_name FROM streams;
SELECT table_name, field_id, column_name, value_type, unit_json FROM stream_columns;
SELECT execution_id, COUNT(*) FROM records GROUP BY execution_id;
```

One bounded build owner writes SQLite and ZIP to private temporary files under the application's export cache, consuming native pages/chunks incrementally. Enforce an explicit archive ceiling and fail rather than truncate. Client cancellation ends the export/transfer, never admitted execution. All adapters share this exporter and verify contiguous chunks, size, checksum and final transport success before atomic destination publication; overwrite is explicit for evidence exports. These are accepted implementation contracts, not a claim that the export API has landed.
