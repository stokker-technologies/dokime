# Tasks

## 1. Implementation

- [ ] 1.1 Map what authoring uses from `dokime-runtime` and fix the crate boundary in design.md.
- [ ] 1.2 Extract the authoring API into `sdk/rust` as `dokime-sdk`.
- [ ] 1.3 Migrate the host and OBC plugins, the starter template and `cargo dokime`.
- [ ] 1.4 Remove `plugin-sidecar` and superseded paths from `dokime`.
- [ ] 1.5 Measure plugin build time and bundle size before and after.
- [ ] 1.6 Update the README, plugin guides and contract tests.
- [ ] 1.7 Add `dokime new --lang` sharing one implementation with `cargo dokime sidecar new`, with per-language minimal `mise.toml` templates.
- [ ] 1.8 Move the sidecar, installer and bindings commands into `dokime`, and delete `crates/cargo-dokime`.
- [ ] 1.9 Replace the starters with pure projects and bundle tasks, dropping their release tooling.
- [ ] 1.10 Migrate Dokime's tasks, CI and documents; measure the `dokime` binary size before and after.

## 2. Final Validation

- [ ] 2.1 Run `check:rust`, compile contracts and focused plugin tests.

## Workflow Follow-Up

- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
