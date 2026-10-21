# Tasks

- [x] Provision Dokime's own single-repository release PAT at the organization's maximum lifetime as repository RELEASE_TOKEN with gh; verify live check-run reads without exposing credentials.
- [ ] Verify Dokime's actual release-candidate rehearsal result is readable with its scoped release token before tagging.

## 1. Registry Adoption And Source

- [ ] 1.1 Verify upstream Harmos 0.1.1 registry, assets and native publication before adopting it.
- [ ] 1.2 Replace four root Harmos Git declarations with registry 0.1.1; regenerate and inspect all affected locks.
- [ ] 1.3 Remove integration patches and obsolete Git pins from starter/bootstrap consumers; inspect Cargo resolution.
- [ ] 1.4 Review README, examples and generated clients; apply only changes required by registry adoption.
- [ ] 1.5 Recover the complete inherited ACCEPT-01 through ACCEPT-14 contracts into tracked acceptance and reconcile source ownership against the preserved legacy inventory.
- [ ] 1.6 Confirm complete PR2 source inventory and ownership, then run relevant local gates and verify signed remote SHA.
- [x] 1.7 Generate the authorized encrypted updater key, retain a private backup, provision both protected release secrets with gh and update the candidate public key; verify the pair and tampered-payload refusal with the production verifier.
- [ ] 1.8 Adopt a verified published Hub linear-release generator; migrate managed workflows, helpers, version policy, notes and provenance consumers, remove superseded paths, and inspect the complete generated diff. Current release/registry.py:399 and release/run.py:412 still enforce the old main route.
- [ ] 1.9 Verify approved RELEASE_TOKEN identity and protection migration, retaining required dev checks/rehearsal and protected fast-forward-only main/tag ownership; do not change current protections without the necessary authorization.
- [ ] 1.10 Retire CHANGELOG.md through a supported Hub owner operation after reconciling the installed changelog edit guard with the new tag-time notes contract; preserve the guard until then.

## 2. Integration And Release

- [ ] 2.1 After source completion and lane coordination, prove inherited installed-desktop, real key/certificate save-dialog and exact twelve-script acceptance against candidate artifacts; inspect native reports rather than assuming hosted checks cover it.
- [ ] 2.2 Run the previously agreed applicable capture/storage/report/transport/shutdown benchmarks after functional validation; retain native reports and report measurement limits.
- [ ] 2.3 Pass all required Linux/Windows PR2 checks including Release / Dry under the migrated graph; enable ready squash auto-merge.
- [ ] 2.4 Verify PR2 integration and announce availability of dev for the feature drop.
- [ ] 2.5 Integrate PR3's canonical license with required checks.
- [ ] 2.6 Generate and review v0.1.0 manifests, locks and release metadata through Release / Prepare's review into dev; preview git-cliff notes without a tracked changelog.
- [ ] 2.7 Confirm complete release source, then pass local gates, full native graph and exact-candidate rehearsal.
- [ ] 2.8 Require real signed Windows package verification with the protected release secrets and matching configured public key.
- [ ] 2.9 Notify before enabling the release review's squash merge into dev; verify rehearsed/merged tree identity and Release / Tag's exact tag and main fast-forward.
- [ ] 2.10 Publish and verify remote registries, assets, checksums and native install jobs; verify main equals the tag commit and dev contains that commit.

## Workflow Follow-Up

- Finish the change after all tasks pass; retain release resources until publication and incorporation are verified.
