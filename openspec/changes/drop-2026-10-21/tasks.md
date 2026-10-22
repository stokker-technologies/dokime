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

- [x] 1. Verify path-based plugin bundle installation without serial JSON chunks. Proof: `native-desktop.mjs / plugin-install` (native picker, single path-staging invocation, digest consent, saved install, restart activation and catalog).
- [x] 2. Format plot X cursor timestamps consistently with axes. Proof: `plots/nominal.test.ts` / `Series Legend`.
- [x] 3. Show statistics headers and units without clipping. Proof: `native-desktop.mjs / cycle` (mounted trace statistics header bounds).
- [x] 4. Distinguish queued execution cancel from active stop. Proof: `cycle/nominal.test.ts` / `Execution Controls`.
- [x] 5. Align parameter labels, values and adjacent units across views. Proof: `execution/nominal.test.ts / Parameter Rows`; `native-desktop.mjs / cycle, execution-stop, report` (mounted column alignment and unclipped content).
- [x] 6. Keep prompt response controls only on their execution. Proof: `execution/nominal.test.ts / Page Sections`; `native-desktop.mjs / cycle` (execution response panel and cycle links).
- [x] 7. Harmonize cycle/execution headers, controls, actions and records. Proof: `cycle/nominal.test.ts / Node Menus, Execution Controls, Focus Keeps Controls`; `native-desktop.mjs / cycle`.
- [x] 8. Label Export Evidence and Download Report consistently. Proof: `native-desktop.mjs / cycle` (cycle and execution action labels); `cycle/nominal.test.ts / Node Menus`.
- [x] 9. Preserve every planned execution in frozen reports. Proof: `domains::documents::report_payloads::cycle_report_planned`; `native-desktop.mjs / report` (all planned members).
- [x] 10. Reuse parameter labels, units and layout in reports; verify step order. Proof: `domains::documents::report_payloads::template_parameter_units`, `criterion_step_order`; `execution/nominal.test.ts / Parameter Rows`; `native-desktop.mjs / report` (mounted parameter columns).
- [x] 11. Render prompt asked/answered plot markers with title hover in live/report views. Proof: `plots/nominal.test.ts / Prompt Markers`; `native-desktop.mjs / cycle, report` (asked and answered hover titles).
- [x] 12. Downsample full series per viewport preserving envelopes and avoid duplicate titles. Proof: `plots/nominal.test.ts / LTTB Reduction, Window Resampling`.
- [x] 13. Pad and scroll console content with newest-line following. Proof: `console/nominal.test.ts / Tail Follow`; `native-desktop.mjs / cycle` (padding, scroll release and Jump to Latest).
- [x] 14. Verify Dokime README CI, crate, PyPI, release and license badges. Proof: `README badge endpoint audit` (workflow API, registry/release absence, four HTTP-200 status badges and Apache-2.0 license).
- [x] 15. Preserve four surface-aware logo variants, native icons and themed README logo. Proof: `domains::installed_plugins::brand::runtime_appearance_brand`, `organization_toml_types`; `organization.test.ts / Brand Asset Lifecycle`; `test_organization_overlay_preserves_native_window_policy`; native icon digest audit (17 outputs); README picture source audit.
- [x] 16. Place the Prompt card separately beneath Control. Proof: `native-desktop.mjs / cycle` (mounted prompt queue below Control).
- [x] 17. Derive cycle status independently of an execution's stop outcome. Proof: `cycle/nominal.test.ts / Cycle State`; `native-desktop.mjs / execution-stop`.
- [x] 18. Prevent cycle/history status pill clipping. Proof: `cycle/nominal.test.ts / Status Pill`; `native-desktop.mjs / cycles-home, history` (mounted badge bounds).
- [x] 19. Persist a visible destination cycle indicator and switcher. Proof: `native-desktop.mjs / active-cycle`; `cycle/nominal.test.ts / Active Containers Persist`.
- [x] 20. Verify README CLI reference against actual commands and shipped examples. Proof: `CLI help audit` (37 built command paths); `test_remote_examples.py` (8 tests); shipped shell command/flag audit.
- [x] 21. Complete seal/report persistence improvements and measure materialization. Proof: `domains::documents::reports` (5 tests), `domains::execution::storage::execution_save_sealing`; `report_materializes_execution/{0.5s,2s}/{1,2,4}` (10 samples each).
- [x] 22. Verify initial plot acknowledgement/keepalive, concurrency and waiting state. Proof: `grpc::plots::tests::keepalive_ordering`, `pending_frame_names_the_plot`; `plots::sealed_window_all`, `plot_points_admission`, `batched_stream_queries`; `plots/nominal.test.ts / Waiting For Data`.
- [x] 23. Show finalization immediately and preserve subsecond duration values. Proof: `cycle/nominal.test.ts / Execution Controls`; `execution/nominal.test.ts / Duration Formatting, Step Durations`; `domains::execution::storage::execution_save_sealing`.
- [x] 24. Remove continue-after-failure, share viewer settings and prevent duplicate continuation. Proof: `continuation_is_frozen_before_later_appends`, `continuation_after_failure`, `continuation_ignores_failures`, `continuation_follows_seal`; `report_digest_is_stable`; `cycle/nominal.test.ts / Settings Dialogs`.
- [x] 25. Harmonize suite/template labels. Proof: `native-desktop.mjs / library, cycle, report` (authored suite/template labels through planning, execution and frozen evidence).
- [x] 26. Verify active-cycle picker and shell destination behavior. Proof: `native-desktop.mjs / active-cycle`; `cycle/nominal.test.ts / Active Containers Persist`.
- [x] 27. Repair live histograms with report parity. Proof: `native-desktop.mjs / cycle, report` (voltage bin ranges, counts and samples); `plots/nominal.test.ts / Histogram Axes`; `distribution_query_budget`; `grpc::plots::tests::histogram_definition_declared`.
- [x] 28. Show centered diagnostics in an erroring plot overlay. Proof: `plots/nominal.test.ts / Stream Errors Replace Retained Charts, Plot Placeholders`; `native-desktop.mjs / report` (controlled analysis failure, centered tinted overlay and long diagnostic wrapping).
- [x] 29. Harmonize progress row styling. Proof: `native-desktop.mjs / cycle` (state/bar/count/actions centers, nonoverlap and rounded bar); `cycle/nominal.test.ts / Execution Controls`.
- [x] 30. Preserve FIFO prompts as self-contained cards. Proof: `execution/nominal.test.ts / Prompt Ordering, Prompt Queue, Prompt Decisions, Prompt Answering`; `native-desktop.mjs / cycle`.
- [x] 31. Show empty pending, ticking running and measured completed step durations. Proof: `execution/nominal.test.ts / Step Durations, Duration Formatting`.
- [x] 32. Complete narrow operator-note persistence and measure latency. Proof: `domains::documents::capture::notes_until_seal`, `domains::evidence::notes::empty_cycle_datasets`; `workflow_adds_note/{idle_rows,during_cycle_report_rows}/100000` (10 samples each).
- [x] 33. Fix access-key dialog overflow. Proof: `settings/access-key-dialog.test.ts`; `native-desktop.mjs / remote-access` (full key and all six controls fit a 1080×760 viewport).
- [x] 34. Apply remote settings live, reflect actual status and generate missing TLS certificates. Proof: `native-desktop.mjs / remote-access` (live apply, bind rollback, TLS generation/rotation, key rotation during execution, independent auth/TLS and restart); `admitted_catalog_is_durable_before_exposure`.
- [x] 35. Use a shared two-column settings layout and repair affected dialogs. Proof: `settings/access-key-dialog.test.ts`; `native-desktop.mjs / settings, remote-access` (equal columns, full-span sections and small-viewport dialog bounds).
- [x] 36. Page remote session history in groups of twenty. Proof: `native-desktop.mjs / remote-access` (21 fresh connections, twenty-row page, disjoint older page and newer-page restoration).
- [x] 37. Answer single-boolean prompts through No/Yes footer actions. Proof: `execution/nominal.test.ts / Prompt Decisions, Prompt Answering`; `native-desktop.mjs / remote-access, restart`.
- [x] 38. Share the page shell across Library, Cycles, History and Explore. Proof: `shell/page-shell.test.ts`; `native-desktop.mjs / library, cycles-home, history`.
- [x] 39. Verify fast drag-and-drop bundle install. Proof: `app::plugins::tests::a_drop_stages_once_and_only_a_single_bundle`; `integration/settings/plugins.test.ts`; `native-desktop.mjs / plugin-install` (shared native path staging of 36 MB in 254 ms, one sub-1-KB IPC request, consent and restart activation).
- [x] 40. Persist runtime organization packs for next boot and retain compiled identity boundaries; prove pending-pack revalidation and rollback in `app::organization::tests`, then native activation. Proof: `app::organization::tests` (9 tests); `native-desktop.mjs / organization` (import, next-boot activation, invalid pending retention, diagnostics and removal).
- [x] 41. Verify the Linux WebView response-stall fix and operator override behavior. Proof: `native-desktop.mjs / startup, settings` (owned renderer environment defaults to 1 and preserves explicit 0; responsive native workflows in both configurations).
- [x] 42. Complete orderly close, bounded force quit and measured idle/running exit. Proof: `native_session::backend::tests::force_quit_reaps_unresponsive_backend`, `app::closing::tests`; `native-desktop.mjs / owned-startup, attachment, restart` (idle/running close measurements, cancelled execution and report recovery, remote host retained).
- [ ] 43. Complete and run six system benchmark scenarios with native evidence. Completed native scenarios: `workflow_installs_plugin`, `workflow_reports_running_execution`, `workflow_reports_cycle`, `workflow_seals_execution`, `workflow_adds_note/during_cycle_report_rows/100000`; run only `workflow_streams_plot` during final validation.
- [ ] 44. Verify optimized dev workspace profiles, package profile reporting and build costs. Profile implementation and `test_package_collects_and_reports_matching_profile` are proven; build-cost evidence remains for final validation.
- [ ] 45. Complete bounded record tails/incremental updates, stream invalidation, document suppression, aggregate detail and virtualization; measure idle invokes. Run the expanded `native-desktop.mjs / idle` story during final validation.
- [ ] 46. Complete native report delivery and verify transfer sizes, plot budgets, deadlines and client flow control; measure export throughput. Run `workflow_exports_evidence/200MB` once during final validation.
- [ ] 47. Complete telemetry/artifact durability batching, codec reuse, narrow state transitions and event-driven prompt waits with behavior tests and measurements; retain approved per-attempt preparation and verify cancellation/terminal evidence through the real public path.
- [x] 48. Verify batched changed-plot queries and telemetry-only refresh semantics. Proof: `plot_refresh_follows_declared_inputs`, `batched_stream_queries`.
- [x] 49. Enlarge sidebar logo to about one sixth window height, center wordmark beneath and separate navigation.

