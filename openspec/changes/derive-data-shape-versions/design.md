## Agreed Design

Data shape revisions derive from Harmos schemas and are opaque equality keys.
The published Harmos pin is `62a554c74010e3b771d060a366fdead44de07a0e`,
which resolves the previous Serde tagging, renaming and omission blocker.

Record contracts derive from their declared field graph through Harmos's shared
schema-only revision function. Telemetry constructors take the declared stream window and derive once from the
row graph and that window. Host declarations reuse their public row types.
Telemetry contracts carry the stream revision,
which becomes `SchemaKey.version`; evidence and query fields carry that identity
so historical definitions can be selected. Persisted product records, transactions
and stored objects use `Described` and `Identified::ID`.

The whole-root schema/source digest and its build inputs are removed. The local
database validates its own layout; Harmos validates built-in journal replay and
snapshot decoding. Existing obsolete digest files remain untouched. Known obsolete
Dokime state files refuse by name because they cannot be replayed. Plugin settings,
secret values, installation metadata and document evidence are data under product
containers, never members of a root compatibility hash. Operational admission and
pending physical-effect restoration retain their existing safety rules.

## Retained Identities and Framing

- The interned runtime snapshot has custom `Serialize`/`Deserialize` implementations; the native derive cannot describe that representation, so its existing key remains, as the owner authorized for individually unsupported types.
- `AppendExecutions`: nested maps and tuple collections are outside the derive's field grammar; keep its current journal identity.
- `RegisterCases`: maps keyed by `CaseId` are unsupported; keep its current journal identity.
- `FinishProvider`: its `Result` payload is unsupported; keep its current journal identity.
- `InstallPlugin`, `InstallHostPlugin`, `ReconcileProviders`, `ReloadProviderCatalog`, and `SetSuiteSet`: catalog payloads reach custom Serde conversions, flattened document cards, and boxed expression containers; keep their current journal identities.
- `AddExecutionDocumentParts`: captured document declarations reach flattened cards and boxed expression containers; keep its current journal identity.
- `RequestReportFinalization` and `CaptureReportLifecycle`: `uuid::Uuid` has no Harmos description; keep their current journal identities.
- `FinalizeExecutionOutputs`: aggregate output contains `time::OffsetDateTime`, which has no Harmos description; keep its current journal identity.

The journal catalog dispatches derived and retained members together through
Harmos's supported catalog/codec traits. It supplies no fabricated schema for a
retained member. All current transactions are forward-only, matching the previous
catalog's absent inverse handlers.

- Captured native bundle framing validates unsupported input before structural and identity checks; its version is retained.
- Export framing stamps the manifest, SQLite artifact and download metadata; clients reject unsupported framing, so it is retained.
- Pairing files validate the supported untrusted input format before importing credentials; its version is retained.
- SQLite's layout sentinel protects the product database layout; it remains independent of plugin and evidence schemas.
- Record and telemetry definition/catalog/evidence wire fields carry derived historical identities, not authored ordinals.

Native system facts now carry Harmos’s payload-encoding identity. Their captured
record carries its separate contract identity; projection still checks owner and
record type, and no longer equates those two different revisions.

Prompt identity comparisons now use prompt type and title; the removed ordinal
was always a fixed value. The fixed report ordinal was unread. Profile, remote security and session files use
structural and resource-bound validation. Page cursors retain fixed-length,
checkpoint, source and boundary validation after removing their ordinal byte.
API, plugin release, package and analysis algorithm versions remain unchanged.

## Built-In History Boundary

Changing a built-in typed journal definition may require a fresh root. Keeping
those entries readable would require storing each entry's historical definition
and decoding through it before product replay; that journal evolution mechanism
is outside this change. No migration, deletion or automatic reset is introduced.

## Validation Boundary

Use deterministic targeted tests, then the owner's all-targets feature build,
TypeScript build and `lint:rust`. No full lanes, starters or CI simulations.
The final candidate must pass the specified scanner with `CLEAN` before merge.
