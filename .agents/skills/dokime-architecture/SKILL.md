---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Apply `stkr-architecture` and `stkr-rust-architecture` for the shared tier model.

| Crate | Ownership And Dependencies |
| --- | --- |
| `dokime` | Consumer entry library; depends on domain and runtime, with optional client support. |
| `dokime-domain` | Host-free operation requests/replies, records, streams, prompts and settings contracts; no runtime dependency. |
| `dokime-runtime` | Handlers, execution, evidence and storage; depends on domain and optionally macros. |
| `dokime-cli` | Command interface and the `dokime` executable, including `dokime server`; depends on the entry library and server library. |
| `dokime-gui` | Desktop interface; depends on the entry library. |
| `dokime-server` | Server library owning session boot, listeners and bounded drain; shared by CLI and desktop composition, depends on the entry library. |
| `dokime-client` | Remote client implementation; depends on domain, exposed through the entry library's `client` feature. |
| `dokime-packaging` | Packaging implementation; depends on the entry library. |
| `dokime-macros` | Authoring declarations; no production runtime dependency. |

The entry library and library/SDK crates permit Cargo publication; registry availability is a release result. CLI, desktop shell, server executable and Cargo subcommand owners declare `publish = false` and ship through installers and target-specific tool archives.

Declare product contracts once in `dokime-domain`; implement handlers in `dokime-runtime`. Interfaces consume `dokime` and expose the same operation contracts through their hosts without redeclaration or forwarding implementations. Host-free consumers depend on `dokime-domain` directly.

Dokime admits native plugins: supervised processes over piped stdio. Embed a product runtime through its entry library by default; use a separately hosted runtime sidecar on the same machine only when process isolation is required. Requirements select operations by contract and shared visibility; ambiguous providers are refused.

Applications launch through `harmos::Runtime::<A>::launch(origin, config)`. `Lifecycle::prepare(config, &mut Setup<Self>)` declares catalogs, lanes and resources before launch; `initialize` activates prepared resources against the live runtime. Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers.
Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

Storage stays at development v1. Use a fresh storage root for incompatible schemas; never silently delete or reset existing data.

Host telemetry belongs to `plugins/sidecars/host`, shipped as `host.dokime`.
The runtime ingests it through the common plugin boundary and carries no host
collector feature or dependency. OBC under `plugins/sidecars/obc` is test-only.
Installer bundles are selected solely by repeatable `--plugin` arguments; the
Dokime packaging task supplies the host bundle explicitly.