## Watcher Extension

- [x] 55. Expose observable capture-armed readiness after the watcher baseline across hosted, harness and streaming APIs, including empty baselines. Proof: `empty_baseline_arms_before_the_producer`, `failed_initial_admission_never_reports_armed`, `watch_file_matchers`, `harness_mirror_arms_before_rotation_and_retains_both_segments`, `file_watch_artifacts`, `test_armed_baseline_is_an_explicit_frame`.
- [x] 56. Preserve uniquely named single-file rotation segments and correct simultaneous-file limits without cumulative replacement rejection. Proof: `domains::files::watch::single_file_retains_renamed_and_copy_truncated_segments_with_one_slot`, `vanished_paths_release_slots_before_new_paths_are_admitted`.
- [x] 57. Fail explicitly on unsupported FTP resume when extraction requires it, preserving evidence and actionable gap/error; retain signalled artifact-only fallback. Proof: `domains::files::ftp::refused_resume_reports_extraction_failure_and_artifact_only_fallback`.
- [x] 58. Add opt-in bounded waiting for absent sources with cancellation/deadline; preserve strict defaults and immediate permission/invalid-path failures. Proof: `source_wait_rejects_invalid_paths_and_permission_failures`, `missing_source_waits_only_when_requested_and_can_seal_before_arming`, `late_source_cannot_arm_after_the_wait_deadline`, `cancelled_wait_reports_failure_to_readiness_waiters`.
- [x] 59. Add independent All/Latest-at-start/New-after-baseline selection with deterministic modification-time/path ordering and from-zero new-file capture. Proof: `domains::files::watch::latest_selects_by_mtime_then_path_and_follows_rotation`, `new_ignores_baseline_and_captures_first_discoveries_from_zero`, `latest_empty_baseline_is_fixed_unless_wait_was_requested`, `new_with_empty_accessible_root_arms_despite_opt_in_source_wait`.
- [x] 60. Prove watcher acceptance through meaningful deterministic tests and migrate hosted/harness/server/client bindings, authoring guidance, examples and README. Proof: `watch_file_matchers`, `harness_mirror_arms_before_rotation_and_retains_both_segments`, `file_watch_artifacts`, `test_armed_baseline_is_an_explicit_frame`, `CanonicalBindings`.
- [x] 61. Resolve the FTP missing-root versus permission-denied ambiguity without silently narrowing absent-source waiting.

