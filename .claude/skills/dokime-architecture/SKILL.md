---
name: dokime-architecture
description: Define Dokime domain, runtime, shell and packaging ownership and lifecycle boundaries.
---
<!-- Rendered by stkr project update from .agents/skills/dokime-architecture/SKILL.md; sha256:286d0fa0d534a08a915d90bf4386a4f0bbcad0dc406a295caff5945557aa6ae5 -->

# Dokime Architecture

Apply `stkr-architecture` and `stkr-rust-architecture` for the shared tier model.

| Crate | Ownership And Dependencies |
| --- | --- |
| `dokime` | Consumer entry library; depends on domain and runtime, with optional client support. |
| `dokime-domain` | Host-free operation requests/replies, records, streams, prompts and settings contracts; no runtime dependency. |
| `dokime-runtime` | Handlers, execution, evidence and storage; depends on domain and optionally macros. |
| `dokime-cli` | Command interface and the `dokime` executable; depends on the entry library. |
| `dokime-gui` | Desktop interface; depends on the entry library. |
| `dokime-server` | Server interface; depends on the entry library. |
| `dokime-client` | Remote client implementation; depends on domain, exposed through the entry library's `client` feature. |
| `dokime-packaging` | Packaging implementation; depends on the entry library. |
| `dokime-macros` | Authoring declarations; no production runtime dependency. |

These crates permit Cargo publication; registry availability is a release result.

Declare product contracts once in `dokime-domain`; implement handlers in `dokime-runtime`. Interfaces consume `dokime` and expose the same operation contracts through their hosts without redeclaration or forwarding implementations. Host-free consumers depend on `dokime-domain` directly.

Dokime admits native plugins: supervised processes over piped stdio. Embed a product runtime through its entry library by default; use a separately hosted runtime sidecar on the same machine only when process isolation is required. Requirements select operations by contract and shared visibility; ambiguous providers are refused.

Storage stays at development v1. Use a fresh storage root for incompatible schemas; never silently delete or reset existing data.
