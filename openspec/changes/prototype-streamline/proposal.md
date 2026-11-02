# Prototype Streamline

## Why

The storage optimization combined useful storage improvements with hundreds of operational knobs and pervasive retained-memory accounting. The prototype needs editable product settings, reliable backend selection, and a smaller implementation surface.

## Status And Scope

This change is active on `refactor/prototype-streamline`. Its task inventory distinguishes landed source, superseded requests and remaining implementation/proof; checked historical tasks are not evidence of a new validation run. The former `storage-optimization` change is absent from the current inventory. This assignment authorizes local commits and targeted proof only, with no push, broad validation or other-worktree writes. Retirement remains blocked until all obligations close.

Later locked changes own overlapping contracts: `embedded-server-and-ci` replaces desktop startup/default selection, `distribution-identity` owns installation identity and application-root resolution, `organization-installers` owns installer composition, and `host-plugin-features` owns host controls and telemetry leases. `dokime-remote-files` supersedes remote file/export/import, endpoint selection, persistent CA and endpoint-stagger preferences. Those changes are not implemented by this assignment. Their requirements supersede the corresponding historical requests here; all other obligations remain.

## Locked Decisions

1. Replace operational configuration with owner-local constants. Of the 409 public leaf fields in `crates/dokime-domain/src/model/settings/config/*.rs`, retain four user-facing controls and remove 405 configuration fields. This count excludes wrapper structs, Harmos-native configuration, runtime handles, and existing preferences. Remove native policy serialization too; do not present its excluded fields as retained settings.
2. Store application settings in the existing SQLite storage. Remove `config.toml` loading, writing, overlays and discovery, and replace `settings.json` persistence. Declare the typed application model once in `dokime-domain`; generate required TS/Python types from it. Keep plugin-authored setting schemas and secret references, with their existing validation and secure storage.
3. Every retained setting is editable in Settings. Simple exposes everyday preferences; Advanced includes all controls. Reuse the CLI backend registry and GUI connection manager. Backend selection and deletion remain explicit; the later `embedded-server-and-ci` owned-selection contract replaces the original startup/CLI-default coupling.
4. Remove Dokime's memory-admission/estimation plumbing and the paired Harmos storage accounting extensions. Preserve opened-file ownership, transactional one-row encoding, repeated-item identity, allocation-free decimation, exact wire sizing, borrowed serialization, range filtering and saved-storage live plots.
5. Consolidate tests around behavior: remove 19 named test definitions, preserving the useful assertions identified in the task inventory. Do not remove corruption, durability, cancellation, authentication, transport-size or performance-regression proof merely because it currently fails.
6. Apply Harmos's flat feature-gated operation authoring to crate layering. The published dependency and `dokime-operations` are present; remaining resource-library and SDK extraction stays open in section 9.

## Acceptance

When a user changes a setting, domain validation precedes one transactional database update; reopening returns the saved value. A stale update returns a conflict without overwriting another edit. No configuration file is created.

When startup cannot connect to the remembered backend, the GUI remains usable and presents saved backends with the connection error. Manual lab profile addition remains offline; remote-file import now verifies and connects under dokime-remote-files. Cancelled deletion changes nothing. The later owned-selection contract separates remembered desktop selection from the CLI default and forbids an unavailable remote from silently starting Local.

When memory accounting is removed, saved evidence, stream ordering, cancellation settlement, exact wire limits and original opened-file identity remain correct. No unbounded replacement queue or silent sample dropping is introduced.

When tests are consolidated, each surviving contract has one primary behavioral owner; process-kill/restart acceptance still proves durability that in-process tests cannot establish.

## Risks

The paired Harmos API and Dokime's adapters must remain aligned; the current dependency is published Harmos 0.1.13. Existing development roots and configuration files must never be silently reset, deleted, imported or rewritten. Task-local targeted proof does not establish combined source closure or native-platform completion. Remaining implementation, final validation, publication authorization and specification retirement stay open in tasks.md.
