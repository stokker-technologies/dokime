# Plugin Security

## Trust Boundary

Install only trusted executable bundles and connect only to trusted plugin URLs.
Managed placement is process supervision, not sandboxing. A plugin retains its
operating-system authority. Never put credentials in a bundle or source archive.

## Admission

`crates/dokime-runtime-next/src/services/bundles.rs` verifies v1 bundle paths,
regular files, inventory sizes/digests, compatibility and native target before
execution. The described identity/catalog must match the retained description.
`services/managed_plugins.rs` owns replacement rollback and child cleanup;
`crates/dokime-packaging/src/bundle/mod.rs` provides current CLI inspection.
Description, operation fingerprints, explicit visibility and declared requirements
remain admission boundaries. See [Architecture](architecture.md) for implemented
proof and remaining library composition/peer-routing obligations.

## Secrets and Remote Access

Product secret storage uses protected OS credential slots and durable metadata;
saved settings are resolved on demand. Exported secret transfer files use bounded
Base64 JSON, not encryption. Remote access uses the composed library Access TLS,
pairing and certificate services, with one authorization layer for product services.
See [Remote Access](remote/overview.md), [Privacy](privacy.md) and [Security](SECURITY.md).

## Failure and Cleanup

Operation policy owns deadlines and retries. Stop reaches a safe boundary; Cancel
interrupts active work and retains required cleanup. Managed startup cancellation
waits for rollback and child reaping before reporting failure. A plugin restart
interrupts its active work; sealed evidence and frozen schemas remain readable.
