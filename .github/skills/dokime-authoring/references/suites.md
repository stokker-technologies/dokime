<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/references/suites.md; sha256:dffc531e5aa7619aaf6eb02c0fa3c106c5a9b44d17d2057736566f43e24a9b54 -->

# Dokime Suite

## Suite Authoring

Declare suites with `#[dokime::suite]` and templates with `#[dokime::template]`. A suite owns its template collection; each template owns its parameters and lifecycle state.

Name template identifiers `<suite>.<segment>`, with lowercase segments matching `[a-z][a-z0-9_]*`. Labels use Title Case and tags use lowercase. Name capability catalogs `Required`, `Subscriptions`, `Streams` and `Prompts`.

```rust
#[dokime::template(
    id = "soak.hold",
    label = "Hold Temperature",
    parameters = HoldParameters,
    requires = Required,
    subscribes = Subscriptions,
    streams = Streams,
    prompts = Prompts,
)]
```

A template declares operation requirements without naming a plugin. Its serving plugin provides or requires those contracts. Declare each prompt, record and stream once; catalogs refer to the declared types.

## Lifecycle

Use the vocabulary in this order: initialize, traceability, steps, criteria, observers, cleanup, document. Observers run concurrently with steps; this vocabulary order does not serialize them after criteria.

| Stage | Capabilities |
| --- | --- |
| initialize | Log, artifact grant, document additions; yields template state and shared handles. |
| traceability | Log, artifact grant, document additions; captures traceability using shared handles. |
| steps | Log, prompt, listen, publish, stream, artifact grant, document additions. |
| criteria | Log and captured evidence for decisions; no prompt, listen, publish, artifact grant or document mutation. |
| observers | Log, publish, stream, artifact grant, document additions; read shared handles only, never template state. |
| cleanup | Log, artifact grant, document additions; termination reason and shared handles for restoration. |
| document | Declared sections, cards and plots over captured evidence. |

Keep handles used only by steps in template state. Return observer-shared handles beside that state during initialize.

## Layout

Keep `suites/mod.rs` as the suite catalog and module declarations. Put each template in `suites/<suite>/<template>.rs`, with its suite declaration in `suites/<suite>/mod.rs`. Suite names carry no product prefix. Retain the repository skeleton and its section markers without an additional item-order rule.
