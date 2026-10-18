---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Apply the shared architecture and Rust rules. The accepted boundary is `dokime-domain` for definitions and pure domain behaviour, `dokime-runtime` for resource-bound behaviour, `dokime-server` and `dokime-gui` for shells, `dokime-client` for remote access, `dokime-packaging` for code generation and packaging, and the bare `dokime` crate for the installable CLI. During an in-progress cutover verify landed paths before citing an API.

The domain crate contains every operation, transaction, state entry, message and value definition. It performs no I/O, async work, storage or network access. Runtime crates implement the resource-bound behaviour and depend on domain crates; domain crates never depend on runtimes. Sidecars, plugins, clients and tools use the portable domain surface.

Organize declarations kind-first: `model/`, `operations/`, `transactions/`, `state/`. Each kind's `mod.rs` is its closed catalog, with folder modules by domain. Runtime implementations mirror the same domain names and paths. A plugin's private definitions live in its `domain/` module; move a declaration to a shared domain crate when another party consumes it typed.

Records select by envelope identity: scope is the cycle, item is the execution (or cycle for cycle context), source is the provider/process epoch. Use typed reads for records and retain columnar queries for plots and telemetry derivations. Lifecycle capture references the durable frozen execution revision rather than duplicating it.

Packaging depends on portable domains and Harmos packaging, never the runtime. Build scripts request only enabled bindings (`bindings-proto`, `bindings-python`, `bindings-typescript`); `grpc` implies proto. The thin cargo tool uses the canonical CLI and packaging instead of embedding a second runtime.

Storage is development v1. A clean schema break may require a fresh storage root; never silently reset existing data. Plugin installation and enablement become active after a full app restart; development reload remains an explicit separate lifecycle.
