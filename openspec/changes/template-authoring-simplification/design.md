# Template Authoring Simplification Design

Status: proposed on 2026-11-09; deferred until the owner schedules it.

## Context

`ctx.operation<Op, Proof>(request)` requires `Op: Called<C, Proof>` where `C`
is the template type. The `requires` catalog implements `Called` for each
variant's operation type; built-ins implement it unconditionally. The serving
plugin reads the catalog to validate that every required operation is provided
or declared, and the frozen registration binds that inventory with the bundle.

## Options for the Inventory Without a Catalog

1. Macro collection. The template macro walks the template's method bodies and
   records every `ctx.operation(<Type>)` whose request type is a path. It misses
   calls made through helpers in other modules and through type aliases; it is
   cheap and compile-time.
2. Link-time registration. Each `#[harmos::operation]` already registers its
   declaration; the plugin binary's registry is the inventory of operations the
   plugin can call. Validation moves from "declared by the template" to
   "provided by the plugin or its dependencies", checked at plugin load before
   activation. Complete and free for authors; it loses the per-template view
   unless the macro also records call sites.
3. Both: option 1 for the per-template inventory shown in plugin inspection
   and used by freezing, option 2 as the ground truth validated at load.

Decision to take with the owner: option 3 unless no consumer needs the
per-template inventory, in which case option 2 alone.

## Context Aliases

The template macro emits, inside the template's module:

```rust
pub type StepCtx<'a> = dokime::execution::StepContext<'a, Template, <Template as SidecarTemplate>::Own, Parameters>;
```

and the same for the initialize, cleanup and observer stages. With `requires`
gone the first parameter still names the template so `Called` can become a
blanket implementation keyed on the inventory, or disappear; the alias stays
useful because `Own` and `Parameters` remain template-specific.

## Open Questions

- Does any consumer (plugin inspection, frozen registration, documentation
  generation) need the per-template operation inventory, or only the
  per-plugin one?
- Should the redundant built-in diagnostic survive as a lint on call sites
  that wrap a built-in in a declared operation, or disappear with the catalog?
