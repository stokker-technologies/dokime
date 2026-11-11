# Template Authoring Simplification

Status: proposed on 2026-11-09; deferred until the owner schedules it.

## Why

Two authoring costs showed up while writing a plugin template this week.

- Every template keeps a `requires` catalog, an enum that lists each plugin
  operation it calls. The enum exists so that `ctx.operation(request)` can
  prove at compile time that the template declared the operation (the
  `Called<Template, _>` bound), so the serving plugin can validate and freeze
  the inventory of operations a template depends on, and so redundant
  built-in entries are rejected. In practice the catalog repeats the import
  list: every call site already names the operation type, and a missing entry
  surfaces as a trait-bound error about `Called`, not as a missing variant.
- A step helper that takes the context must spell the context's full type,
  `StepContext<'_, Template, <Template as SidecarTemplate>::Own, Parameters>`,
  because the first type parameter carries the `requires` proof. Authors fall
  back to repeating code inside one step, and the authoring guide documents no
  helper signature.

## What Changes

- Remove the `requires` catalog from template declarations. The operations a
  template calls become its requirement inventory without a second
  declaration: the macro records every `ctx.operation` call type it can see,
  and the runtime completes the inventory at plugin load from the operations
  the plugin binary links, so frozen registration and load-time validation
  keep their inputs.
- Make the redundant built-in diagnostic unnecessary: built-ins and declared
  operations go through one path.
- Give step, initialize, cleanup and observer helpers a one-line signature:
  the template macro emits a context alias per stage (`StepCtx<'_>`,
  `InitCtx<'_>`, `CleanupCtx<'_>`, `ObserverCtx<'_>`) bound to the template,
  and the authoring guide shows a helper taking `ctx: &mut StepCtx<'_>`.
- Update the generated starter, the OBC plugin, the authoring skill and the
  plugin and product guides; the private plugin migrates in its own repository
  once a Nightly carries the change.

## Impact

- Affected: `dokime-macros` template and operation macros, the `Called` proof
  in the runtime execution context, plugin load-time validation, the starter
  template, `plugins/sidecars/obc`, authoring documentation.
- Clean break for templates: a template that still declares `requires` fails
  to compile with a diagnostic naming the removal and the migration; no
  compatibility shim.
- Not affected: operation declarations and policies, prompts, streams, records.
