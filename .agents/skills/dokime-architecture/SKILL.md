---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Apply `stkr-architecture` and `stkr-rust-architecture` for the shared tier model.

| Crate | Ownership And Dependencies |
| --- | --- |
| `dokime` | Consumer entry library; depends on domain and runtime, with optional client support. |
| `dokime-domain` | Portable models, records, streams, prompts and settings contracts; no runtime dependency. |
| `dokime-operations` | Operation declarations, policies and catalogs over domain; its optional `runtime` feature dispatches through host-provided resources without depending on the runtime crate. |
| `dokime-runtime` | Host resources, execution, evidence and storage; depends on domain and operations, with optional macros. |
| `dokime-cli` | Command interface and the `dokime` executable, including `dokime server`; depends on the entry library and server library. |
| `dokime-gui` | Desktop interface; depends on the entry library. |
| `dokime-server` | Server library owning session boot, listeners and bounded drain; shared by CLI and desktop composition, depends on the entry library. |
| `dokime-client` | Remote client implementation; depends on domain and operations, exposed through the entry library's `client` feature. |
| `dokime-packaging` | Packaging implementation; depends on the entry library. |
| `dokime-macros` | Authoring declarations; no production runtime dependency. |

The locked extraction in `prototype-streamline` assigns portable contracts to domain, behavior per operation file to operations, resources to runtime, third-party authoring to SDK, and host assembly to the facade. SDK and further engine/resource extraction remain the work of 9.0.5 and 9.2; the table above describes the landed crates consumed today.

The entry library and library/SDK crates permit Cargo publication; registry availability is a release result. CLI, desktop shell, server executable and Cargo subcommand owners declare `publish = false` and ship through installers and target-specific tool archives.

Host plugin rows, contracts, cadences and document defaults belong to the definitions-only default of `dokime-host`, consumed explicitly without runtime re-exports. Application lifecycle records remain in `dokime::host`. `PluginOwner::Host` identifies embedding application catalogs, presented as Built-in.

Declare data contracts once in `dokime-domain` and operations once in `dokime-operations`. Runtime composition supplies resources to the operation implementations enabled by the `runtime` feature. Interfaces consume `dokime` and expose those declarations through their hosts without redeclaring them. Data-only consumers depend on `dokime-domain`; contract-only operation consumers use `dokime-operations` with `runtime` disabled.

Dokime admits native plugins: supervised processes over piped stdio. Embed a product runtime through its entry library by default; use a separately hosted runtime sidecar on the same machine only when process isolation is required. Requirements select operations by contract and Public visibility; ambiguous providers are refused.

Applications launch through `harmos::Runtime::<A>::launch(origin, config)`. `Lifecycle::prepare(config, &mut Setup<Self>)` declares catalogs, lanes and resources before launch; `initialize` activates prepared resources against the live runtime. Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers.
Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

Storage stays at development v1. Use a fresh storage root for incompatible schemas; never silently delete or reset existing data.

Host telemetry belongs to `plugins/sidecars/host`, shipped as `host.dokime`.
The runtime ingests it through the common plugin boundary and carries no host
collector feature or dependency. OBC under `plugins/sidecars/obc` is test-only.
Installer bundles come from the resolved organization pack's optional `plugins/`
folder and repeatable `--plugin` arguments; the Dokime packaging task supplies the
host bundle explicitly. Packaging verifies member digests, target and the bundle's
declared Dokime requirement before embedding. Bundled packages install through the
ordinary plugin store at startup when absent or older; equal or newer installed
versions are retained, including their settings and disabled state.
