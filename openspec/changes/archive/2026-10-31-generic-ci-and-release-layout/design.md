# Design

## Locked Contract

The owner-approved Hub design at `generic-ci-and-release-layout/design.md` governs this adoption. Release archives use `<tool>-<version>-<Rust-target>.zip` on Windows and `.tar.gz` on Unix. Payloads use `dokime-payload-<version>-<Rust-target>.zip`. Native DEB/RPM conventions and `latest.json` remain.

`package.executables` already selects both shipped tools. The rendered package implementation assembles archives with root binaries, LICENSE, README and generated changelog; release staging produces `plugins.json`, `SHA256SUMS` and transport-only `upload.json`. The rendered rehearsal strips credentials and stages all configured release targets.

## Ownership And Inventory

- `.stkr/project.toml`: explicit native installers, payload kind and host-plugin release declaration; remove retired download declarations and bundle release flag.
- `mise.toml`: native task references, package target arguments and credential-free rehearsal description.
- `crates/dokime-packaging/src/installer/native.rs`: Windows output name includes its Rust target so generated latest.json URLs match staged release names.
- `crates/cargo-dokime/Cargo.toml`: tgz metadata and Windows ZIP override, root binary path.
- `tools/python/src/dokime_tools/package/plugins.py`: collect packages selected by release.plugins for the native target, once each; explicit test/bench calls select OBC separately. The release declaration owns shipped-package selection.
- `packages/dokime-client/tests/integration/client/test_plugin_releases.py`: prove collection from existing compiler records without a compiler rerun.
- `tools/python/src/dokime_tools/package/{payload,installer}.py`: payload naming and signed verification using the signature embedded in latest.json when the standalone sidecar is absent.
- `packages/dokime-client/tests/integration/client/test_installers.py`: affected payload and published-signature behavior.
- `.agents/skills/dokime-testing/SKILL.md` and generated provider copies: distinguish test-only OBC from shipped plugins.
- `README.md`, `docs/product-guide.md`, `packages/dokime-client/examples/remote/README.md`: archive installation and release inventory.

## Dependencies And Proof

The owner-selected next-release branch supplies the shipped plugin-host package, staged as plugins/plugin-host.dokime. Its release declaration uses the workspace version and the compatible 0.1 host series. Native package and release proof requires matching Product Build outputs. The owner requests local-platform dry or rehearsal proof. All-target release rehearsal requires both release-profile native packages and remains with combined integration; Windows-only proof cannot establish Unix execution.

The Python integration task delegates argument parsing to its existing module so focused pytest selections reach the runner.

The streamline owner decision of 2026-10-29 removes the custom report-save task; useful timing uses the standard system benchmark category. There is no report-save task-category exception to preserve.

Targeted proof includes changed installer tests, package/release dry or rehearsal tasks, strict OpenSpec validation, and project render diagnostics (the intentional CI hold digest is expected). No full lane, push, release or Hub checkout mutation is authorized.

## Next Release Integration

The owner authorizes merging feat/next-release into this render branch, reconciling product-owned inputs, re-rendering Hub-owned files with provider access disabled and restoring every Check/Test hold. Retired managed workflow files stay deleted. No push is authorized.