## Consolidation And Acceptance

- [ ] 50. Adopt all outstanding source slices and regenerate every affected client/binding from the combined tree; remove superseded artifact serialization aliases in `crates/dokime-runtime/src/transactions/mod.rs:89`, `crates/dokime-runtime/src/state/ownership.rs:41` and `crates/dokime-domain/src/model/plugins/error.rs:44`, preserving foreign protocol field names and the explicit legacy pack importer.
- [ ] 51. Account for source, tests, configuration, examples, README and superseded paths; declare source readiness only after all writers hand off.
- [ ] 52. Run relevant local gates, development build, required native system walkthrough and affected lanes; repair every required failure.
- [ ] 53. Verify published upstream pins and remote-streamline integration, complete required provider checks and authorized review into dev.
- [ ] 54. Finish the change after all acceptance and evidence tasks are satisfied.

## Organization Packs (absorbed from the organization-packs change on 2026-10-21)
- [x] 62. Replace branding-pack layout/naming with organization/organization.toml across assets, generators, imports, UI, commands, examples and guidance; preserve compiled identity and restart activation. Proof: `the_repository_organization_folder_zipped_as_is_loads`, `a_staged_pack_activates_on_the_next_boot_and_an_invalid_one_falls_back`, `native-desktop.mjs / organization`.
- [x] 63. Preserve installed legacy pack data through safe migration or explicit import guidance; retain the last valid organization/update configuration on invalid input with a visible error. Proof: `explicit_legacy_import_normalizes_only_the_manifest_name`, `invalid_pending_pack_preserves_the_last_valid_active_pack`, `invalid_update_policy_preserves_the_previous_pending_pack`, `native-desktop.mjs / organization`.
- [x] 64. Implement one disabled/manual/automatic-check update policy with direct manifest, GitHub Releases and GitLab Releases sources; installation stays explicit and signing trust stays compiled. Proof: `invalid_selection_preserves_saved_policy_and_switching_invalidates_review`, `disabled_sources_are_local_and_authorization_is_origin_scoped`.
- [x] 65. Add named GitLab hosts/projects/OAuth connections and authenticated sources with host/account/client-scoped credentials, origin-safe forwarding and no secrets in packs. Proof: `unmatched_callbacks_preserve_pending_auth_and_namespaces_are_disjoint`, `disabled_sources_are_local_and_authorization_is_origin_scoped`.
- [x] 66. Implement the version-1 plugin release feed contract in design.md, organization-declared sources/links and authoritative backend target identity; check advertised package/version against inspected bytes, enforce host compatibility/target/digest and reuse Dokime staging/consent/install/restart. Proof: `release_selection_requires_compatible_host_unique_target_and_digest`, `Reviewed Release Reuses Native Staging And Leaves Consent Explicit`, `test_release_feed_uses_inspected_identity_targets_and_complete_digest`.
- [x] 67. Migrate all updater/organization consumers, generated bindings, tests, README and shipped examples; remove superseded GitLab-only updater coupling. Proof: `test_installer_keeps_compiled_trust_and_organization_owned_sources`, desktop TypeScript typecheck, `mise run check`.
- [ ] 68. Validate switching, disabled-network behavior, public/auth/missing sources, target selection, signature rejection, invalid-pack retention, credential separation and plugin lifecycle with focused tests and native acceptance.

## Shared Secret Storage (absorbed from the shared-secret-storage change on 2026-10-21)

- [ ] 69. Inventory secret persistence/keyring keys, plugin bindings and deployed consumers; lock safe adoption and disjoint ownership before the naming cutover.
- [ ] 70. Replace Dokime-owned credential terminology across public declarations, operation IDs, modules, fields, settings, FTP, clients, GUI, tests, examples and guidance; preserve foreign APIs and stored values.
- [ ] 71. Add host-owned shared named-secret lookup for all plugins with clear missing/deleted/replaced behavior and existing requirement preflight, without plugin ACLs or org-pack bindings.
- [ ] 72. Add Secret Storage UI with target-host attribution, freely managed names/labels/descriptions, masked values and honest known-usage indication.
- [ ] 73. Add bounded password-encrypted authenticated selected-secret export/import, conflict preview and explicit replace/skip with atomic validated import and no writes on password/tamper failure.
- [ ] 74. Verify multi-plugin lookup, lifecycle changes, host attribution, encryption/conflicts/limits/tamper/traversal/atomicity, redaction, namespace separation, plugin settings/FTP and regenerated client coherence.
