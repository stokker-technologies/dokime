# Plugin Lifecycle Tasks

The Harmos `plugin-lifecycle` change lands first. Dokime adopts the revision that landed it:
an integration-only local override while that revision is unpublished, replaced by the
published pin before final validation (refresh locks with `mise run prepare:locks`). Each group
keeps the build green, commits with Conventional Commits and proves only changed behavior with
focused `mise run test`, `mise run build` and `mise run generate` invocations. Named stories
run with `mise run test -- --package dokime-runtime --target system --story <name>`. Full lanes
run only after group 8. No compatibility code; incompatible roots are refused with the
fresh-root action.

## 1. Inventory and Contract

- [ ] 1.1 Lock the source inventory against design.md Context: every template `cleanup` symbol, wire name and operation identity; every plugin `on_stop`/`OnStop` use; every Dokime `launch`, `shutdown` and `serve` symbol and whether it denotes start, stop or run; every plugin-level prompt and record registration; every import alias and path override in the repository; every module entry point in sample plugins, examples and fixtures. Accept when each item has an owning task below; record surprises in design.md before editing.
- [ ] 1.2 Quote invariant 24 (Cancellation Preserves Cleanup and Evidence) and invariant 7 in `docs/project/architecture.md` with their purposes, update them and the `Executions` mapping and message rows of `docs/guides/execution-host.md`, and record the paired-verb rule, host verbs, plugin and template phases (finalize after attempted initialize, termination in every SDK), Dokime kinds on the library registrar, template-owned prompts and records and the no-alias rule as locked direction with proposed status. Prove with `mise run check -- --repository`.
- [x] 1.3 Replace library-registrar re-exports and SDK ownership with the `plugin-boundary` decision; review both boundary changes and pass strict validation. Documentation only; implementation remains open.

## 2. Adopt the Harmos Revision

- [ ] 2.1 Adopt the Harmos revision that landed `plugin-lifecycle`; rename `Application`'s `cleanup` to `finalize`, `Host::shutdown` to `stop`, `services::serve` to `run` and the desktop and CLI owners' verbs where they denote start, stop or run (design.md D2), with no other behavior change. Prove with `mise run build -- --package dokime-runtime --check` and stories `manager_lifecycles` and `execution_isolation`.

## 3. Template Finalize

