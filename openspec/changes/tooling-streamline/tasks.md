# Tooling Streamline Tasks

## 1. Contract

- [x] 1.1 Lock entry-point argument names and workflow matrices from `mise.toml`, the dispatcher and project-owned CI tasks.
- [x] 1.2 Lock `mise.toml`, `tools/python`, `.github/workflows`, setup action, contributor guide and tooling guide as the source inventory.

## 2. Tasks

- [x] 2.1 Expose language/layer dispatch through the entry points in `proposal.md`.
- [x] 2.2 Merge generation into `generate` and route organization/report preparation through build and package paths.
- [x] 2.3 Remove every `CARGO_INCREMENTAL` setting from `mise.toml` and tooling and set it once in the workflows.
- [ ] 2.4 Delete superseded language/layer task leaves and migrate their callers; retain only the agreed public entry points and necessary workflow transport operations.
- [ ] 2.5 Remove PyO3 wheel, binding and native fixture tasks and jobs when the pure Python SDK lands; replace them with pure-package proof, coordinated with `grpc-plugins` 4.3 and 5.3.

## 3. Tooling

- [x] 3.1 Adopt the shared build tooling without the storage preflight and delete the wrapper code that existed only for it.
- [x] 3.2 Delete the duplicated and dead modules in `tools/python` and keep one task dependency graph.
- [x] 3.3 Pin installable uv 0.13.7 in the task declaration and lockfile; remove reliance on a temporary tool-version override.

## 4. Workflows

- [x] 4.1 Rewrite the CI, Nightly and Release workflows so each job runs one task, per the shape task 1.1 records.
- [x] 4.2 Provision Python before native CI setup and guard Windows-only process and filesystem APIs by platform.

## 5. Final Validation

- [ ] 5.1 Run every entry point locally on Windows and Linux, and confirm hosted CI and Nightly run the same tasks.
- [x] 5.2 Update `CONTRIBUTING.md` and the tooling guide.
- [ ] 5.3 Verify the Linux product build and repair platform-specific failures through the product build task; Windows proof does not close this task.
- [ ] 5.4 Reconcile machine Hub repository-check support for the entry-point model before final repository-check proof; keep Hub implementation outside this product.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
