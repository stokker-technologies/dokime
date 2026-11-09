# Template Authoring Simplification Tasks

Status: proposed; deferred until the owner schedules it.

## 1. Contract

- [ ] 1.1 Decide the inventory source (design.md options) with the owner and record it; lock the source inventory before editing.

## 2. Macros and Runtime

- [ ] 2.1 Emit per-stage context aliases from the template macro and document a helper signature in the authoring guide; prove with a starter template helper whose context type is one line.
- [ ] 2.2 Replace the `requires` catalog with the decided inventory source while plugin load validation and frozen registration keep equivalent inputs; a template that still declares `requires` fails to compile with a diagnostic naming the removal.
- [ ] 2.3 Migrate the generated starter, `plugins/sidecars/obc`, tests and the authoring skill; remove or recast the redundant built-in diagnostic per the decision.

## 3. Validation

- [ ] 3.1 Focused macro, runtime execution and plugin tests; the system stories that install and run the OBC plugin; `mise run lint:rust`; project check.
- [ ] 3.2 Tell the private plugin owner the migration (delete the catalog, adopt the aliases) once the change is on a published Nightly.
