# Stable Remote Listener

## Agreed Contract

Names refresh adds missing currently advertised or configured names and retains covered names when addresses disappear or reorder. New pairings advertise all eligible endpoints, so each current advertised name must be covered. Leaf replacement preserves the CA and reloads in place. Only enabled/address/TLS/auth changes rebind. Heartbeat is cancelled and joined before management closes. Rust/Python safe reads retry UNAVAILABLE once within the existing request deadline; resumable listeners retain their existing policies. Mutations without established request-id deduplication are never retried. Harmos API is unavailable at the current pin: use an untracked local patch to C:/Users/ricca/Work/repositories/github/stokker-technologies/harmos.wt/tlsreload during implementation. The orchestrator must publish Harmos, pin its verified revision and remove the override before final validation; committed Cargo.toml/Cargo.lock stay on the current revision.

## Ownership And Dependency Order

Harmos implements and proves certificate replacement first; Dokime adopts locally, proves remote refresh and client recovery, and commits without pushing. Each repository retains its own change.

## Upstream Capability And Adoption

`crates/dokime-server/src/grpc/remote/mod.rs` previously detached the Harmos
attachment whenever `store::host_names` changed. The pinned Harmos `attach(settings)`
API accepts a fixed Tonic TLS identity and provides no live certificate update.
The smallest new capability is a shared certificate resolver and a TLS attachment
that consumes it without replacing fixed-TLS consumers:

```rust
let certificate = harmos::grpc::ServerCertificate::from_pem(certificate_pem, key_pem)?;
let listener = runtime.grpc().attach_with_certificate(settings, certificate.clone(), handshake_timeout).await?;
certificate.reload(next_certificate_pem, next_key_pem)?;
```

The upstream change landed in Harmos as `ea6c9fd8d0bbc8560f23e5d6ca6af2ac3225e370`;
the three root Harmos dependencies, the README revision and the committed
lockfile pin that revision.
