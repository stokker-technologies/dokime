# Imported Executions

## Why

Scripts already parse and process measurement files. They need to retain that
evidence in Dokime and iterate on a report without impersonating a plugin or
claiming Dokime evaluated the measurements. This is the locked target design,
not implemented behavior.

## What Changes

An `Import` is the definition for filling an execution with existing evidence.
The execution model, origin union, frozen schemas, attributed outcomes, backend
capabilities and render/publish split belong to `grpc-plugins`, before its host
execution manager is built. This change depends on that model rather than
introducing another execution-definition type or a stored run.

```python
imported = dokime.Import(cycle="Thermal Campaign", label="Run 42", key="run42")
imported.attach("run42.mat")
imported.stream("temperature", time=timestamps, values={"inlet": inlet, "outlet": outlet})
imported.stream("pressure", start=recorded_at, elapsed_s=seconds, values={"line": line})
imported.outcome(dokime.Outcome.PASSED)

draft = client.import_execution(imported)     # staged, published atomically, unsealed
draft.preview(document_v1).save("try1.html")  # nothing stored
draft.preview(document_v2).save("try2.html")
execution = draft.seal(document_v2)           # joins its cycle, report published
execution.report.save("run42.html")
```

Rust, Python, C# and C++ expose this flat shape in their SDK idiom, with awaits
where appropriate and explicit resource release. No block construct is required.
Document builders produce the existing `dokime.v1.Document`; no second document
format or expression parser is introduced.

### Time and Columnar Evidence

Each sample retains absolute UTC source time, host receipt time and durable
position. Each stream declares its axis. Supply either absolute timestamps or a
UTC start instant with exactly one elapsed-offset array: `elapsed_s`, `elapsed_ms`,
`elapsed_us` or `elapsed_ns`. Neither, both, ambiguous local dates, incompatible
lengths or unrepresentable nanoseconds are refused. Numeric time arguments name
the unit; the wire carries integer nanoseconds. History places an import at its
import time and separately exposes its measurement period, the UTC range of its
data. Upload time never rewrites source time.

Imported stream tables use the same `dokime.v1` columnar-frame message as the
plot pipeline: typed arrays, explicit null validity, row count and timestamps,
preserving signed/unsigned 64-bit values and bytes. Arrow/dataframe support is an
optional SDK layer. Import-local schemas come from the script and are frozen
without plugin registration or overwriting any existing definitions.

### Staging and Drafts

`import_execution` is one SDK convenience over bounded, checksummed, resumable
transfers followed by atomic publication of a validated manifest as an open,
unsealed execution. It does not use live stream admission or partially accepted
samples. The host owns limits and backpressure. Invalid or incomplete manifests
publish no draft. Interrupted uploads retain only resumable staging, reclaimed
by host policy if abandoned.

The draft is visible only through its handle. It is absent from cycle membership,
other clients, history, search and ordinary exports until seal. Handle-scoped
queries support preview at a fixed revision; evidence follows are finite at that
revision's committed frontier. An upload is not a running execution.

`preview(document)` validates and renders that revision into a returned report
without durable writes, report publication or mutation of the draft. It may be
called repeatedly with different documents. Temporary output belongs to the
renderer and is released on completion/cancellation. `seal(document)` freezes the
final document and report and atomically publishes the seal, report and cycle
membership. A render failure leaves the draft available and nothing published.
`discard()` removes the draft and schedules reclamation of its storage unit.

Handles and preview results bind store identity, execution and draft revision.
Replacement and seal require the expected revision; discard also checks it to
avoid removing a replacement. Active previews pin their revision against expiry;
replacement cannot change their snapshot. A stale preview cannot authorize a
seal of newer evidence. The host owns unsealed-draft expiry, which is distinct
from interrupted-run recovery. Draft attachments, columns, projections and
metadata form an independently reclaimable unit; logical removal and completed
reclamation are distinct, without promising a smaller shared database file.

### Keys, Retries and Trust

An import key identifies the whole import within the backend store identity.
Request identity identifies a retry of one mutation. Successful mutations retain
durable receipts: replay of the same request and payload returns its original
result, including after seal, rather than applying it to a newer draft. Reusing
a request identity with different payload is a conflict. A new request with an
existing unsealed key conditionally replaces its draft; a new request with a
sealed key is refused naming the existing execution. Competing creates and stale
revisions return explicit conflicts. Restart retains identity and receipts;
a fresh store has a new key namespace and cannot prevent duplicates across roots.

Any connected client may call any operation under the agreed trust model.
Importer labels and handles do not establish per-user ownership. Handle-only
visibility is an API discovery boundary, not a new authentication system.

The host hashes uploaded bytes and verifies manifest checksums. Unattached source
hashes are importer claims. Evidence exports carry origin, claimed importer and
SDK version, source inventory/hashes, script-supplied processing description and
declared outcome. Importer-supplied traceability is allowed and marked as such;
step/attempt references and criteria are rejected. Import execution and cycle
reports explain that sealing attests persistence, not measurement authenticity.

Imports are whole submissions. A correction creates a new execution linked to
its predecessor and uses a new key if the predecessor is sealed. Incremental
imports and mutation of sealed evidence are out of scope.

## Dependencies

- `grpc-plugins` owns the execution model in its host wave, before manager
  construction, and supplies the four SDK client halves. Import implementation
  follows that SDK wave; the model is not deferred until imports are implemented.
- `storage-interface` owns the persistence boundary used for staged publication,
  conditional update and durable receipts. Import implementation must establish
  the manifest commit boundary through that interface, not assume independent
  backend writes compose into a transaction. Missing staged-publication support
  is a dependency obligation before import implementation, not a local workaround.
- The plot pipeline owns the shared `dokime.v1` columnar-frame message; reuse it
  rather than fork a transport table. Columnar persistence follows the storage
  interface, without starting other changes here.

## Impact and Scope

Protocol operations/bindings, storage and recovery, four SDKs, history and queries,
desktop controls, reports/exports and affected README/guides require implementation
and focused proof in tasks.md. This specification task changes only OpenSpec;
it runs no builds or tests. Existing development data is never reset silently;
incompatible storage requires an explicit fresh root and keeps the v1 baseline.

## Open Decisions

The locked model leaves review issues **4, 11, 12, 23, 24 and 26** partially open.
Their exact questions, implementation gates and all 30 dispositions are in
`design.md`. No unresolved policy is presented as an implemented guarantee.