- [ ] 3.1 Rename Executions cleanup RPC/messages, template reference (field 8), evidence origin (field 4) and operation identity to finalize, retaining Protected visibility. Lock these declarations with both boundary changes for `plugin-boundary` task 6.1's single regeneration; prove resulting generated consumers and protocol through focused project tasks.
- [ ] 3.2 Make `crates/dokime-runtime/src/execution.rs` call finalize once initialize was attempted or is absent, including failed, timed-out and cancelled initialize, with `TERMINATION_FAILED` for a failed initialize. Rename the story `cancelling_a_thermal_step_runs_cleanup` to `cancelling_a_thermal_step_runs_finalize`. Prove with a new story `template_finalizes_after_failed_initialize` and stories `cancelling_a_thermal_step_runs_finalize`, `execution_outcomes` and `client_controls_execution_lifecycle`.
- [ ] 3.3 Rename the template hook to `finalize` (`Finalize` in C#) in the Rust, Python, C# and C++ SDKs and give the C# and C++ finalize contexts the termination reason. Prove with each SDK's focused tests for finalize termination and failed initialize, and `mise run build:host-fixtures -- --language rust`, then python, csharp, cpp.

## 4. Plugin Phases in the SDKs

- [ ] 4.1 Replace `on_stop`/`OnStop` in the four SDKs with the Harmos initialize and finalize hooks: the SDK's finalize releases remaining execution resources, then runs the author's finalize; expose an author finalize in C++. Prove with each SDK's focused tests for a plugin stopped during an execution, and `mise run build:cli-fixtures`.
- [ ] 4.2 Rename Rust and C++ SDK serving entry points to `run` (`RunAsync` in C#) and the Python endpoint `close` to `stop`, matching the Harmos SDKs; keep the locked Python sidecar entry `plugin.serve()`. Prove with each SDK's focused fixture build.

## 5. Registration and Catalogs

- [ ] 5.1 Implement Dokime-owned public `Registrar` and `Registration` contracts for native SDK suites/templates/prompts/records and generic author declarations, and internally aggregate Python class declarations; translate internally through Harmos carrier helpers without type leaks or aliases. Remove record-driven plugin type growth and superseded builder chains. Coordinate with `plugin-boundary` and prove ten typed records retain one plugin type plus installed author consumers through focused project tasks; update SDK guides.
- [ ] 5.2 Add native SDK template prompt/record APIs and Python `TestTemplate` class declarations, aggregating into the frozen declarations with duplicate refusal and no host scope enforcement. Prove with a new story `template_records_admit_against_frozen_contracts` and each SDK's focused tests.
- [ ] 5.3 Move every sample plugin, example and fixture (`sdk/*/examples`, `sdk/*/tests/fixtures`, `dokime new` templates) to native SDK root `register` functions and module catalogs, or the locked Python `Plugin`/`Suite`/`TestTemplate` class composition and function operations, with side-effecting construction moved into initialize. Prove with `mise run build:host-fixtures -- --language rust`, then python, csharp, cpp, `mise run build:cli-fixtures` and the `dokime new` generation tests for each language.
- [ ] 5.4 Switch OBC (`plugins/sidecars/obc`, already restructured to the canonical layout in its separate batch) to a root `register()` function, module and suite `catalog()` entry points, template-owned prompts and records and initialize/finalize, moving `operations::Manual` construction out of declaration into initialize and `power_down` from `on_stop` into finalize. Prove with `mise run build -- --package plugin-obc` and story `client_runs_packaged_obc_plugin`.

## 6. Qualified Paths

- [ ] 6.1 Remove every import alias and path override in `sdk/` (sources, generators and their templates, generated sources, tests, examples), referring to the Harmos plugin as `harmos::plugin::Plugin`, `harmos.plugin.Plugin`, `Harmos.Plugins.Plugin` and `harmos::plugin::Plugin` (C++), holding it in a field named `plugin` (`_plugin` in Python, `plugin_` in C++). Accept when the inventory shows only named exceptions; prove with `mise run generate -- --protocol`, `mise run check:generated` and each SDK's focused fixture build.
- [ ] 6.2 Remove the remaining aliases and path overrides in `crates/`, `tools/` and `plugins/`, moving shared test support into ordinary module trees where `#[path]` or `sys.path` shares it today. Prove with focused builds of each touched package and the touched stories.

## 7. Authoring Guidance

- [ ] 7.1 Document in the plugin authoring skill (`references/plugins.md` and `references/suites.md` under every tracked copy in `.claude/skills/dokime-authoring`, `.agents/skills/dokime-authoring` and `.github/skills/dokime-authoring`) the plugin and template phases, inert registration, native SDK register functions and module catalogs, the locked Python class composition and `plugin.serve()`, template-owned prompts and records, the canonical plugin layout with per-language sections (design.md D5), and the no-alias, no-path-override rule with each named exception. Accept when the three copies are identical; prove with `mise run check -- --repository`.
- [ ] 7.2 Update `docs/sdks/rust.md`, `sdk/python/README.md`, `docs/sdks/python.md`, `docs/sdks/csharp.md`, `docs/sdks/cpp.md`, `docs/guides/plugins.md` and `docs/plugin-obc.md` examples and lifecycle descriptions to the new names, register phase, native SDK catalogs and locked Python project/class surface. Prove with `mise run check -- --repository`.

## 8. Integration

- [ ] 8.1 Replace the integration-only Harmos override with the published revision, run `openspec validate plugin-lifecycle --strict`, the format task and the Rust, Python, C# and C++ language checks on the combined tree, and confirm every source item above is closed; report pending native platform or hosted proof explicitly.

## Workflow Follow-Up

- OBC is restructured to the canonical layout on today's SDK in a separate batch, not tracked here; task 5.4 switches it to this change's API.
- A security and prerequisite phase between admission and initialize is a possible later change.
- Full native Linux and Windows validation and hosted CI follow source closure under the delivery authorization.
- Archive this change after delivery and fill each new capability's Purpose in the main specs.
