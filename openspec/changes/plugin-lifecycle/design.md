# Plugin Lifecycle Design

## Context

See proposal.md for motivation and the locked owner decisions. The Harmos `plugin-lifecycle`
change owns host verbs, plugin phases, internal registration accumulation,
declaration sections and the no-alias rule. Under `plugin-boundary`, Dokime owns
the public registrar and author SDKs over Harmos carrier helpers. This design records how Dokime
adopts them and what Dokime owns: product kinds, template phases and the plugin layout.
Paths are relative to the repository root; cite symbols, not lines.

Current state (landed, verified 2026-11-16):

- Template phases: Rust `Template::{materialize, initialize, cleanup}`
  (`sdk/rust/src/template.rs`), Python template methods `initialize`/`cleanup` and the
  `@materialize` decorator (`sdk/python/src/dokime/authoring.py`), C#
  `Template.{Initialize, Cleanup, Materialize, Resource}` (`sdk/csharp/src/Dokime/Authoring.cs`)
  and C++ `initialize`/`cleanup`/`materialize` fields (`sdk/cpp/include/dokime/plugin.hpp`).
- Wire: `Executions.Cleanup` with `ExecutionsCleanupRequest`, which already carries
  `termination` (`proto/dokime/plugin/v1/executions.proto`); `TestCaseTemplate.cleanup`
  (field 8, `proto/dokime/model/v1/suites.proto`); operation identity `<template>.cleanup`,
  Protected; evidence produced during the release phase carries
  `EvidenceOriginKind.cleanup` (field 4, `proto/dokime/model/v1/evidence.proto`).
- Host sequencing: `crates/dokime-runtime/src/execution.rs` sets `initialized` only when
  initialize succeeds or is absent and calls cleanup only then; a failed initialize skips it.
- Termination: the C# cleanup handler builds `Context` without termination
  (`sdk/csharp/src/Dokime/Plugin.cs`); the C++ cleanup context omits it too
  (`sdk/cpp/src/plugin.cpp`). Rust and Python expose it.
- Plugin stop: Rust `Plugin::on_stop` (`sdk/rust/src/plugin/mod.rs`) clears execution
  resources, then runs author callbacks; Python passes `on_stop(release)`
  (`sdk/python/src/dokime/_plugin.py`); C# registers `OnStop(DisposeResourcesAsync)`; C++
  registers `on_stop` on its Harmos member and exposes no author hook.
- Declarations: prompts and records are plugin-level only; Rust `Plugin::record<T>()` returns
  `Plugin<(C, T)>`, so a plugin's type grows with each record. `TestCase` references
  parameters, steps, criteria and streams only (`proto/dokime/model/v1/execution.proto`).
- Aliases: Rust `use harmos::plugin::Plugin as HarmosPlugin` (field `inner`), Python
  `from harmos.plugin import Plugin as LibraryPlugin` (field `_plugin`), C#
  `using TransportPlugin = Harmos.Plugins.Plugin;` (field `transport`), C++ member `native_`
  of type `harmos::plugin::Plugin`. The SDKs, generated sources and tests contain further
  import aliases and `#[path]`/`sys.path` overrides.
- Host verbs: `Host::shutdown` (`crates/dokime-runtime/src/host.rs`), `services::serve`
  (`crates/dokime-runtime/src/services/mod.rs`), desktop session `shutdown`
  (`crates/dokime-gui/src-tauri/src/native_session/`), SDK `serve`/`ServeAsync`/`serve`;
  `Application` implements the Harmos `Lifecycle` (`services/composition.rs`).
- OBC (`plugins/sidecars/obc/src`): root modules `analyses`, `catalog`, `device`, `evidence`,
  `measurement`, `operations`, `prompts`, `records`, `settings`, `testbench` and `suites/`;
  `lib.rs` constructs `operations::Manual::new()` while collecting declarations and calls
  `Manual::power_down` from `on_stop`; suites expose `suite()` plus `contracts()`.

Touched invariants in docs/project/architecture.md:

- Invariant 24, **Cancellation Preserves Cleanup and Evidence**: "Interrupt the active
  invocation while still running required cleanup". Purpose: cancellation never skips the
  release phase or loses attributed evidence. Preserved and widened: the release phase is
  finalize and also follows a failed initialize.
