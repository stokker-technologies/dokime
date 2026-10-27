# Prototype Streamline

## Why

The uncommitted storage optimization combines useful storage improvements with hundreds of operational knobs and pervasive retained-memory accounting. Tonight's prototype needs editable product settings, reliable backend selection, and a smaller implementation surface.

## Status And Scope

This is an authored implementation plan, not landed behavior or validation evidence. Preserve the existing `perf/storage-optimization` workspace and `.cargo/config.toml`; no reset, stash, checkout, commit, compilation, or other-worktree writes are authorized in this design phase. The branch has no tracked upstream; `origin` is the GitHub Dokime repository. Do not activate or finish either specification during planning. The existing `storage-optimization` change remains intact; integration must reconcile its superseded accounting/configuration obligations before retiring it.

## Locked Decisions

1. Replace operational configuration with owner-local constants. Of the 409 public leaf fields in `crates/dokime-domain/src/model/settings/config/*.rs`, retain four user-facing controls and remove 405 configuration fields. This count excludes wrapper structs, Harmos-native configuration, runtime handles, and existing preferences. Remove native policy serialization too; do not present its excluded fields as retained settings.
2. Store application settings in the existing SQLite storage. Remove `config.toml` loading, writing, overlays and discovery, and replace `settings.json` persistence. Declare the typed application model once in `dokime-domain`; generate required TS/Python types from it. Keep plugin-authored setting schemas and secret references, with their existing validation and secure storage.
3. Every retained setting is editable in Settings. Simple exposes everyday preferences; Advanced includes all controls. Reuse the CLI backend registry and GUI connection manager. A failed startup connection opens the Backends table, with selection, Add dialog and confirmed row deletion.
4. Remove Dokime's memory-admission/estimation plumbing and the paired Harmos storage accounting extensions. Preserve opened-file ownership, transactional one-row encoding, repeated-item identity, allocation-free decimation, exact wire sizing, borrowed serialization, range filtering and saved-storage live plots.
5. Consolidate tests around behavior: remove 19 named test definitions, preserving the useful assertions identified in the task inventory. Do not remove corruption, durability, cancellation, authentication, transport-size or performance-regression proof merely because it currently fails.
6. Defer crate layering until Harmos `authoring-streamline` lands. Its flat feature-gated operation authoring supersedes the current mirrored handler layout only in that later work.

## Acceptance

When a user changes a setting, domain validation precedes one transactional database update; reopening returns the saved value. A stale update returns a conflict without overwriting another edit. No configuration file is created.

When startup cannot connect to the remembered backend, the GUI remains usable and presents saved backends with the connection error. Adding a profile does not require a connection; successful selection remembers it; cancelled deletion changes nothing.

When memory accounting is removed, saved evidence, stream ordering, cancellation settlement, exact wire limits and original opened-file identity remain correct. No unbounded replacement queue or silent sample dropping is introduced.

When tests are consolidated, each surviving contract has one primary behavioral owner; process-kill/restart acceptance still proves durability that in-process tests cannot establish.

## Risks

The paired Harmos API must change with Dokime's adapters. A running test may own the target directory. Available test output ends in linker errors, so no completed failure list or passing runtime result is asserted. Existing development roots and configuration files must never be silently reset, deleted, imported or rewritten. Source closure, targeted proof, final native-platform validation, publication authorization and specification retirement remain future work.
