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

## 5. Tests And Documentation

- [ ] 5.1 Add focused unit, integration and acceptance tests for conversions, ownership, shutdown and errors.
- [ ] 5.2 Add `test:python:system` with the end-to-end story on Windows and Linux, wired into Nightly only.
- [ ] 5.3 Add Python examples, and update the README, the comparison table and the remote and plugin guides.
- [ ] 5.4 Update `AGENTS.md` and `docs/tooling.md` where the package changes project structure or tasks.
- [ ] 5.5 Add the `dokime new --lang python` template with `main.py` and `plugin/`.

## 6. Final Validation

- [ ] 6.1 Run the targeted checks and review the full diff, including required-term scans.

## Workflow Follow-Up

- This slice owns 4.1–4.6 on `feat/dokime-python`: embedded operation, observer and settings composition; native callback and column publication bindings; Python decorators, validation and stubs; finished-execution GUI availability; targeted installed-wheel and GUI proof, and affected reference documentation. Dependency order is native provider composition, callback bridge, declarations and columns, then built-wheel proof. Tasks 5.x remain open.
- Slice proof: a Python operation, observer and settings exercised against the Windows wheel; callback cancellation and tracebacks; NumPy and Arrow batch publication and a plain Rust comparison; touched-surface stubtest, GUI availability tests, `check:rust`, repository checks and the required diff scan. Record decisions and measurements in `design.md`. Push without PR, dev merge or full simulation; later tasks stay open.
- Task 6.1 retains the existing native source-policy repairs; this slice does not waive them.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- The owner decides the PyPI and import name before any publication; a final `dokime-client` release then points to the new package.
- Depends on the separate rule that completed executions and cycles do not depend on their plugins.
- Packaging Python plugins as distributable bundles stays out of this change.
