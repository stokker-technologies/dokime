---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Declare product contracts once in `dokime-domain` and implement handlers in `dokime-runtime`. Interfaces consume the `dokime` entry library and expose the same operation contracts without redeclaration or forwarding implementations. Host-free consumers depend on `dokime-domain` directly.

Host plugin rows, contracts, cadences and document defaults belong to the definitions-only default of `dokime-host`, consumed explicitly without runtime re-exports. Application lifecycle records remain in `dokime::host`. `PluginOwner::Host` identifies embedding application catalogs, presented as Built-in.

Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers. Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

Host telemetry belongs to `plugins/sidecars/host`, shipped as `host.dokime`; the runtime ingests it through the common plugin boundary and carries no host collector feature or dependency. OBC under `plugins/sidecars/obc` is test-only. Installer bundles come from the resolved organization pack's optional `plugins/` folder and repeatable `--plugin` arguments. Bundled packages install through the ordinary plugin store at startup when absent or older; equal or newer installed versions are retained, including their settings and disabled state.
