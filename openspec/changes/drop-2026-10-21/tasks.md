# Feature Drop Tasks

## Crate Layout And Catalogs

- [x] 77. Rewrite the remote examples as linear learning exercises with explicit repeated calls.

- [x] Expose `plugin-sidecar`, migrate all plugin consumers and generated starter dependencies, audit macro facade paths and update feature guidance.
- [x] Pass repository check, entry/CLI/macro nextest, OBC bundle build and fresh starter Rust check for the plugin feature.

- [x] 76. Reject duplicate suite, template and record stream identifiers across a plugin's catalogs at compile time.

- [x] 75. Make dokime the entry library and dokime-cli the CLI; migrate shipped interfaces, plugins, fixtures, templates, tooling and guidance.
- [x] Resolve terminal ownership at `crates/dokime/src/terminal/`: keep shared output in the entry library for the shipped presentation consumers.
- [x] Repair the candidate-wheel tutorial lock, add explicit relock authoring, prove dev Python Build/Package and compare the release wheel, and restore the editable tooling binding with prepare.

## Agreed Inventory

- [ ] 1. Verify path-based plugin bundle installation without serial JSON chunks; native installation walkthrough proof remains required beyond the single-invoke adapter tests.
- [x] 2. Format plot X cursor timestamps consistently with axes. Proof: `plots/nominal.test.ts` / `Series Legend`.
- [ ] 3. Show statistics headers and units without clipping.
- [x] 4. Distinguish queued execution cancel from active stop. Proof: `cycle/nominal.test.ts` / `Execution Controls`.
- [ ] 5. Align parameter labels, values and adjacent units across views.
- [ ] 6. Keep prompt response controls only on their execution.
- [ ] 7. Harmonize cycle/execution headers, controls, actions and records.
- [ ] 8. Label Export Evidence and Download Report consistently.
- [ ] 9. Preserve every planned execution in frozen reports.
- [ ] 10. Reuse parameter labels, units and layout in reports; verify step order.
- [ ] 11. Render prompt asked/answered plot markers with title hover in live/report views.
- [ ] 12. Downsample full series per viewport preserving envelopes and avoid duplicate titles.
- [ ] 13. Pad and scroll console content with newest-line following.
- [ ] 14. Verify Dokime README CI, crate, PyPI, release and license badges.
- [ ] 15. Preserve four surface-aware logo variants, native icons and themed README logo.
- [ ] 16. Place the Prompt card separately beneath Control.
- [ ] 17. Derive cycle status independently of an execution's stop outcome.
- [ ] 18. Prevent cycle/history status pill clipping.
- [ ] 19. Persist a visible destination cycle indicator and switcher.
- [ ] 20. Verify README CLI reference against actual commands and shipped examples.
- [ ] 21. Complete seal/report persistence improvements and measure materialization.
- [ ] 22. Verify initial plot acknowledgement/keepalive, concurrency and waiting state.
- [ ] 23. Show finalization immediately and preserve subsecond duration values.
- [ ] 24. Remove continue-after-failure, share viewer settings and prevent duplicate continuation.
- [ ] 25. Harmonize suite/template labels.
- [ ] 26. Verify active-cycle picker and shell destination behavior.
- [ ] 27. Repair live histograms with report parity.
- [ ] 28. Show centered diagnostics in an erroring plot overlay.
- [ ] 29. Harmonize progress row styling.
- [ ] 30. Preserve FIFO prompts as self-contained cards.
- [ ] 31. Show empty pending, ticking running and measured completed step durations.
- [ ] 32. Complete narrow operator-note persistence and measure latency.
- [ ] 33. Fix access-key dialog overflow.
- [ ] 34. Apply remote settings live, reflect actual status and generate missing TLS certificates.
- [ ] 35. Use a shared two-column settings layout and repair affected dialogs.
- [ ] 36. Page remote session history in groups of twenty.
- [ ] 37. Answer single-boolean prompts through No/Yes footer actions.
- [ ] 38. Share the page shell across Library, Cycles, History and Explore.
- [ ] 39. Verify fast drag-and-drop bundle install.
- [ ] 40. Persist runtime branding packs for next boot and retain compiled identity boundaries.
- [ ] 41. Verify the Linux WebView response-stall fix and operator override behavior.
- [ ] 42. Complete orderly close, bounded force quit and measured idle/running exit.
- [ ] 43. Complete and run six system benchmark scenarios with native evidence.
- [ ] 44. Verify optimized dev workspace profiles, package profile reporting and build costs.
- [ ] 45. Complete bounded record tails/incremental updates, stream invalidation, document suppression, aggregate detail and virtualization; measure idle invokes.
- [ ] 46. Complete native report delivery and verify transfer sizes, plot budgets, deadlines and client flow control; measure export throughput.
- [ ] 47. Complete telemetry/artifact durability batching, codec reuse, narrow state transitions and event-driven prompt waits with behavior tests and measurements; retain approved per-attempt preparation and verify cancellation/terminal evidence through the real public path.
- [ ] 48. Verify batched changed-plot queries and telemetry-only refresh semantics.
- [x] 49. Enlarge sidebar logo to about one sixth window height, center wordmark beneath and separate navigation.

