# Dokime Plugins

Plugins expose operations and typed catalogs over gRPC. The host materializes
cases and owns their execution loop; plugin steps receive invocation context.
Managed executable and remote URL placements share these contracts.

## Authoring

Use the [Rust](rust-sdk.md), [Python](../sdk/python/README.md),
[C#](sdk-csharp.md) or [C++](sdk-cpp.md) SDK. Each owns its starter. The current
CLI embeds those exact starter sources:

```sh
dokime-next new bench --lang rust
dokime-next new bench --lang python
dokime-next new bench --lang csharp
dokime-next new bench --lang cpp
```

Execution-local resources belong to the SDK's context/resource mechanism;
Python keeps them on the suite instance. Initialize acquires resources and cleanup
releases them, including after cancellation. Materialization must remain pure.
Typed records, samples, prompts, progress and logs use scoped host calls.
The [Execution Host Guide](execution-host.md) distinguishes served context calls
from the remaining locked context work.

## Bundles and Admission

```sh
dokime-next plugin bundle --executable build/plugin --out plugin.dokime
dokime-next plugin inspect --bundle plugin.dokime
```

Bundle production describes the executable, freezes its catalog and builds a v1
tar manifest with per-file sizes/digests and product compatibility. Shared bundle
inspection belongs to `crates/dokime-packaging`; host admission independently
checks inventory, paths, compatibility and description before starting a process.
See [Plugin Security](plugin-security.md). Startup/stop bounds come from placement
settings; invocations retain their own operation policies.
