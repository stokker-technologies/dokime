# Migration Design

## Inputs
The authoritative upstream contracts are in the Hub fixes worktree at openspec/changes/machine-work-root/specs/ci-contract/spec.md and specs/ci-cache-impact/spec.md. Parent owns those decisions; product owner supplies implementation details and raises contract changes.

## Ownership
Use this product's chore/migrate-hub-ci branch and hub-ci worktree. The product agent owns authored product changes and canonical regeneration; parent owns Hub integration and final cross-product coordination. No direct dev commits.

## Convention Audit Scope

The user explicitly approved fixing migration-related findings and Hub false positives while deferring unrelated pre-existing convention cleanup. This is a scope exception for this migration, not a claim that the whole-project convention audit passes. Preserve evidence of the deferred formatting, test-layout and naming findings in the maintained migration specification. Correct introduced crate/configuration/task-shape defects and required canonical CI task/pin requirements. Shared checker false positives belong to Hub. Required CI, packaging and publication dry-run failures remain blocking; no deadline, gate or dirty-source check is weakened.

Deferred pre-migration categories are skeleton/header and attributed-member spacing, test-domain mirroring and placement, existing identifier naming, existing Error module placement and test environment access, existing Rustdoc examples, dependency grouping outside the newly created executable crates, and the unchanged Apache license text. Representative unchanged sources include cargo-dokime/src/cli.rs, runtime test service state modules and LICENSE-APACHE. The broad audit remains failing; this migration does not claim otherwise. New executable sources and manifests are canonically formatted. Root task ordering, tool pins, OpenSpec Markdown placement and the new required dependency-use check are migration-owned. Unused dependency declarations are removed; the server uses the Harmos generic transport and generated descriptors rather than a tonic-prost client/server source module.

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

The current producer's --bins --benches selection also compiles the runtime's implicit library harness, which the artifact exporter does not consume. During the native Windows build, the still-running normal library and implicit harness compiler processes had accumulated approximately 903 and 649 CPU seconds respectively; these are partial observations, not completed build durations or benchmark measurements. The parent explicitly deferred a benchmark-selector redesign. Preserve the current producer and all five explicit benchmark executables; reducing redundant harness compilation is a future optimization opportunity.

## Published Dependency Candidate
The existing Harmos pin is 6eeabf4d4905c9ee984cbfc640502f914bbe6041, whose manifests identify as 0.2.0. Released v0.1.2 at b5b6aa5ba337828fcd18b5b41581ef21b065b995 descends from that revision and matches the published registry versions. Static API review found no removed Dokime dependency surface: signal and SDK error exports move owners while retaining their facade. Prefer this newer released pin with matching version requirements if targeted consumer compilation and final behavior proof pass; no upstream release is authorized or necessary on that path.

The newer released pin is adopted with version 0.1.2 on each direct Harmos dependency. Locked, all-feature compilation passes for dokime-runtime, dokime-client, dokime-packaging and dokime-gui. Full behavioral and dry-publication proof remains required.

Server compilation also exposes the released Authorizer interface change from required returning Required to authorize returning Authorization. Map each existing success to Restricted(existing Required), preserving all denial branches, caller grants and contract checks. Never select Full for this migration; existing authorization stories must still pass.

## Signed Assets
The approved Hub signed_only asset category excludes updater signatures/latest.json from unsigned Dry/Rehearsal and requires them during signed stable publication with checksum and download verification. Declare Windows Package signing and retain installer/version/manifest/signature consistency checks over the downloaded asset root. These existing checks do not establish cryptographic authenticity; no additional verifier dependency or publication path belongs to this migration.

## Executable Publication Ownership

The user approved separating executable targets from published libraries. Keep dokime and dokime-server as the existing published libraries with unchanged APIs. Move only executable entry points into dokime-cli and dokime-server-cli packages with publish = false, retaining the dokime and dokime-server binary names and behavior. Move binary-bound process stories to the owning CLI test targets or consume declared Product artifacts explicitly; never retain a dependency on a binary target removed from a library package. Migrate workspace declarations, feature forwarding, Product/Bench/package selections, release metadata and affected README references together. Raw asset filenames follow the existing package-identity contract (dokime-cli and dokime-server-cli prefixes); command mappings retain dokime and dokime-server. Preserve installer product identities and update asset declarations and install mappings together. No release is authorized by this structural migration.

The user approved allowing thin binary-only delegating crates to reuse their owning libraries' Error/Result and exit handling without introducing unused crate-local error wrappers. Hub owns the matching canonical guidance and deterministic checker repair. Crates that own an independent runtime error API retain the existing Error requirements.

