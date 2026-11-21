---
sidebar: { order: 3 }
---
# Dokime Plugins

Plugins expose operations and typed catalogs over gRPC. The host materializes
cases and owns their execution loop; plugin steps receive invocation context.
Managed executable and remote URL placements share these contracts.

## Authoring

Use the [Rust](../sdks/rust.md), [Python](../../sdk/python/README.md),
[C#](../sdks/csharp.md) or [C++](../sdks/cpp.md) SDK. Each owns its starter. The current
CLI embeds those exact starter sources:

```sh
dokime new bench --lang rust
dokime new bench --lang python
dokime new bench --lang csharp
dokime new bench --lang cpp
```

Execution-local resources belong to the SDK's context/resource mechanism;
Python keeps them on the suite instance. Initialize acquires resources and cleanup
releases them, including after cancellation. Materialization must remain pure.
Typed records, samples, prompts, progress and logs use scoped host calls.
Every SDK declares a template's steps, in order, on its catalog entry
(`TestCaseTemplate.steps`), so the desktop library shows the procedure before
any case exists; materialization can still reshape a case's steps.
The [Execution Host Guide](execution-host.md) distinguishes served context calls
from the remaining locked context work.

## Bundles and Admission

```sh
dokime plugin bundle --executable build/plugin --out plugin.dokime
dokime plugin inspect --bundle plugin.dokime
```

Bundle production describes the executable, freezes its catalog and builds a v1
tar manifest with per-file sizes/digests and product compatibility. Shared bundle
inspection belongs to `crates/dokime-packaging`; host admission independently
checks inventory, paths, compatibility and description before starting a process.
See [Plugin Security](plugin-security.md). Startup/stop bounds come from placement
settings; invocations retain their own operation policies.
