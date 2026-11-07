## Implementation

- [ ] 1.1 Pin Harmos a41e00fd and migrate dependency API consumers across workspace, sdk/python and plugins.
- [ ] 1.2 Derive telemetry, record and snapshot shape identities with Harmos, eliminate hand-maintained shape declarations and ordering assumptions.
- [ ] 1.3 Assess and remove redundant saved-context, export, wire, report, cursor, profile, remote and backend shape versions with all readers, bindings, stubs and templates; record justified retained checks.
- [ ] 1.4 Replace whole-root source digest refusal with Dokime-owned persistent decode compatibility; adopt stored plugin schemas and retention, audit catalogs/settings/secrets/installations/documents.
- [ ] 1.5 Add deterministic targeted proof for publish/seal/report, restart, plugin stream and record evolution on one root, built-in refusal and a built-in identity golden value.
- [ ] 1.6 Update product guide, affected authoring references and README storage claims.

## Final Validation and Delivery

- [ ] 2.1 Run only specified deterministic targeted tests through mise tasks.
- [ ] 2.2 Run the owner-required all-targets feature build, TypeScript build and lint:rust; repair failures.
- [ ] 2.3 Review relevant hosted CI and obtain CLEAN from the owner-specified scan.
- [ ] 2.4 Archive this change and publish the completed candidate through the authorized PR.

## Workflow Follow-Up

The native Harmos `Described` derive cannot describe Serde tagging, enum renaming or conditional omission used by Dokime's built-in journal records. See design.md for the exact declarations, observed errors and recommended upstream resolution. Implementation and final proof remain open; do not merge the dependency-only checkpoint.
