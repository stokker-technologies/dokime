# Dokime Plugin

A Dokime plugin is a native process launched and supervised by the host over piped stdio.

Embed the product runtime through the product entry library by default. Declare its handler catalog in the plugin's `const CATALOGS`; the plugin owns suites, templates, observers, listeners and its own records and streams.

When crash isolation or a native library requires a separate process, host that runtime as a sidecar on the same machine with the same operation contracts. The plugin requires those operations and depends only on that product's domain crate, beside its Dokime authoring dependency.

Declare operation requests and replies, records, streams, prompts and settings once in their domain owner. Runtime handlers implement those contracts. Public visibility permits another provider to satisfy a requirement; ambiguous providers are refused.

A template lists its calls in its `requires` catalog. Its plugin's own `Private` or `Public` operations can answer without a plugin-level requirement. Calls to another plugin require a `Public` operation and a declared plugin requirement. `Host` operations cannot satisfy a template call.

Contributions name only populated catalogs: `Suites`, `Records`, `Evidence`, `RunStart` and `Settings`. Template records, prompts and streams belong to their template declarations; plugin contributions cover evidence outside those declarations. Observers publish evidence; listeners receive their declared record types.

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

The host sidecar keeps request definitions in `host/<domain>/mod.rs`:
request structs, ids, policies, responses and contract tests. `host/controls`
shares native state capture, inverse transactions and backend journal callbacks
for radio, adapter and power writes. Runtime implementations
live in parallel `windows.rs` and `unix.rs` modules selected only by
`#[cfg(all(feature = "runtime", target_os = "windows"))] mod windows;` and
`#[cfg(all(feature = "runtime", unix))] mod unix;`. Gate module declarations and
re-exports, never implementations or catalog variants. `host/catalog.rs` assembles collection and stream declarations;
`host/operations.rs` assembles Public queries and controls on both platforms.
`host/observers.rs` selects runtime observers, including stateful native
notification ownership; cancelling its demand activation drops registrations.
Preparation registers mutable collector resources as `Arc<tokio::sync::Mutex<T>>`;
operation policies own collection deadlines and configuration owns probe parameters.


## Run Effects And Start Evidence

`dokime::RunStart<Capture>` registers a default-constructed Host operation returning `dokime::operations::effects::StartEvidence`. The backend calls each enabled provider synchronously before any execution step, qualifies the route with its provider owner, and records the structured JSON snapshot in that execution's evidence. A refused capture refuses execution start. Keep capture read-only.

An operation that changes external state requires `ReadEnvironment`, `PrepareEffect` and `ConfirmEffect` from `dokime::operations::effects`. Read and validate the native prior state, prepare an owner-qualified encoded inverse durably before changing it, then confirm with the actual native completion timestamp. Empty tickets mean a standalone change outside an execution. The backend restores execution effects in reverse order after pass, failure or cancellation and restores persisted pending inverses before startup returns. Failed inverses remain pending and refuse startup; never reset storage to discard them. Native embedding owners use `RuntimeHandle::begin_external_change` and `complete_external_change` with the same journal.

Cancellation must retain native work and its serialization gate until it settles. A plugin retains its generation `Supervisor` during initialization and copies the serving execution scope for journal callbacks, so cancellation of the parent operation cannot strand its completion timestamp. Backend cleanup closes write admission before restoring; it keeps execution capture open through restoration. An inverse is a Host operation callable by the backend, not another plugin. Internal storage remains v1; an incompatible development store requires a fresh root without deleting the old one.
