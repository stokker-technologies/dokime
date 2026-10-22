# Migration Design

## Inputs
The authoritative upstream contracts are in the Hub fixes worktree at openspec/changes/machine-work-root/specs/ci-contract/spec.md and specs/ci-cache-impact/spec.md. Parent owns those decisions; product owner supplies implementation details and raises contract changes.

## Ownership
Use this product's chore/migrate-hub-ci branch and hub-ci worktree. The product agent owns authored product changes and canonical regeneration; parent owns Hub integration and final cross-product coordination. No direct dev commits.

## Sequence
Record baseline failures before edits. Migrate authored sources against the stable Hub interface, then regenerate only with the rebuilt Hub handed off by parent. Run targeted tests while implementing; once source closes run complete applicable available-local proof, repair failures, then publish and monitor every required provider platform through merge.

## Baseline
Windows is the current local platform. Ordinary candidate checks/builds/tests/package and required dry-release proof are baseline scope; nightly coverage is not implicitly included. User additionally requests benchmark results: run all declared bench layers in a quiet window coordinated by parent after baseline validation, record workloads/revision/environment and report failures honestly.

Initial format verification passed. Python tooling type checking fails on Unix-only os.geteuid access. Native Rust check and Rust/TypeScript lint passed after correcting shell linker precedence. Build failed at Cargo shim resolution, so all thirteen compile-contract cases lacked build metadata, ordinary tests lacked their archive and Package lacked its wheel; these are prerequisite failures and establish no semantic test result. Dry publication fails because the dokime crate's path dependencies omit required version requirements. The initial baseline is complete.

## Portability
No simulator; CI provides unavailable native platform proof. Fix baseline failures without weakening checks. Maintain public CI without private Hub dependencies or local-only paths. Preserve machine environment and mandatory hooks.

## Producer Decisions
Product owns shipping binaries, plugin bundles, generated clients and frontend exports. Tests owns its nextest archive, offline starter closure and fixtures; compile-contract stories execute there while compiler dependencies exist, never in transported Test lanes. Move authored build orchestration away from build/run.py so the canonical shared module can own that path and target recording.

Build / Bench is standalone in the locked Hub graph. Its build:rust:bench recipe owns the report resource preparation required by the runtime's embedded asset, benchmark compilation and matching native hosts/plugins. The bench leaves only execute those existing benchmark-profile bytes. Declare frontend bootstrap/cache inputs on that producer where necessary.

## Published Dependency Candidate
The existing Harmos pin is 6eeabf4d4905c9ee984cbfc640502f914bbe6041, whose manifests identify as 0.2.0. Released v0.1.2 at b5b6aa5ba337828fcd18b5b41581ef21b065b995 descends from that revision and matches the published registry versions. Static API review found no removed Dokime dependency surface: signal and SDK error exports move owners while retaining their facade. Prefer this newer released pin with matching version requirements if targeted consumer compilation and final behavior proof pass; no upstream release is authorized or necessary on that path.

The newer released pin is adopted with version 0.1.2 on each direct Harmos dependency. Locked, all-feature compilation passes for dokime-runtime, dokime-client, dokime-packaging and dokime-gui. Full behavioral and dry-publication proof remains required.

Server compilation also exposes the released Authorizer interface change from required returning Required to authorize returning Authorization. Map each existing success to Restricted(existing Required), preserving all denial branches, caller grants and contract checks. Never select Full for this migration; existing authorization stories must still pass.

## Signed Assets
The approved Hub signed_only asset category excludes updater signatures/latest.json from unsigned Dry/Rehearsal and requires them during signed stable publication with checksum and download verification. Declare Windows Package signing and retain installer/version/manifest/signature consistency checks over the downloaded asset root. These existing checks do not establish cryptographic authenticity; no additional verifier dependency or publication path belongs to this migration.

## Executable Publication Ownership

The user approved separating executable targets from published libraries. Keep dokime and dokime-server as the existing published libraries with unchanged APIs. Move only executable entry points into dokime-cli and dokime-server-cli packages with publish = false, retaining the dokime and dokime-server binary names and behavior. Move binary-bound process stories to the owning CLI test targets or consume declared Product artifacts explicitly; never retain a dependency on a binary target removed from a library package. Migrate workspace declarations, feature forwarding, Product/Bench/package selections, release metadata and affected README references together. Raw asset filenames follow the existing package-identity contract (dokime-cli and dokime-server-cli prefixes); command mappings retain dokime and dokime-server. Preserve installer product identities and update asset declarations and install mappings together. No release is authorized by this structural migration.

## Python Workspace
Preserve Dokime's existing root uv workspace, product/tooling members and lock pins. Starters adopt the approved canonical root virtual workspace containing tools/python and a root uv.lock; retire the old managed nested lock after replacement. The offline starter wheelhouse and generated-project story consume that layout, proving root .venv creation without UV_PROJECT_ENVIRONMENT or temporary-environment redirects.

## Background Windows

The user requested suppression of flashing noninteractive terminal windows. Authored noninteractive Python launch sites use creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0), preserving Unix behavior and explicit PTY/interactive stories. Bind inherited standard streams explicitly to sys.stdin/sys.stdout/sys.stderr because implicit handles lose output in this Windows environment; retain existing captures, redirections and input ownership. The canonical test runner retains a hidden real console for CTRL_BREAK cleanup. Native noninteractive story/compiler/benchmark launchers use Windows-only CommandExt/Command creation_flags(0x08000000); the existing GUI backend already applies that policy. Canonical runtime launchers are regenerated from the parent Hub fix; authored producers use the same standard-library keyword without a competing helper API.
