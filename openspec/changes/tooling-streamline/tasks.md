# Tooling Streamline Tasks

## 1. Contract

- [ ] 1.1 Settle the open points in `proposal.md` with the owner from the CI audit and record the final task table and CI shape.
- [ ] 1.2 Lock the source inventory: `mise.toml`, `tools/python`, workflows, contributor and tooling documents.

## 2. Tasks

- [ ] 2.1 Replace the task families with the entry points in `proposal.md`, with language and layer as arguments, and delete the unused and superseded tasks.
- [ ] 2.2 Merge the generation tasks into `generate` and move organization and report preparation into `build` and `package`.
- [ ] 2.3 Remove every `CARGO_INCREMENTAL` setting from `mise.toml` and tooling and set it once in the workflows.

## 3. Tooling

- [ ] 3.1 Adopt the shared build tooling without the storage preflight and delete the wrapper code that existed only for it.
- [ ] 3.2 Delete the duplicated and dead modules in `tools/python` and keep one task dependency graph.

## 4. Workflows

- [ ] 4.1 Rewrite the CI, Nightly and Release workflows so each job runs one task, per the shape task 1.1 records.

## 5. Final Validation

- [ ] 5.1 Run every entry point locally on Windows and Linux, and confirm hosted CI and Nightly run the same tasks.
- [ ] 5.2 Update `CONTRIBUTING.md` and the tooling guide.

## Workflow Follow-Up

- Harmos gets the same change after its own audits.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
