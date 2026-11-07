## Agreed Design

Data shape revisions derive from Harmos schema identity and are opaque equality keys. Plugin-owned shapes never invalidate a root; product-owned persistent state refuses only when it cannot be decoded. Existing evidence keeps its stored definitions. API, algorithm and package versions are outside scope.

## Dependency Contract Requiring Resolution

The pinned Harmos revision makes `Identified` require `Described`. Dokime's built-in journal types (`model::records::evidence::Record`, `state::records::CapturedRecord`, and `state::records::work::Work`) previously supplied no schema description. Adding the native derive and following their field dependencies exposes unsupported Serde representations:

- `model/execution/scope.rs`: `EvidenceOrigin` uses `#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]`. Harmos's `Described` derive rejects `tag`.
- `model/records/contract/severity.rs`: `RecordSeverity` uses `#[serde(rename_all = "snake_case")]`. The derive rejects `rename_all`.
- `model/records/evidence/record.rs`: the optional error uses `skip_serializing_if`, also rejected by the derive.

The errors come from Harmos's `harmos-macros/src/message.rs` stored-Serde `shape` implementation. Its supported container attributes are currently `transparent`, `deny_unknown_fields`, and `default`; supported field attributes are `skip`, `rename`, and `default`.

A clean adoption needs a single schema owner that accurately describes these existing representations. Recommended resolution: extend the Harmos derive to support the Serde representations reached by Dokime's persistent state, validate those schema identities upstream, then pin that checkpoint here. Hand-written parallel schemas would duplicate the declarations; changing serialization to fit the derive would expand this task beyond deriving existing shapes. No such workaround is implemented.

## Inventory Findings

The proposed removal inventory contains live reads, so these are not all dead constants:

- Native captured-source schema version rejects mismatched framing before structural and identity validation.
- Evidence export schema version stamps manifests, download metadata and SQLite exports; the client checks metadata.
- Prompt schema version participates in Rust and Python pending-prompt identity comparisons.
- Remote pairing, backend profile and cursor versions participate in decoding/validation.
- Record/telemetry definition, catalog and evidence wire revisions select captured historical schemas. These should remain derived identity carriers unless their selection behavior is replaced.
- The broad root digest includes evidence codecs and source text. Its replacement remains unimplemented; existing-root behavior is unchanged.

## Validation Boundary

Targeted `mise run build:rust:focused -- check --locked -p dokime-domain` fails with the representation errors above (task exit 1). The early ready PR's hosted Windows and Unix Rust builds also fail on the new `Described` requirement. Implementation is incomplete: no requested deterministic tests or final all-targets, TypeScript or Clippy gates have run. Nothing has been merged. Incomplete exploratory source edits were removed; only the dependency pin and tracked specification remain.