## Watcher Extension

- [ ] 55. Expose observable capture-armed readiness after the watcher baseline across hosted, harness and streaming APIs, including empty baselines.
- [ ] 56. Preserve uniquely named single-file rotation segments and correct simultaneous-file limits without cumulative replacement rejection.
- [ ] 57. Fail explicitly on unsupported FTP resume when extraction requires it, preserving evidence and actionable gap/error; retain signalled artifact-only fallback.
- [ ] 58. Add opt-in bounded waiting for absent sources with cancellation/deadline; preserve strict defaults and immediate permission/invalid-path failures.
- [ ] 59. Add independent All/Latest-at-start/New-after-baseline selection with deterministic modification-time/path ordering and from-zero new-file capture.
- [ ] 60. Prove watcher acceptance through meaningful deterministic tests and migrate hosted/harness/server/client bindings, authoring guidance, examples and README.
- [x] 61. Resolve the FTP missing-root versus permission-denied ambiguity without silently narrowing absent-source waiting.

## Consolidation And Acceptance

- [ ] 50. Adopt all outstanding source slices and regenerate every affected client/binding from the combined tree.
- [ ] 51. Account for source, tests, configuration, examples, README and superseded paths; declare source readiness only after all writers hand off.
- [ ] 52. Run relevant local gates, development build, required native system walkthrough and affected lanes; repair every required failure.
- [ ] 53. Verify published upstream pins and remote-streamline integration, complete required provider checks and authorized review into dev.
- [ ] 54. Finish the change after all acceptance and evidence tasks are satisfied.

## Organization Packs (absorbed from the organization-packs change on 2026-10-21)
- [ ] 62. Replace branding-pack layout/naming with organization/organization.toml across assets, generators, imports, UI, commands, examples and guidance; preserve compiled identity and restart activation.
- [ ] 63. Preserve installed legacy pack data through safe migration or explicit import guidance; retain the last valid organization/update configuration on invalid input with a visible error.
- [ ] 64. Implement one disabled/manual/automatic-check update policy with direct manifest, GitHub Releases and GitLab Releases sources; installation stays explicit and signing trust stays compiled.
- [ ] 65. Add named GitLab hosts/projects/OAuth connections and authenticated sources with host/account/client-scoped credentials, origin-safe forwarding and no secrets in packs.
- [ ] 66. Implement the version-1 plugin release feed contract in design.md, organization-declared sources/links and authoritative backend target identity; check advertised package/version against inspected bytes, enforce host compatibility/target/digest and reuse Dokime staging/consent/install/restart. Missing feed and target source blocks source completion.
- [ ] 67. Migrate all updater/organization consumers, generated bindings, tests, README and shipped examples; remove superseded GitLab-only updater coupling.
- [ ] 68. Validate switching, disabled-network behavior, public/auth/missing sources, target selection, signature rejection, invalid-pack retention, credential separation and plugin lifecycle with focused tests and native acceptance.

## Shared Secret Storage (absorbed from the shared-secret-storage change on 2026-10-21)

- [ ] 69. Inventory secret persistence/keyring keys, plugin bindings and deployed consumers; lock safe adoption and disjoint ownership before the naming cutover.
- [ ] 70. Replace Dokime-owned credential terminology across public declarations, operation IDs, modules, fields, settings, FTP, clients, GUI, tests, examples and guidance; preserve foreign APIs and stored values.
- [ ] 71. Add host-owned shared named-secret lookup for all plugins with clear missing/deleted/replaced behavior and existing requirement preflight, without plugin ACLs or org-pack bindings.
- [ ] 72. Add Secret Storage UI with target-host attribution, freely managed names/labels/descriptions, masked values and honest known-usage indication.
- [ ] 73. Add bounded password-encrypted authenticated selected-secret export/import, conflict preview and explicit replace/skip with atomic validated import and no writes on password/tamper failure.
- [ ] 74. Verify multi-plugin lookup, lifecycle changes, host attribution, encryption/conflicts/limits/tamper/traversal/atomicity, redaction, namespace separation, plugin settings/FTP and regenerated client coherence.
