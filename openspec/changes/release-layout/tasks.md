# Release Layout Tasks

## 1. Contract

- [ ] 1.1 Lock the source inventory: asset producers, the release and Nightly tooling, the updater, the download index and plugin feed consumers, rehearsal, tests and guides.

## 2. Implementation

- [ ] 2.1 Bundle every bill of materials with the notices into `dokime-compliance-<version>.zip` and stop attaching them singly.
- [ ] 2.2 Publish `release.json` and move the updater, the download index and the plugin feed onto it; delete `latest.json`, `downloads.json` and `plugins.json`.
- [ ] 2.3 Rename the sidecars archive to `dokime-plugins`, attach `dokime-proto-<version>.zip`, and move the changelog into the release body.
- [ ] 2.4 Make Nightly publish the same layout from the same tasks as a stable release.
- [ ] 2.5 Attach `dokime-reports-<version>.zip` to Nightly and add the results table to its notes.
- [ ] 2.6 Update the rehearsal, the product guide and the README.
- [ ] 2.7 Attach `benchmarks.json` to Nightly under its fixed name.
- [ ] 2.8 Extract the shared validation, publish and verify jobs as reusable workflows and make Nightly call them with the nightly channel.
- [ ] 2.9 Align the Nightly contract with Harmos: triggers, cancellation, freshness, names, artifact interface, publish entry point, verification and report.

## 3. Final Validation

- [ ] 3.1 Run the release rehearsal and one Nightly, and compare the published asset list with the layout.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
