<<<<<<< HEAD
<!-- Rendered by stkr project update from .agents/skills/dokime-architecture/SKILL.md; sha256:d7fe60afccba0c152ffa99aa5476ecd91a513a9168f7dac6fffdd9b044306fd1 -->
=======
<!-- Rendered by stkr project update from .agents/skills/dokime-architecture/SKILL.md; sha256:8e624c711646dc6b462f41de79c1bb9ab98f3a4c1e15c314776d9b93d3dcb52f -->
>>>>>>> 11aeba01 (feat(runtime): journal and restore external execution effects)

---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---

# Dokime Architecture

Declare product contracts once in `dokime-domain` and implement handlers in `dokime-runtime`. Interfaces consume the `dokime` entry library and expose the same operation contracts without redeclaration or forwarding implementations. Host-free consumers depend on `dokime-domain` directly.

Host plugin rows, contracts, cadences and document defaults belong to the definitions-only default of `dokime-host`, consumed explicitly without runtime re-exports. Application lifecycle records remain in `dokime::host`. `PluginOwner::Host` identifies embedding application catalogs, presented as Built-in.

Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers. Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

<<<<<<< HEAD
Host telemetry belongs to `plugins/sidecars/host`, shipped as `host.dokime`; the runtime ingests it through the common plugin boundary and carries no host collector feature or dependency. OBC under `plugins/sidecars/obc` is test-only. Installer manifests select bundles only through top-level `[[plugin]]` entries with exactly one of `name` (selected release's format 2 feed) or `path` (relative to the organization pack, with no fallback to base plugin files). Repeatable `--plugin` arguments add equivalent external inputs relative to the invocation directory. The pack's `plugins/` folder is storage only: unlisted files never ship, and plugin files are not discovered or overlaid. Distribution and sidecars archives are verified against release SHA256SUMS; selected plugin members are also verified against feed digests. Bundled packages install through the ordinary plugin store at startup when absent or older; equal or newer installed versions are retained, including their settings and disabled state.
=======
Dokime admits native plugins: supervised processes over piped stdio. Embed a product runtime through its entry library by default; use a separately hosted runtime sidecar on the same machine only when process isolation is required. Requirements select operations by contract and Public visibility; ambiguous providers are refused.

Applications launch through `harmos::Runtime::<A>::launch(origin, config)`. `Lifecycle::prepare(config, &mut Setup<Self>)` declares catalogs, lanes and resources before launch; `initialize` activates prepared resources against the live runtime. Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers.
Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

The backend owns the durable external-effect journal and execution cleanup. Plugins durably prepare owner-qualified inverses before external writes and confirm native completion timestamps; run cleanup closes write admission and restores in reverse order while evidence capture stays open. After plugin admission, startup restores pending inverses before interrupted executions are recovered or plugins are exposed; a missing or failed inverse refuses startup and remains pending. Read-only `RunStart` contributions capture structured execution evidence before steps through the ordinary plugin boundary. The host plugin owns native controls and inventory; the backend has no host implementation dependency.

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
>>>>>>> 11aeba01 (feat(runtime): journal and restore external execution effects)
