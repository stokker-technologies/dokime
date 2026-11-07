# Tasks

## 1. Foundations

- [x] 1.1 Derive the storage format identity, verify it covers every stored encoding, and refuse mismatched roots in every backend.
- [x] 1.2 Create `sdk/python` with maturin, abi3, `_native`, `py.typed` and declared layout exceptions, outside default builds.
- [x] 1.3 Build one wheel per platform after the product build through the machine wrapper; measure build reuse and wheel size.
- [x] 1.4 Replace the pure-wheel CI, package and release steps with per-platform wheels.

## 2. Client Parity

- [x] 2.1 Wrap the Rust client with awaitable, cancellable Python operations and typed errors.
- [x] 2.2 Port evidence queries, finite readers, dataframes, profiles and pairing behaviour.
- [x] 2.3 Write stubs for the client surface and wire stubtest against the built extension.
- [x] 2.4 Move the client tests to the new package, removing redundant ones.
- [x] 2.5 Delete `packages/dokime-client`, its generated wire code and its tooling; migrate consumers.

## 3. Backend

- [x] 3.1 Let `Session::start` accept embedded providers and expose pairing export.
- [x] 3.2 Add the Backend async context, port and root configuration, and typed startup errors.
- [x] 3.3 Load installed plugins and refuse Python plugin IDs that clash with them.

## 4. Plugin Authoring

- [x] 4.1 Extend `EmbeddedProvider` with operations, observers and settings sections.
- [x] 4.2 Implement the Rust-to-Python callback bridge: event loop, worker threads, cancellation and tracebacks.
- [x] 4.3 Implement the decorators and registration validation from the mapping table.
- [x] 4.4 Implement batch stream publication from NumPy and Arrow columns.
- [x] 4.5 Show an unavailable plugin clearly in the GUI for finished Python executions.
- [x] 4.6 Write stubs for the backend and authoring surface.

## 5. Tests and Documentation

- [x] 5.1 Add focused unit, integration and acceptance tests for conversions, ownership, shutdown and errors.
- [x] 5.2 Add `test:python:system` with the end-to-end story on Windows and Linux, wired into Nightly only.
- [x] 5.3 Add Python examples, and update the README, the comparison table and the remote and plugin guides.
- [x] 5.4 Update `AGENTS.md` and `docs/tooling.md` where the package changes project structure or tasks.
- [x] 5.5 Add the `dokime new --lang python` template with `main.py` and `plugin/`.

## 6. Final Validation

- [ ] 6.1 Run the targeted checks and review the full diff, including required-term scans.

## Workflow Follow-Up

- Current owner-locked slice: 5.1–5.5 and targeted 6.1. Source inventory: `sdk/python` declarations/conversion and callback tests, installed-wheel acceptance and system stories, reusable starter examples; `crates/dokime-cli` Python project generation and its targeted proof; `mise.toml`, `tools/project.toml`, Nightly and its simulation graph for the Python system lane; README, SDK, remote/plugin/product/tooling guides and root `AGENTS.md`. Dependencies: merged dev, starter and focused tests, system/CI wiring, documentation, then built-wheel and requested targeted proof. Repository-policy repairs are limited to files owned by this change; dev's later sweep owns other findings.
- Delivery for this slice: push `feat/dokime-python` without force, PR, dev merge, full simulation or `stkr specification finish`. Keep the change open during the owner's build freeze. Linux system execution remains required in Nightly; this slice's local system proof runs once on Windows. Re-measure the existing Python/native batch workload after the dev merge and separate bridge work from admitted storage work.
- Tasks 5.1–5.5 are source-complete in the combined feature worktree with no delegated writers. Targeted proof covers portable values, callback/resource ownership, cancellation/errors, installed-wheel column conversion and refusal, and the Windows generated-project system story. The requested targeted Rust/repository checks, refreshed built-wheel proof, diff review and post-ingest-fix measurements are recorded in `design.md`; the residual native audit and Linux execution keep 6.1 open.
- Task 6.1 remains open for the residual native source-policy audit; the owner explicitly assigns inherited dev findings to a later sweep. Do not widen this slice into that sweep. Strict installed-wheel stubtest is the executable binding proof; no native-mapper failure is represented as a passing audit.
- The owner decides the PyPI and import name before any publication; a final `dokime-client` release then points to the new package.
- Depends on the separate rule that completed executions and cycles do not depend on their plugins.
- Packaging Python plugins as distributable bundles stays out of this change.
