---
name: dokime-conventions
description: Apply Dokime domain names, feature names, suite layout and product-specific source conventions.
---

# Dokime Conventions

Apply shared code, Rust and frontend conventions. Cargo feature names are words: `simulation`, not abbreviations. Features are additive and preserve public bounds and signatures at both checked edges.

Use `obc-plugin` for the shipped onboard charger demonstrator, its bundle and its executable. Its suite identifiers begin with `obc.`. Catalogs live in their kind's `mod.rs`; maintain one owner per definition and migrate consumers when replacing an internal API.

A suite is a self-contained folder module; `suites/mod.rs` holds the catalog only and shared suite code lives in `suites/common/`. The product starter uses the same shape.

Project-local guidance and these four skills are canonical under `.agents/`. Regenerate provider copies with `stkr project update`; do not maintain competing copies. Keep runtime examples aligned with the source currently published, and distinguish accepted cutover direction from available APIs.
