# Release Layout Tasks

## 1. Contract

- [x] 1.1 Lock the source inventory: asset producers, the release and Nightly tooling, the updater, the download index and plugin feed consumers, rehearsal, tests and guides.

Source inventory: `tools/python/src/dokime_tools/{release,package,ci,test}/`,
`tools/python/tests/integration/{test_release_*,test_run,test_ci_workflow}.py`,
`tools/project.toml`,
`.github/workflows/{ci,nightly,release}.yml`, desktop `app/updates/`, installer
`crates/dokime-packaging/src/installer/release.rs` and its integration test,
`README.md`, `docs/{tooling,product-guide}.md`. There is no client-crate download
index reader in this revision; `release.layout.downloads` owns that contract.

## 2. Implementation

- [x] 2.1 Bundle every bill of materials with the notices into `dokime-compliance-<version>.zip` and stop attaching them singly.
- [x] 2.2 Publish `release.json` and move the updater, the download index and the plugin feed onto it; delete `latest.json`, `downloads.json` and `plugins.json`.
- [x] 2.3 Rename the sidecars archive to `dokime-plugins`, attach `dokime-proto-<version>.zip`, and move the changelog into the release body.
- [x] 2.4 Make Nightly publish the same layout from the same tasks as a stable release.
- [x] 2.5 Attach `dokime-reports-<version>.zip` to Nightly and add the results table to its notes.
- [x] 2.6 Update the rehearsal, the product guide and the README.
- [x] 2.7 Attach `benchmarks.json` to Nightly under its fixed name.
- [ ] 2.8 Extract the shared validation, publish and verify jobs as reusable workflows and make Nightly call them with the nightly channel.
- [ ] 2.9 Align the Nightly contract with Harmos: triggers, cancellation, freshness, names, artifact interface, publish entry point, verification and report.

## 3. Final Validation

- [ ] 3.1 Run the release rehearsal and one Nightly, and compare the published asset list with the layout.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.

Implementation decisions: updater fields remain at the index root for Tauri’s
native parser; `downloads` and `plugins` are named sections. SHA256SUMS covers
the index, so the download inventory excludes itself and SHA256SUMS. Local
signing’s internal latest.json remains a package input, never a published asset.
Native wheels and cargo-dokime archives remain as agreed.

Tasks 2.8–2.9 remain open: extract CI validation and Release publish/verify jobs
into reusable workflows with a channel input; replace callers without changing
stable signing, registry identity, review gates or artifact transport. Then align
job names, the named artifact interface, channel publish/verify entry points and
remote download digest verification with Harmos. The existing four-hour trigger,
noncancelling concurrency and fail-closed freshness already match. The report is
idempotent and run-identity guarded, but this is not complete workflow alignment.

Validation still required: native consumer tests and GUI Rust check (compilation
stopped when the session disk watcher crossed 50 GiB), native package rehearsal
(no target/package/dev/windows-x86_64 inputs), Linux proof and an authorized
hosted Nightly with published inventory comparison. Fixture-based rehearsal and
exact stable/Nightly inventories, signature-index verification and native
Criterion measurement normalization are covered by tooling tests. No publishing,
tag or specification lifecycle commands were run.
