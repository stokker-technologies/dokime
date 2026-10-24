<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/references/plugins.md; sha256:48b19d32611a2ca8d2f8d11c59d5bd54008bb2a8d6bd314709e48dfebc70c588 -->

# Dokime Plugin

A Dokime plugin is a native process launched and supervised by the host over piped stdio.

Embed the product runtime through the product entry library by default. Register its handler catalog as the plugin's provided operations; the plugin owns suites, templates, observers, listeners and its own records and streams.

When crash isolation or a native library requires a separate process, host that runtime as a sidecar on the same machine with the same operation contracts. The plugin requires those operations and depends only on that product's domain crate, beside its Dokime authoring dependency.

Declare operation requests and replies, records, streams, prompts and settings once in their domain owner. Runtime handlers implement those contracts. Shared visibility permits another provider to satisfy a requirement; ambiguous providers are refused.

Contributions name only populated catalogs: `Suites`, `Records`, `Evidence` and `Settings`. Template records, prompts and streams belong to their template declarations; plugin contributions cover evidence outside those declarations. Observers publish evidence; listeners receive their declared record types.

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
