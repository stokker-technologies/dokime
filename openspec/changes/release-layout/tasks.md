# Release Layout Tasks

## 1. Contract

- [x] 1.1 Lock the source inventory: asset producers, the release and Nightly tooling, the updater, the download index and plugin feed consumers, rehearsal, tests and guides.

Source inventory: `tools/python/src/dokime_tools/{release,package,ci,test}/`,
`tools/python/tests/integration/{test_release_*,test_run,test_ci_workflow}.py`,
`tools/project.toml`,
`.github/workflows/{ci,nightly,release,publish,verify,validate-*}.yml`, desktop `app/updates/`, installer
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
- [x] 2.8 Extract the shared validation, publish and verify jobs as reusable workflows and make Nightly call them with the nightly channel.
- [x] 2.9 Align the Nightly contract with Harmos: triggers, cancellation, freshness, names, artifact interface, publish entry point, verification and report.

## 3. Final Validation

- [ ] 3.1 Run the release rehearsal and one Nightly, and compare the published asset list with the layout.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.

Implementation decisions: updater fields remain at the index root for Tauri’s
native parser; `downloads` and `plugins` are named sections. SHA256SUMS covers
the index, so the download inventory excludes itself and SHA256SUMS. Local
signing’s internal latest.json remains a package input, never a published asset.
The pure Python SDK wheel `dokime-<version>-py3-none-any.whl` and source
distribution `dokime-<version>.tar.gz` each ship once under `any/`.
The cargo-dokime archives remain as agreed.

Shared validation now lives in `validate-*.yml`; `publish.yml` and `verify.yml`
take the channel, profile and validated review metadata. Release retains signing
and review routing. Nightly publishes after packages alone; its final report uses
the same verified draft publisher to replace the inventory with completed reports,
index, checksums and suite notes. This second upload preserves the original notes
and exact publication identity. Native Nightly verification uses mise, while
Cargo's semantic-version installer remains stable-only. The four-hour UTC trigger
and noncancelling concurrency are unchanged. Duplicate revision detection admits
the oldest run; API lookup errors fail closed. Named input archives fail when
missing, and draft/public download bytes are hashed against the upload manifest.
`docs/tooling.md#harmos-alignment` records the present shared contracts from the
read-only Harmos comparison; its tooling remains separately owned. Every package
download honors `build-run || github.run_id`, including Windows Rust inputs.

Focused workflow proof: repository check (including actionlint), full formatting,
37 CI tooling tests and 33 release tooling tests passed. Tooling-only Ruff and ty
checks passed. Combined-tree integration repaired both SDK `RUF022` findings in
`sdk/python/python/dokime/{plot.py,v1/__init__.py}`; Python checking now passes
including 107 tooling tests. The focused release selection passes 35 tests and
the focused workflow selection passes 12 tests.
A clean `release:rehearse -- --profile dev --target all --stage-only` passed with
both platforms' staged packages and 17 upload entries. Previous release outputs
were preserved at `target/release-assets-before-workflow-rehearsal`.
The combined-tree clean-output repeat also passed for both platforms with 17
upload entries, using the existing dev packages (native executables identify
revision `ca2fc48d`). This proves staging and native Windows package verification;
it does not prove current-source product packaging or native Linux execution.

The combined-tree Rust check now passes its dev/test, desktop and bench-profile
Clippy selections and cargo-machete; the prior disk-space interruption no longer
blocks that check. The nonfatal unreachable-method configuration warning remains.
Validation still required: native consumer tests, full native
release-profile rehearsal, Linux execution proof and an authorized hosted Nightly
with published inventory comparison. The local dev-package dry run does not close
3.1. Fixture-based rehearsal and
exact stable/Nightly inventories, signature-index verification and native
Criterion measurement normalization are covered by tooling tests. No publishing,
tag or specification lifecycle commands were run.
