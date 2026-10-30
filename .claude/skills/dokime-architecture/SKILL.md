<!-- Rendered by stkr project update from .agents/skills/dokime-architecture/SKILL.md; sha256:8304f5ba46c171becec69258aa2bdc957776183476086e8e0f8ea7c77ec00ab0 -->

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

Declare data contracts once in `dokime-domain` and operations, their policies and catalogs once in `dokime-operations`; its optional `runtime` feature dispatches through host-provided resources without depending on the runtime crate, which owns resources, execution, evidence and storage. Interfaces consume the `dokime` entry library and expose those declarations without redeclaration or forwarding implementations. Data-only consumers depend on `dokime-domain`; contract-only operation consumers use `dokime-operations` with `runtime` disabled.

Host plugin rows, contracts, cadences and document defaults belong to the definitions-only default of `dokime-host`, consumed explicitly without runtime re-exports. Application lifecycle records remain in `dokime::host`. `PluginOwner::Host` identifies embedding application catalogs, presented as Built-in.

Mutable resources register as `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Server owners prepare their composition before launching Dokime and acquire the activated service afterward; listeners do not register handlers. Dokime initialization validates recovered ownership, starts record capture and commits startup recovery before assembling its live handle. Harmos invokes storage reconciliation after application initialization, so that callback cannot own prerequisites of the initialized handle.

The backend owns the durable external-effect journal and execution cleanup. Plugins durably prepare owner-qualified inverses before external writes and confirm native completion timestamps; run cleanup closes write admission and restores in reverse order while evidence capture stays open. After plugin admission, startup restores pending inverses before interrupted executions are recovered or plugins are exposed; a missing or failed inverse refuses startup and remains pending. Read-only `RunStart` contributions capture structured execution evidence before steps through the ordinary plugin boundary. The host plugin owns native controls and inventory; the backend has no host implementation dependency.

Host telemetry belongs to `plugins/sidecars/host`, shipped as `host.dokime`; the runtime ingests it through the common plugin boundary and carries no host collector feature or dependency. OBC under `plugins/sidecars/obc` is test-only. Installer manifests select bundles only through top-level `[[plugin]]` entries with exactly one of `name` (selected release's format 2 feed) or `path` (relative to the organization pack, with no fallback to base plugin files). Repeatable `--plugin` arguments add equivalent external inputs relative to the invocation directory. The pack's `plugins/` folder is storage only: unlisted files never ship, and plugin files are not discovered or overlaid. Distribution and sidecars archives are verified against release SHA256SUMS; selected plugin members are also verified against feed digests. Bundled packages install through the ordinary plugin store at startup when absent or older; equal or newer installed versions are retained, including their settings and disabled state.
