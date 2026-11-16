# Plugin Lifecycle

## Why

Dokime plugins and templates use lifecycle verbs that differ from Harmos and between
languages (`on_stop`, `cleanup`, `serve`), skip template cleanup when initialize fails,
omit the termination reason in the C# and C++ cleanup contexts, declare prompts and records
only at plugin level through a Rust builder whose type grows per record, import the Harmos
plugin under four different aliases, and acquire resources while declaring. The owner locked
one lifecycle vocabulary, one registration shape and one plugin layout on 2026-11-16.

## What Changes

Owner decisions locked on 2026-11-16. Status per row: **proposed** until its tasks land.
Harmos owns the generic lifecycle and registration machinery (its `plugin-lifecycle`
change); Dokime owns its product kinds and adopts that machinery.

| Decision | Direction |
| --- | --- |
| Paired verbs | `initialize` ↔ `finalize` acquire and release resources, and finalize always runs once initialize was attempted, including after a failed initialize; `start` ↔ `stop` begin and end activity; `register` is declaration and admission with no counterpart. Each placement uses only the pairs it needs. |
| Host verbs | **BREAKING.** Dokime's runtime goes new → register → start → stop; `run` is start, wait, stop. `launch`, `shutdown` and `serve` become `start`, `stop` and `run` where they denote these. |
| Plugin phases | **BREAKING.** register (admission) → initialize → finalize. Plugin `on_stop` merges into finalize: the host cancels in-flight work, then calls finalize. No plugin start or stop. |
| Template phases | **BREAKING.** materialize → initialize → steps, criteria → finalize; `cleanup` becomes `finalize` on the wire and in all four SDKs. Finalize runs even when initialize failed (today `crates/dokime-runtime/src/execution.rs` skips it). C# and C++ finalize contexts carry the termination reason like Rust and Python. |
| Registration is admission | The plugin declares streams, templates, prompts, records, operations and settings; the host only checks definitions. No run behavior, prompts or resource acquisition during registration; construction with side effects moves into initialize. Registration completes before serving, reaches the host through `Instance.Get` and is frozen per activation. |
| Registration API | **BREAKING.** `fn register(ctx: &mut Registrar) -> Result<()>` with `ctx.register(x::catalog())`; typed registrations accumulate descriptors, removing `Plugin<(C, T)>` growth. C++ calls `add` (register is a reserved keyword). Dokime kinds register through Harmos declaration sections. |
| Module catalogs | Every global module exposes `catalog()` (settings, operations, observers, streams, prompts, records, suites); a suite folder's `mod.rs` exposes its suite catalog including its templates' prompts, records, observers and streams. `declarations()` stays the frozen host-facing description. |
| Template-owned declarations | `Template::prompt()` and `Template::record()` in all four SDKs aggregate into the plugin's frozen catalog; no host-side template-scope enforcement. Declarations are immutable; observer instances, demands, resources and bench state die with the execution. |
| No aliases or path overrides | Dokime refers to the Harmos plugin by qualified path (`harmos::plugin::Plugin`, `harmos.plugin.Plugin`, `Harmos.Plugins.Plugin`, `harmos::plugin::Plugin` in C++); no import aliases or path overrides anywhere except named unavoidable exceptions. |
| Canonical plugin layout | The plugin authoring skill documents one layout per language: root `main.rs`, `lib.rs`, `settings.rs`, `error.rs`; folders `device/`, `observers/`, `operations/`, `prompts/`, `records/`, `streams/`, `suites/<suite>/mod.rs` with one file per template. |

- Rename `Executions.Cleanup`, its request and response and the template definition's
  `cleanup` operation reference to finalize; keep the termination field.
- Security checks and prerequisite UI after admission and before initialize are a possible
  later phase, not part of this change.

Supersedes, in the `grpc-plugins` change: "Cleanup remains required after successful
initialization" (finalize now also follows a failed initialize), the `<template>.cleanup`
operation identity and the `host-executions` delta's cleanup wording; in the
`operation-grammar` change, the `Executions.Initialize/Cleanup` row of the visibility mapping
(now Initialize/Finalize, still Protected).

## Capabilities

### New Capabilities

- `template-lifecycle`: template phases, finalize guarantees and termination reasons in the
  host and all four SDKs.
- `plugin-composition`: plugin registration of Dokime kinds, module and suite catalogs,
  template-owned prompts and records, plugin finalize and the lifetime of declarations
  versus execution instances.

### Modified Capabilities

None. `plugin-authoring` (Canonical Runnable Examples) keeps its requirement: examples keep
their behavior, including release of resources, under the new names.

## Impact

`proto/dokime/plugin/v1/executions.proto`, `proto/dokime/model/v1/suites.proto`,
`crates/dokime-runtime` (execution sequencing, host verbs, Harmos lifecycle adoption),
`crates/dokime-cli`, `crates/dokime-gui/src-tauri`, the four product SDKs and their
generators, `tools/python`, the sample plugins under `plugins/`, stories, the authoring
skill (`.claude/skills`, `.agents/skills`, `.github/skills`), `docs/architecture.md`,
`docs/execution-host.md` and the SDK guides. Depends on the Harmos `plugin-lifecycle`
revision.
