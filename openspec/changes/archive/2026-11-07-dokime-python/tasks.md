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
- [x] 3.2 Add the explicit Backend start/close lifecycle, port and root configuration, and typed startup errors.
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

- [x] 6.1 Run the targeted checks and review the full diff, including required-term scans.

## Workflow Follow-Up

- Completed source inventory: native SDK, facade/server integration, client replacement, Python starter, Windows/Linux Nightly wiring, build/package/release ownership, installation guidance, SDK audit repairs, bounded paging regressions and bench measurements. Publication and workspace retirement are separate authorized delivery operations. No full simulation or WatchFile redesign is part of this change.
- Final SDK checks, full diff review, required-term scan, native SDK export/ownership audit repairs, Linux wheel/system proof and bench measurements are performed and recorded in design.md. The repository-wide native audit still fails for baseline findings assigned to the separate sweep; no passing full native audit or full simulation is claimed. Native sample paging regressions use small pages through retained seals, without timing assertions.
- Registry publication remains blocked on the owner's naming decision. Native release wheels are the installation path for this delivery.
- Packaging Python plugins as distributable bundles stays out of this change.
