# Plugin Boundary Design

## Context

See proposal.md for motivation. The locked owner decision of 2026-11-16 follows
comparison of go-plugin, the Grafana plugin SDK, Kubernetes CSI/CRI, Wasmtime
components and OSGi. Harmos retains plugin runtime machinery; Dokime owns policy
and everything an author touches. No `PluginBackend` trait is introduced.

Verified source inventory in this worktree:

| Concern | Evidence |
| --- | --- |
| Duplicated managed inspection | `crates/dokime-runtime/src/plugin.rs:95` starts a process; `:107` spawns it, `:138` closes it with Stop/kill/reap, `:186` describes an endpoint and `:207` connects. |
| Parallel renderers | `tools/python/src/dokime_tools/build/render_resources.py:80` builds descriptor tables; `:315`, `:700`, `:1083` and `:1761` render languages. This richer implementation must contribute its fixes to Harmos. |
| Duplicate archive builder | `crates/dokime-packaging/src/bundle/mod.rs:64` creates an archive, while `:28` imports the library verifier/manifest. Both must become one product owner. |
| Author boundary leaks | `sdk/rust/src/plugin/mod.rs:222` exposes `into_harmos`; `sdk/csharp/src/Dokime/Plugin.cs:96` returns a Harmos plugin from Build; `sdk/python/examples/plugin/main.py:20` imports Harmos; `sdk/cpp/include/dokime/plugin.hpp:126` names Harmos operation types. |
| Contracts archive gap | `tools/python/src/dokime_tools/release/run.py:220` selects the proto archive; `tools/protocol/src/protocol.rs:157` embeds repository protos, while `:111` separately adds compiler includes during generation. |

Invariant 15 currently says "language SDKs stay in the library". Its purpose is
to avoid reimplementing transport per host. Replace that ownership claim with
product author APIs over shared carrier helpers, preserving the purpose.
Invariant 7's assignment of SDKs to the library and its PluginEntry/settings
composition claims, invariant 6's installed-entry ownership, and the Final Library
Assembly tables change with it. The related `operation-boundary` D7 and
`plugin-lifecycle` registrar/SDK clauses are amended here. Older research that
reported those changes absent predates this tree; it is not current evidence.

## Goals / Non-Goals

**Goals:** authors install and use Dokime only in every supported language;
installation and persisted policy have one Dokime owner; live activations retain
one Harmos owner; generation and process inspection are reused, not forked.

**Non-Goals:** no copied carrier runtime, author-facing Harmos types, aliases,
compatibility shims, migration, silent root reset or second bundle implementation.
Product execution/template phases remain distinct from plugin activation phases.

## Decisions

### D1. Ownership and Admission

Harmos owns the internal plugin manager and register → admission → initialize →
finalize machinery, descriptor closure/identity/fingerprint/declaration-digest
validation, routing, process/Wasm hosting, transport, reconnect/fencing, windows,
sampling, deadlines and cancellation. Its minimal carrier carries connection
identity/revision, admission descriptors, invocation correlation and opaque
payloads, cancellation, host-call frames, windows/samples and opaque product
sections. Four-language carrier helpers, shared generation, bounded inspection
and generic conformance stay upstream.

Dokime owns `proto/dokime/plugin/v1`, its public author API, starters, installed
registry, installation, bundle format/build/verification/manifests/artifact
digests, distribution, admission policy, persisted settings and plugin management
operations and their SDK projections. Descriptor/declaration digests remain
generic; artifact digests move with installed-byte verification.

Proposed shape, not landed source; through `operation-boundary`'s typed facade:

```rust
setup.plugin_kind("dokime.grpc",
    grpc::Kind::<DokimeContract>::new().admit(dokime::admit));
let host = runtime.for_call(&ctx);
let plugin = host.plugins().load(Load {
    kind, placement: Managed { command }, settings,
}).await?;
let binding = host.plugins().bind(plugin, scope, clock).await?;
let demand = host.plugins().start_samples(binding, streams).await?;
```

The kind registers product admission and settings resolution. Generic validation
precedes those callbacks; Dokime validates suites, templates, prompts, records,
secrets and other product sections, resolves the effective settings, then Harmos
initializes. Callbacks inherit the load operation's deadline and cancellation,
not a second policy authority. Rejection publishes no activation and Harmos
settles resources it started. Managed commands, remote endpoints and Wasm bytes
come from Dokime selection; all returned activation, binding and demand handles
are opaque and fenced by generation. No callback discovers installed entries for
Harmos: Dokime's registry calls load for each installed plugin at startup.

### D2. Product Installation, Registry and Settings

Move Harmos registry/install/settings and management operations into Dokime,
including the whole durable behavior, not only wire declarations. Product
management is policy code, not a thin wrapper around a Harmos installation
service. It uses Harmos storage operations through the facade and asks the runtime
to load, bind, sample, finalize and settle activations. The backend remains the
physical storage owner; no direct Store bypass returns.

Preserve conditional revisions, atomic publication and delete/settings changes,
original mutation receipts on replay, verified inventory and artifact digests,
path/extraction bounds, staged replacement rollback and the rule that installation
reuse/deletion waits for process settlement. If an operation spans filesystem and
registry state, preserve its durable recovery/cleanup owner rather than claiming
a cross-owner transaction. A failed candidate never replaces the admitted entry.
Secret values and authorization stay in the existing product secret owner.
Keep the development baseline v1; refuse incompatible roots with an action to
choose a fresh root and leave existing data intact. No migration or reset.