- Invariant 7's earlier assignment of plugin machinery and SDKs to the library is
  amended by `plugin-boundary`: lifecycle/accumulation machinery stays in Harmos;
  author SDKs and registrars belong to Dokime. Suites, templates, prompts and
  records remain product kinds carried in opaque declaration sections.

## Goals / Non-Goals

**Goals:**

- Template finalize after every attempted initialize, with termination in every SDK.
- One product-owned declaration model: native SDK module catalogs and the locked
  Python class composition, with the same frozen host-facing declarations.
- Template-local prompts and records without builder type growth.
- One documented plugin layout per language in the authoring skill.

**Non-Goals:**

- No host-side enforcement that a template only uses its own prompts or records.
- No per-execution registration or removal of declarations.
- No security or prerequisite phase between admission and initialize.
- No new template behavior in OBC beyond the layout and lifecycle adoption.

## Decisions

Decision sources. Owner-locked (proposal.md): paired verbs, host verbs, plugin and template
phases, finalize after a failed initialize, termination in C# and C++, registration as
admission, the registrar API, module and suite catalogs, template-owned prompts and records,
no aliases or path overrides, the canonical layout. Derived by this design; the owner may
override each before its group starts: the internal Harmos helper field named
`plugin`, Python endpoint `close` becoming
`stop`, `TERMINATION_FAILED` for a failed initialize and the per-language layout mapping.

### D1. Template Phases (Proposed)

