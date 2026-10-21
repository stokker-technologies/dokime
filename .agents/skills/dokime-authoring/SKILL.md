---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

Read the guide for the contribution being changed:

- [Plugins](references/plugins.md): lifecycle, operation grants, contributions, settings and supervision.
- [Suites](references/suites.md): suite catalogs, parameters, steps, resources, prompts, records and cleanup.
- [Documents](references/documents.md): typed plots, captured evidence and sealed reports.

Use the current domain declarations and maintained starter as the source of exact signatures. A plugin declares its own private operations in `domain/`; a shared operation belongs to the shared domain crate. Runtime implementations mirror their declaration paths. Match the same kind-first model, operations, transactions and state layout as the product.

Plugins depend on the `dokime` entry library with feature `plugin`. Import authoring APIs from `dokime::` and shared definitions from `dokime::domain`; do not depend directly on internal runtime or domain crates.

`suites/mod.rs` contains only the suite catalog and module declarations. Each suite owns a folder module. Shared suite code lives under `suites/common/`; keep no loose implementation files at the suites root. Templates render the same layout.

A plugin repository keeps its mise tasks, provider workflows, managed Python runner and skeletons current through `stkr project update`. The starter is refreshed from the Hub plugin profile; never hand-edit rendered templates. Product stories generate the starter and exercise its declared tasks before delivery.
