# Operation Invocation Spec Delta (Proposed)

## MODIFIED Requirements

### Requirement: Declared Operation Requirements

A template SHALL NOT declare a `requires` catalog. The operations a template
calls SHALL form its requirement inventory from the call sites and the plugin's
linked operation declarations, validated when the plugin loads.

#### Scenario: Template Calls an Operation

- **WHEN** a template calls `ctx.operation(request)` for an operation its plugin provides or declares
- **THEN** the call compiles without a catalog entry and the operation appears in the plugin's validated inventory

#### Scenario: Catalog Still Declared

- **WHEN** a template declares a `requires` catalog
- **THEN** compilation fails with a diagnostic naming the removal and the migration

## ADDED Requirements

### Requirement: Stage Context Aliases

The template macro SHALL emit per-stage context aliases bound to the template so that helpers take `ctx: &mut StepCtx<'_>` (and the initialize, cleanup and observer equivalents) without spelling the template's own and parameter types.

#### Scenario: Step Helper

- **WHEN** a template defines a helper function taking `&mut StepCtx<'_>` and calls operations through it
- **THEN** the helper compiles and the calls are recorded in the template's inventory like direct step calls