materialize (pure, per case, no execution) → initialize → steps → criteria → finalize. The
host calls finalize once the execution reached initialize: when initialize was attempted
(succeeded, failed, timed out or cancelled during the call) or the template declares none. A
failed initialize finalizes with `TERMINATION_FAILED`; no new termination value is needed.
Wire rename with field numbers kept: `Executions.Finalize`, `ExecutionsFinalizeRequest`/
`Response`, `TestCaseTemplate.finalize` (field 8), `EvidenceOriginKind.finalize` (field 4), operation
identity `<template>.finalize`, still Protected. All four SDKs name the hook `finalize` (`Finalize` in C#) and pass a context
carrying termination. Sampling still starts after initialize and stops before finalize.

### D2. Plugin Phases and Host Verbs (Proposed)

Dokime adopts the Harmos plugin phases: the SDK's finalize disposes execution resources first
(what `on_stop` does today), then runs the author's finalize; `on_stop`/`OnStop` are deleted.
Author resources move from `on_stop` and registration into initialize and finalize. Dokime's
runtime adopts Harmos's `start`, `stop` and `run`: `Host::shutdown` → `stop`,
`services::serve` → `run` where it starts, waits and stops; desktop and CLI owners follow
where their verbs denote the same phases. Rust and C++ SDK serving entry points
become `run` (`RunAsync` in C#); Python uses `plugin.serve()`. `Application` adopts the Harmos hook renames (`cleanup` → `finalize`).

### D3. Registration of Dokime Kinds (Proposed)

```rust
// lib.rs
pub fn plugin() -> Result<dokime::Plugin> {
    dokime::Plugin::new("thermal", env!("CARGO_PKG_VERSION"))
        .register(register)?
        .initialize(device::initialize)
        .finalize(device::finalize)
}

fn register(ctx: &mut dokime::Registrar) -> Result<()> {
    ctx.register(settings::catalog())?;
    ctx.register(operations::catalog())?;
    ctx.register(prompts::catalog())?;
    ctx.register(suites::catalog())?;
    Ok(())
}
```

`dokime::Registrar` and `dokime::Registration` are product-owned public contracts,
not re-exported Harmos types (owner amendment, 2026-11-16, `plugin-boundary`). Native SDK suite,
template, prompt and record values implement the product contract; internal
translation uses Harmos accumulation helpers and opaque declaration sections.
C# uses `ctx.Register(Settings.Catalog())`; C++ uses `ctx.add(settings::catalog())`.
Python authors compose `Plugin`, `Suite` and `TestTemplate` classes listing their
contents, with plain function operations; internal SDK registration aggregates
those declarations. Helpers stay private, with qualified internal
paths and no import aliases; no public signature, attribute, error or escape hatch
exposes Harmos. The plugin value retains one type however many records register.

Local registration remains inert and freezes before serving. Harmos generic
admission then calls the product kind's admission/settings-resolution hooks before
initialize. Dokime owns installation and startup load selection; Harmos owns live
activation and lifecycle guarantees. There is no `PluginBackend` trait.

### D4. Template-Owned Prompts and Records (Proposed)

Native SDK template declarations and Python `TestTemplate` class declarations
define prompts and record contracts beside the steps that use them; registering the template aggregates them into the
plugin's frozen declarations once, refusing duplicate identities as today. The host does not
scope their use to the template. Declarations are immutable in the catalog; observer
instances, demands, resources and bench state are per execution and die with it.
`Plugin::record<T>()` type growth is removed by the registrar. Alternative rejected:
frozen-case references and host scope enforcement, which add identity and admission machinery
without an owner requirement.

### D5. Module Catalogs and Plugin Layout (Proposed)

In Rust, C# and C++, every global module exposes a catalog (using the language’s
naming convention); the Rust entry points are `settings::catalog()`, `operations::catalog()`,
`observers::catalog()`, `streams::catalog()`, `prompts::catalog()`, `records::catalog()`,
`suites::catalog()`. A suite folder's `mod.rs` exposes that suite's catalog including its
templates' prompts, records, observers and streams. `declarations()` stays the frozen
host-facing description.

```text
src/
├── main.rs  lib.rs  settings.rs  error.rs   # root holds only these
├── device/                                   # device model and handles opened in initialize
├── observers/                                # plugin-level observers only
├── operations/mod.rs                         # registers every operation
├── prompts/  records/                        # plugin-level prompts and records
├── streams/                                  # only streams genuinely shared by templates
└── suites/<suite>/mod.rs                     # suite definition and suite catalog
    └── suites/<suite>/<template>.rs          # vertical slice: its prompts, records, streams, observers
```

The authoring skill documents this layout once with per-language sections: Rust as above;
Python as the locked single project in `grpc-plugins/design.md`, with client
`src/<package>/__main__.py`, sidecar `src/<package>/plugin/__main__.py`, plugin and suite
classes in their `__init__.py`, operation-family modules and one module per template; C# as folders and namespaces of the same names with `Program.cs` and `Plugin.cs`;
C++ as `src/` and `include/` trees with the same folder names and `main.cpp`. OBC is
restructured now on today's SDK in a separate batch (not tracked here) and switches to
`register()`, `catalog()` and finalize when this change lands.

### D6. Qualified Paths Instead of Aliases (Proposed, Owner Direction)

Dokime imports the names it uses and qualifies clashes by module path;
no Rust `use x as y`, Python `import x as y`/`from x import y as z`, C# `using X = Y;`, C++
renaming `namespace`/`using` aliases, `#[path]` or `sys.path` edits. Defining a new name for a
composed type is not a rename. Generators stop emitting aliases. Each unavoidable exception is
named with its reason in the authoring skill and `docs/project/architecture.md`.

## Risks / Trade-Offs

- [Finalize after a failed initialize meets partial resources] → termination is
  `TERMINATION_FAILED`, SDK-owned resources are released by the SDK and the skill tells authors
  to tolerate absent resources.
- [Wire rename breaks stored suite definitions and running plugins] → development storage
  policy: field numbers are kept, incompatible roots are refused with the fresh-root action,
  never migrated or deleted.
- [Layout rules become ceremony for small plugins] → folders appear only when they have
  content; `streams/` only for real duplication.
- [Removing test `#[path]` sharing grows test structure] → shared support moves into ordinary
  module trees; named exceptions remain where nothing else works.

## Migration Plan

`plugin-boundary` owns the author API/helper split and coordinated cutover. Lock
operation, carrier and lifecycle contracts together and regenerate once with
`operation-boundary`; generated output remains tracked. Qualifying internal helper
names never permits author-facing Harmos values.

Adopt the Harmos revision that landed its `plugin-lifecycle` change: an integration-only local
override while it is unpublished, replaced by the published revision before final validation.
Groups land in task order on the integration branch; rollback is the previous Harmos pin and
the previous commit. No compatibility shims.
