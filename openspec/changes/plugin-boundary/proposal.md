# Plugin Boundary

## Why

Several products will embed Harmos. Dokime must own its plugin policy and everything
an author touches while reusing Harmos's runtime, carrier and tooling. The owner
locked this split on 2026-11-16; today's library-owned registry/settings and exposed
Harmos SDK types do not satisfy it.

## What Changes

- **BREAKING.** Dokime owns its plugin contract, the only author-facing Rust,
  Python, C# and C++ SDKs, starters, installation, registry, bundles and artifact
  digests, distribution, persisted settings and plugin management operations.
- Register a product plugin kind with product admission and settings-resolution
  callbacks. The registry selects what and when to load and calls the Harmos typed
  facade plainly; Harmos retains its plugin manager and live lifecycle machinery.
  No `PluginBackend` trait.
- Consolidate richer descriptor/codegen renderers and fixes into Harmos. Replace
  `crates/dokime-runtime/src/plugin.rs` spawn/connect/Get/Stop inspection with its
  bounded inspection facility, retaining product interpretation only.
- Move Harmos bundle code into Dokime and remove the duplicate archive builder;
  keep one product implementation, preserving atomicity, receipts and explicit
  incompatible-root refusal without migrations.
- Hide Harmos types, attributes and namespaces from authors without import aliases;
  use its thin carrier helpers internally in all four languages. Ship one
  self-contained contracts archive per release with Dokime and Harmos carrier
  protos, imported `google/*`, licenses and protoc/Buf instructions.
- Regenerate once with `operation-boundary` and `plugin-lifecycle`; split generic
  carrier conformance from Dokime authoring and domain conformance.

This amends architecture invariant 15 and the two changes' SDK and management
assignments. This documentation delivery records direction, not landed behavior.

It also supersedes `operation-grammar`'s library-owned SDK and plugin administration
assignments, preserving visibility, callers and declaration integrity. The earlier
research recommendation to keep full Harmos author SDKs is replaced by internal
carrier helpers and Dokime-owned author APIs.

## Capabilities

### New Capabilities

- `plugin-policy`: installed-plugin registry, bundles, settings, management and
  product admission over Harmos activation operations.
- `plugin-sdk-boundary`: Dokime-only author APIs, shared carrier helpers and
  generator/inspection consolidation.
- `contracts-archive`: one import-closed release archive for independent protocol
  consumers, including dependency sources, licenses and compiler instructions.

### Modified Capabilities

None. Existing runnable examples and release validation guarantees remain; the
in-flight lifecycle and operation changes are amended with this boundary.

## Impact

`proto/dokime/{host,plugin,model}/v1`, `crates/dokime-runtime/src/{plugin.rs,services/}`,
`crates/dokime-operations`, `crates/dokime-storage`, `crates/dokime-packaging`,
`crates/dokime-protocol`, CLI and desktop consumers, all four SDKs and starters,
`tools/python/src/dokime_tools/build/` and release tooling, protocol export,
packaging tasks, conformance, README, `docs/project/architecture.md`, SDK/plugin guides
and tracked authoring guidance. Harmos owns the upstream runtime/tooling half.
