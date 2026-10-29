<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/references/plugins.md; sha256:1abf1aa7dfb91012621dc79217a95610dccf890c94128d4e09c196bce234628c -->

# Dokime Plugin

A Dokime plugin is a native process launched and supervised by the host over piped stdio.

Embed the product runtime through the product entry library by default. Declare its handler catalog in the plugin's `const CATALOGS`; the plugin owns suites, templates, observers, listeners and its own records and streams.

When crash isolation or a native library requires a separate process, host that runtime as a sidecar on the same machine with the same operation contracts. The plugin requires those operations and depends only on that product's domain crate, beside its Dokime authoring dependency.

Declare operation requests and replies, records, streams, prompts and settings once in their domain owner. Runtime handlers implement those contracts. Shared visibility permits another provider to satisfy a requirement; ambiguous providers are refused.

Contributions name only populated catalogs: `Suites`, `Records`, `Evidence` and `Settings`. Template records, prompts and streams belong to their template declarations; plugin contributions cover evidence outside those declarations. Observers publish evidence; listeners receive their declared record types.

The plugin declaration implements `dokime::Plugin`; resource ownership implements the shared `dokime::harmos::Lifecycle` separately. Its async `prepare(config, &mut Setup<Self>)` registers shared resources and prepares suites with `dokime::execution::prepare::<Self, Suites>(setup)`. `initialize` activates those resources, followed by serving, `stop` and `cleanup`. Mutable resources use `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Observers receive an execution context and read named resources through `context.resource::<T>()`.

## Layout

```text
src/
  domain/                       Plugin-Owned Contracts
  operations/                   Provided Handlers
  observers/
  listeners/
  records/
  streams/
  settings/
  suites/
    mod.rs                      Suites Catalog
    <suite>/
      mod.rs                    Suite Declaration
      <template>.rs             Template Declaration
```

Use unprefixed suite names and only populated modules. Keep the selected repository skeleton and all its section markers. Keep declarations explicit, with no prelude modules or placeholder suites.

The host sidecar keeps one request definition per `host/<operation>/mod.rs`:
request struct, id, policy, response and contract tests. Runtime implementations
live in parallel `windows.rs` and `unix.rs` modules selected only by
`#[cfg(all(feature = "runtime", target_os = "windows"))] mod windows;` and
`#[cfg(all(feature = "runtime", unix))] mod unix;`. Gate module declarations and
re-exports, never implementations or catalog variants. One `host/catalog.rs`
assembles every operation on both platforms. The telemetry requests also carry
their stream and observer declarations, so those inventories share the catalog.
Preparation registers mutable collector resources as `Arc<tokio::sync::Mutex<T>>`;
operation policies own collection deadlines and configuration owns probe parameters.
