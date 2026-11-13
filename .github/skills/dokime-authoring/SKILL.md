---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

Read [Architecture](../../../docs/architecture.md) for current invariants and the
locked operation grammar. Author plugins with the product SDKs; Rust product
SDK dependencies stop at protocol. The host owns execution sequencing and cleanup.

- [Plugins](references/plugins.md): SDKs, catalog operations and bundle admission.
- [Suites](references/suites.md): materialization, lifecycle and evidence ownership.

The [Rust SDK](../../../docs/rust-sdk.md), [Python SDK](../../../sdk/python/README.md),
[C# SDK](../../../docs/sdk-csharp.md) and [C++ SDK](../../../docs/sdk-cpp.md) own
language examples. Generate projects with `dokime-next new <path> --lang <language>`.
Use explicit invocation scope and operation visibility, pure materialization,
execution-local resources and required cleanup. Query/document builders retain
protocol values; supported variants are documented in the execution-host guide.
