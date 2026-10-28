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
