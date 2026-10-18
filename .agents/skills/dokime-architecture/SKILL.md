---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Apply the shared architecture and Rust rules. The accepted boundary is `dokime-domain` for definitions and pure domain behaviour, `dokime-runtime` for resource-bound behaviour, `dokime-server` and `dokime-gui` for shells, `dokime-client` for remote access, `dokime-packaging` for code generation and packaging, and the bare `dokime` crate for the installable CLI. During an in-progress cutover verify landed paths before citing an API.

The domain crate contains every operation, transaction, state entry, message and value definition. It performs no I/O, async work, storage or network access. Runtime crates implement the resource-bound behaviour and depend on domain crates; domain crates never depend on runtimes. Sidecars, plugins, clients and tools use the portable domain surface.

Organize declarations kind-first: `model/`, `operations/`, `transactions/`, `state/`. Each kind's `mod.rs` is its closed catalog, with folder modules by domain. Runtime implementations mirror the same domain names and paths. A plugin's private definitions live in its `domain/` module; move a declaration to a shared domain crate when another party consumes it typed.

Records select by envelope identity: scope is the cycle, item is the execution (or cycle for cycle context), source is the provider/process epoch. Use typed reads for records and retain columnar queries for plots and telemetry derivations. Lifecycle capture references the durable frozen execution revision rather than duplicating it.

Packaging depends on portable domains and Harmos packaging, never the runtime. Build scripts request only enabled generators (`generate-proto`, `generate-python`, `generate-typescript`); `grpc` implies proto. The thin cargo tool uses the canonical CLI and packaging instead of embedding a second runtime.

Storage is development v1. A clean schema break may require a fresh storage root; never silently reset existing data. Plugin installation and enablement become active after a full app restart; development reload remains an explicit separate lifecycle.

## Accepted Remote Cutover

The remote-streamline implementation is migrating to explicit container phases: Active, Sealing, Sealed. Several cycles may be Active; one server-wide execution lane runs admitted requests in acceptance order. A request captures its complete execution selection at admission, including all provider groups. Later cycle additions do not join that selection. Execution completion seals its evidence independently and leaves the container Active. Sealing closes admission immediately, stops only that cycle's work, cancels unrun members and waits for durable execution and cycle seals. Saved seals are immutable. Startup seals previous-session cycles without replaying device work before accepting new work; shutdown seals all active containers. Aggregate run activity and outcomes do not control container admission. Verify landed source before using these accepted contracts as available APIs.

The accepted remote-access cutover keeps native local and remote listener ownership independent. Receiving listener identity controls local-only operation admission and host-path disclosure; a remote call through loopback never inherits native trust. The server shell owns listener settings, TLS material, key verification and listener restart/rollback. The runtime owns admitted executions, so listener changes and key rotation cannot cancel them. Persist the certificate/private key and key verifier under the application root; expose a generated full-control access key once and the public certificate separately. TLS and authentication are independent explicit choices. Retained certificate identities change only through explicit regeneration, with clients updating trust. Preserve plugin-to-plugin capability requirements when replacing remote user IAM. These contracts require supported Harmos listener and external-authorization capabilities; source-only projections do not establish available or working remote access.