Bring the library's bundle format, verifier/extractor and builder into
`crates/dokime-packaging` and the product installation owner; consolidate with
`crates/dokime-packaging/src/bundle/mod.rs`, then remove its duplicate builder and
the upstream bundle paths. There is one resulting format/build implementation,
with product description additions kept. Distribution and release tooling stay
Dokime-owned.

### D3. Shared Generation and Inspection

Move the reusable descriptor table/renderers and fixes from
`tools/python/src/dokime_tools/build/{render_resources,resource_roots}.py` and SDK
generator adaptations into Harmos tooling. Preserve optional presence, nested
messages, keyword escaping, enums/repeated values, typed client/context calls,
request/response mapping and call options. Product tasks retain their descriptors,
namespace/output inputs and packaging, not a parallel renderer implementation.

Replace the entire spawn/connect/Get/Stop probe in
`crates/dokime-runtime/src/plugin.rs` with Harmos bounded inspection and migrate
all CLI/bundle/diagnostic callers. Delete superseded probe types, teardown
constants and tests when no consumer remains; retain placement configuration at
its actual product owner. Inspection returns frozen generic-validated metadata,
does not initialize, sample, install or persist, and always settles its owned
process under existing operation policies. Remote inspection detaches without
terminating an unowned process. Dokime interprets the returned product sections.

### D4. Only Dokime Is Author-Facing

Rust, Python, C# and C++ expose Dokime plugin declarations, operations, policies,
visibility, failures, contexts, windows, lifecycle hooks and generated product
host calls. Rust, C# and C++ expose the product registrar/registration API; Python
authors use the locked `dokime_sdk` class composition and plain function operations.
Its `operations` and `templates` submodules own separate contexts; the root exports none. Harmos carrier helpers remain internal implementation
dependencies, with automatic package/native dependency resolution. Authors need
no Harmos imports, attributes, namespaces, exception types, transport escape
hatches, package-install instructions or compiler paths.

The earlier proposal to re-export Harmos `Registrar`/`Registration` is superseded:
Dokime owns the native SDK public registrar and typed registration contract;
Python class declarations are aggregated by the SDK. Both translate to generic
helpers internally. Qualification of clashing Harmos names is internal;
no renaming import/type aliases conceal ownership. Provide real product wrappers
where a language cannot hide helper types through an ordinary same-name public
export. Migrate derives, errors, return types, examples, starters, docs and every
tracked authoring-skill copy in the same cutover. The lifecycle registrar still
accumulates typed declarations without plugin-type growth; all phase guarantees
remain those of `plugin-lifecycle`.

Helpers retain time, own-invocation and admitted discovery facilities over opaque
carrier host-call frames; own settings use the product kind's resolver. Dokime
SDKs expose product values without requiring new product wrappers for these
facilities. Harmos supplies clock, routing and cancellation mechanics; product API
DTOs/policy do not enlarge the carrier. Generic payload reuse applies only where the
payload still belongs to Harmos, not product plugin administration.

### D5. Self-Contained Contracts Archive

One archive per release contains Dokime protos, the exact Harmos carrier dependency
closure, every imported `google/*` proto recursively, applicable licenses and
protoc/Buf instructions and configuration. Include all imports of other shipped
product host/model protos too. SDK installation remains the author entry point;
the archive supports independent protocol consumers without a second Harmos
download or an undocumented include tree.

Packaging resolves imports against archive members only, rejects missing or
conflicting definitions and exercises the supplied compiler instructions with
the intended compiler installed and no external proto include sources. The
archive and embedded export share the same source inventory and upstream pin.
Preserve tracked generated output pending the owner's separate tracking decision.

### D6. Coordinated Cutover and Conformance

Follow the six ordered task groups together with Harmos. Lock carrier, facade,
lifecycle and product declarations first, then regenerate all languages once for
all three changes. Preserve generic transport/lifecycle/cancellation/window proof
in Harmos's conformance kit with a fixture product contract; Dokime owns proof of
its SDK boundary, admission, installation, settings, suites/templates, evidence,
secrets and execution resource lifecycle. Product tests reuse the generic kit
rather than clone it. Native Windows and Linux proof remain required wherever
configured; no docs-only validation claims to establish them.

## Risks / Trade-Offs

- [C# and C++ expose helper types through signatures] → explicit product wrappers
  and installed-consumer fixtures, no aliases or author-facing qualification.
- [Registry move loses atomicity or receipts] → transfer durable behavior with
  deterministic fault/replay tests and fresh-root refusals.
- [Product fixes disappear during generator consolidation] → compare both trees,
  preserve descriptor behavior before deleting the product renderers.
- [Contracts archive depends on compiler include trees] → verify recursive import
  closure against archive members and exercise its compiler instructions.

## Migration Plan

Ownership amendments precede shared tooling/inspection, runtime kind separation,
product persistence/bundles, four author SDKs/archive and coordinated regeneration.
Use integration-only upstream overrides during implementation, then published
immutable revisions before final validation. This is a clean replacement; rollback
uses prior source and a compatible root, never silent rewriting of existing data.