## Self-Contained Runtime Resources

The user approved tracking approximately 1.7 MiB of generated crate-local report and branding distribution inputs. GUI report sources and root branding remain authoritative; the runtime copies have no independent authorship. Explicit preparation refreshes those copies, and a freshness check detects mismatches without repairing them silently. Cargo includes the report and required branding files; runtime build.rs reads only crate-local inputs, so a published host-feature build needs neither Node nor the repository root. Preserve Cargo dirty-source enforcement, pinned generation and public feature/API behavior. Verify actual packaged host-feature builds as well as ordinary Dry; no scratch-repository publication assembly or allow-dirty bypass.

The payload is runtime/dist-report/report.html plus runtime/dist-brand/brand.toml and the theme/fonts/logos files (16 files, 1,698,065 bytes). `mise run prepare:report` uses the GUI report renderer and copies the selected brand inputs; the report's obsolete input-hash cache is retired. Rust Check calls `dokime_tools.prepare.resources --check` before compilation, rebuilding into a normal temporary directory and comparing the full file inventory and bytes. Modified, missing and extra brand files and stale report bytes fail without repairing the payload. Product/Tests/Bench retain the preparation dependency, and desktop startup uses the same resource owner.

## Python Workspace
Preserve Dokime's existing root uv workspace, product/tooling members and lock pins. Starters adopt the approved canonical root virtual workspace containing tools/python and a root uv.lock; retire the old managed nested lock after replacement. The offline starter wheelhouse and generated-project story consume that layout, proving root .venv creation without UV_PROJECT_ENVIRONMENT or temporary-environment redirects.

## Background Windows

The user requested suppression of flashing noninteractive terminal windows. Authored noninteractive Python launch sites use creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0), preserving Unix behavior and explicit PTY/interactive stories. Bind inherited standard streams explicitly to sys.stdin/sys.stdout/sys.stderr because implicit handles lose output in this Windows environment; retain existing captures, redirections and input ownership. The canonical test runner retains a hidden real console for CTRL_BREAK cleanup. Native noninteractive story/compiler/benchmark launchers use Windows-only CommandExt/Command creation_flags(0x08000000); the existing GUI backend already applies that policy. Canonical runtime launchers are regenerated from the parent Hub fix; authored producers use the same standard-library keyword without a competing helper API.

## Per-Job Runner Routing

The user's later decision supersedes the six provider runner variables. Each existing [[ci.jobs]] selector accepts runner = "stokker-local-linux-x64" for platform = "linux", or runner = "stokker-local-windows-x64" for platform = "windows". Merge into existing selectors without duplicating them. Use these labels for check:rust, build, build:rust:tests and every declared ordinary test:rust layer on its existing platforms. Keep existing platform policy; do not introduce a platform simply to route it. Nightly build:rust:bench and bench leaves use the local Linux label. Lightweight repository/Python/TypeScript/generated checks and packaging retain hosted defaults; Rust coverage retains its hosted policy. Remove project-wide local ci.runners defaults that would route lightweight jobs locally, while preserving any necessary declared hosted platform labels. Omitted runner follows the project/provider default, and no provider variable overrides it. Shared setup/tasks/gates remain identical for local and hosted jobs; no dynamic overflow scheduler is added. Explicit Linux UI bootstrap remains attached only to jobs that need it.

Adopt the parent Hub implementation snapshot .cache/local-hub/stkr-ci-dd8f13d1f440.exe (SHA256 dd8f13d1f44018591e2618aaba18156f39dd3736bf8edbed2f37e1f62b15073d) or a later parent-approved canonical snapshot with --no-provider. It also disables redundant upload compression for existing zstd archives without changing archive contents. Do not interrupt live validation to adopt; reopen affected source at its terminal boundary, then preserve unaffected passing proof. The machine's two slots per provider and explicit opt-in runner provisioning are owned by the Hub, not Dokime source.

## Windows Archive Discovery

Canonical90f6e9c7825d discovers GNU tar from PATH or the installed Git executable's sibling usr/bin, verifies GNU identity plus force-local/external-compressor capabilities, and prepends its directory for zstd. Missing capability fails with a runner provisioning diagnostic; no ProgramFiles/Git assumption remains. Windows Build, downstream extraction and Package use the same discovery block; GitLab Windows any-profile archive commands read env:PROFILE. Native zstd and plain archive roundtrips use the actual generated discovery code. Cargo-dokime embeds its starter directory through build.rs/include_bytes, so changed starter workflow copies require that compiled consumer to rebuild; unchanged runtime/server/GUI compilation and byte-bound fixture stamps remain valid.
