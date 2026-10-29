# Design

## Locked Contract

The owner-approved Hub design at `generic-ci-and-release-layout/design.md` governs this adoption. Release archives use `<tool>-<version>-<Rust-target>.zip` on Windows and `.tar.gz` on Unix. Payloads use `dokime-payload-<version>-<Rust-target>.zip`. Native DEB/RPM conventions and `latest.json` remain.

`package.executables` already selects both shipped tools. The rendered package implementation assembles archives with root binaries, LICENSE, README and generated changelog; release staging produces `plugins.json`, `SHA256SUMS` and transport-only `upload.json`. The rendered rehearsal strips credentials and stages all configured release targets.

## Ownership And Inventory

- `stkr.toml`: explicit native installers, payload kind and host-plugin release declaration; remove retired download declarations and bundle release flag.
- `mise.toml`: native task references, package target arguments and credential-free rehearsal description.
- `crates/dokime-packaging/src/installer/native.rs`: Windows output name includes its Rust target so generated latest.json URLs match staged release names.
- `crates/cargo-dokime/Cargo.toml`: tgz metadata and Windows ZIP override, root binary path.
- `tools/python/src/dokime_tools/package/{payload,installer}.py`: payload naming and signed verification using the signature embedded in latest.json when the standalone sidecar is absent.
- `packages/dokime-client/tests/integration/client/test_installers.py`: affected payload and published-signature behavior.
- `.agents/skills/dokime-testing/SKILL.md` and generated provider copies: distinguish test-only OBC from shipped plugins.
- `README.md`, `docs/product-guide.md`, `packages/dokime-client/examples/remote/README.md`: archive installation and release inventory.

## Dependencies And Proof

This tree contains only OBC. The shipped host plugin producer is owned by `prototype-streamline` (task 9.0.3); its exact package/bundle identity is pending owner input. This adoption will declare the matching input without implementing that parked change. Native package and release proof requires matching Product Build outputs. All-target rehearsal requires both native package artifacts; Windows-only proof cannot establish Unix execution.

The Python integration task delegates argument parsing to its existing module so focused pytest selections reach the runner.

The Hub skeleton formatter rejects the existing `measure:report-save` task, which the parked streamline contract owns. Its owner must resolve that task-category conflict; this adoption preserves it.

Targeted proof includes changed installer tests, package/release dry or rehearsal tasks, strict OpenSpec validation, and project render diagnostics (the intentional CI hold digest is expected). No full lane, push, release or Hub checkout mutation is authorized.
