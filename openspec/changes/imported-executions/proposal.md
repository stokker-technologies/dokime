# Imported Executions

## Why

People already have measurements in MATLAB files, JSON and logs. Today they rig
a script to make a plot. The script should load and post-process those files in
its own language, shape the result into Dokime streams and records, and send a
document definition. Dokime then stores a formal execution, seals it, generates
its report and lets the script download it.

## What Changes

This is the locked target design, not implemented behavior.

An import is an ordinary execution in the same store, inside the cycle the caller
names; create that cycle if absent. It has resolved parameters and streams, no
steps and no criteria. There is no dataset entity or fabricated plugin/test case.
History, plots, statistics, documents and reports use their existing execution
paths. Original source files may be attached as artifacts with verified hashes.

```python
imported = dokime.Import(cycle="Thermal Campaign", label="Run 42", key="run42")
imported.attach("run42.mat")
imported.stream("temperature", time=t, values={"inlet": inlet, "outlet": outlet})
imported.record("ambient", {"humidity": 41.0})
imported.outcome(dokime.Outcome.PASSED)
imported.document(dokime.Document("Thermal Run 42").plot(dokime.Plot("Temperatures").series("temperature.inlet").series("temperature.outlet")))

execution = client.import_execution(imported)
execution.report.save("run42.html")
```

Each SDK uses this value-composition shape in its language's idiom, including
awaits where its client is asynchronous. No public API requires a context
manager or an extra block. The document/plot conveniences build the existing
`dokime.v1.Document`, sections, cards and typed field references; a series path
resolves against the supplied stream schema before admission. They introduce no
second document format or host expression parser.

### Origin Contract First

Add the origin union to `dokime.v1` before the `grpc-plugins` execution manager
is built. The existing `TestCase.template`, `plugin` and `digest` move to
`PluginRun`; `digest` identifies the materialized test case's catalog snapshot,
as in the current protocol. Plugin start time comes from the connected plugin
description. The execution owns provenance, and plugin calls receive it alongside
the full case. Reserve the removed TestCase field numbers/names, retaining its
parameters, steps, criteria and stream field numbers. No compatibility shim is
needed for development storage.

Protobuf sketch (new field numbers on existing execution/call messages are
assigned during protocol implementation):

```proto
package dokime.v1;
// Uses google.protobuf.Timestamp and the existing Value and TestCase messages.
message ExecutionOrigin {
  oneof kind {
    PluginRun plugin_run = 1;
    ImportOrigin import = 2;
  }
}
message PluginRun {
  string plugin = 1;
  string template = 2;
  bytes digest = 3;
  google.protobuf.Timestamp started_at = 4;
}
message ImportOrigin {
  repeated ImportSource files = 1;
  string client = 2; // Importing client identity.
  google.protobuf.Timestamp imported_at = 3;
}
message ImportSource {
  string name = 1;
  bytes sha256 = 2;
  optional string artifact = 3;
}
message TestCase {
  reserved 1, 2, 3;
  reserved "template", "plugin", "digest";
  map<string, Value> parameters = 4;
  repeated Step steps = 5;
  repeated Criterion criteria = 6;
  repeated string streams = 7;
}
// Execution gains ExecutionOrigin origin; plugin requests gain origin beside
// test_case. Import executions have no plugin TestCase or plugin invocation.
```

The import carries source-file names and hashes even when their bytes are not
attached. When attached, the host verifies the bytes against their hashes. Import
time is distinct from source measurement time. Origin is retained in the seal.

### One Call and One Commit

`client.import_execution(imported)` submits one composed value through the
application interface that already runs cycles and executions. The Public
`dokime.execution.import` operation owns upload, storage, sealing and report
generation. Bulk input is one columnar table per stream, never a message per
sample; its explicit source timestamps remain the time axis for plots and queries.

Extend that interface with four host operations, following the existing
`dokime.execution.*` naming convention:

| Operation | Responsibility |
| --- | --- |
| `dokime.execution.import.begin` | Reserve the key and resolve or provision the named cycle in staging. |
| `dokime.execution.import.write_stream` | Write one stream table with its schema and source timestamps. |
| `dokime.execution.import.put` | Put parameters, records, source artifacts, document and optional declared outcome. |
| `dokime.execution.import.seal` | Seal, generate the report and publish the execution atomically. |

These four are transaction-scoped host steps of the single import call, not
independently committed client round trips. Follow existing handler/resource and
authorization boundaries; callers cannot publish a partial import. Report spool
files remain with their existing owner. Stage all effects and publish only once
the report is ready. Failure, cancellation or crash leaves no execution, added
cycle membership, new cycle, artifact, report or consumed key behind; recovery
removes abandoned staging without changing an existing cycle's prior contents.
Independent storage writes are not assumed to be a cross-method transaction.

The caller supplies `key`. A repeated key for a sealed execution is refused with
an error naming that execution, rather than returning it or duplicating it.
Key reservation and publication must serialize competing uses in the same store.
A failed call can be retried with its key after rollback; if a response was lost
after commit, the duplicate error identifies the sealed execution to retrieve.

An importer may declare passed or failed. This is stored and displayed as
declared by the importer, never evaluated by Dokime. Absence of a declaration
does not imply passed. No import criterion engine is added: criteria remain
plugin operations. History, plots, documents, reports and seals visibly mark
imported provenance, including the source hashes and declaration attribution.

### Flat Python References

Python examples compose values and call operations without `with`. Resources
use explicit `close()` (or an existing async reader's `aclose()`), awaited where
needed; use `try/finally` when release must survive failure. Backend startup and
owned cycle cleanup need public flat entry points rather than magic methods.
Reference edits describe that target where source cannot yet support it.

## What Stays

Plugin execution ordering, retry policy, plugin-operated criteria and evaluated
verdicts stay. Cycles retain their existing lifecycle responsibilities. Import
uses the ordinary evidence, document and report mechanisms, storage identity
policy and application authorization. Source parsing and post-processing belong
to the importing script; Dokime adds no MATLAB/JSON/log parsers. Existing data
is never silently deleted or reset; incompatible development storage uses an
explicit fresh root, with schema/evidence baselines still v1.

## Impact

Protocol and generated bindings; execution persistence and call envelopes;
application operations and runtime transaction/recovery; the columnar storage
path; Rust, Python, C# and C++ client SDKs; history and plot interfaces; report
and seal presentation; guides and SDK documentation. No source implementation,
build or test is part of this specification task.

## Dependencies

- `grpc-plugins` adopts the origin union before its execution-manager work,
  regenerates bindings and migrates materialization and plugin calls from the
  start. Its SDK wave supplies all four client halves and shared documents.
- Import implementation follows that SDK wave and `storage-interface`, which
  starts after gRPC wave 5. Reuse its columnar path and staged file/durability
  capabilities; do not replay imported data through live per-sample ingest.
- `arrow-telemetry-storage` remains the columnar foundation; coordinate through
  `storage-interface` without beginning or expanding its implementation here.
- The flat API deltas below supersede context-manager requirements in
  `python-client` and `python-backend` while those APIs exist. The gRPC wave still
  removes the Python backend; this does not revive it or extend its lifetime.
