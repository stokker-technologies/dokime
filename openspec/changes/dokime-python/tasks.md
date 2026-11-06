# Tasks

## 1. Foundations

- [ ] 1.1 Derive the storage format identity, verify it covers every stored encoding, and refuse mismatched roots in every backend.
- [ ] 1.2 Create `sdk/python` with maturin, abi3, `_native`, `py.typed` and declared layout exceptions, outside default builds.
- [ ] 1.3 Build one wheel per platform after the product build under the machine lock; measure build reuse and wheel size.
- [ ] 1.4 Replace the pure-wheel CI, package and release steps with per-platform wheels.

## 2. Client Parity

- [ ] 2.1 Wrap the Rust client with awaitable, cancellable Python operations and typed errors.
- [ ] 2.2 Port evidence queries, finite readers, dataframes, profiles and pairing behaviour.
- [ ] 2.3 Write stubs for the client surface and wire stubtest against the built extension.
- [ ] 2.4 Move the client tests to the new package, removing redundant ones.
- [ ] 2.5 Delete `packages/dokime-client`, its generated wire code and its tooling; migrate consumers.

## 3. Backend

- [ ] 3.1 Let `Session::start` accept embedded providers and expose pairing export.
- [ ] 3.2 Add the Backend async context, port and root configuration, and typed startup errors.
- [ ] 3.3 Load installed plugins and refuse Python plugin IDs that clash with them.

## 4. Plugin Authoring

- [ ] 4.1 Extend `EmbeddedProvider` with operations, observers and settings sections.
- [ ] 4.2 Implement the Rust-to-Python callback bridge: event loop, worker threads, cancellation and tracebacks.
- [ ] 4.3 Implement the decorators and registration validation from the mapping table.
- [ ] 4.4 Implement batch stream publication from NumPy and Arrow columns.
- [ ] 4.5 Show an unavailable plugin clearly in the GUI for finished Python executions.
- [ ] 4.6 Write stubs for the backend and authoring surface.

## 5. Tests And Documentation

- [ ] 5.1 Add focused unit, integration and acceptance tests for conversions, ownership, shutdown and errors.
- [ ] 5.2 Add `test:python:system` with the end-to-end story on Windows and Linux, wired into Nightly only.
- [ ] 5.3 Add Python examples, and update the README, the comparison table and the remote and plugin guides.
- [ ] 5.4 Update `AGENTS.md` and `docs/tooling.md` where the package changes project structure or tasks.

## 6. Final Validation

- [ ] 6.1 Run the targeted checks and review the full diff, including required-term scans.

## Workflow Follow-Up

- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- The owner decides the PyPI and import name before any publication; a final `dokime-client` release then points to the new package.
- Depends on the separate rule that completed executions and cycles do not depend on their plugins.
- Packaging Python plugins as distributable bundles stays out of this change.
