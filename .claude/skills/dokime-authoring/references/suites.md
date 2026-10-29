<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/references/suites.md; sha256:608dba1f7024b6564b2d3d351f7e45026d265225750e770155eabf6786f3507d -->

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

## File Capture Readiness

After submitting `WatchFile`, await `watch.armed().await?` before starting the producer. Readiness means the baseline and capture were admitted; an accessible empty directory is a valid baseline for `All` and `New`. The streaming `FileWatch` API reports the same boundary through its `Armed` frame. Join or cancel the watcher within the owning step.

Selection is independent of capture policy: `All` follows baseline files and arrivals, `Latest` selects the newest baseline file and follows its rotation, and `New` captures later discoveries from byte zero. Use `wait_for_source_seconds` only when an absent source is expected. Its deadline precedes the capture duration; cancellation, permission failures and invalid paths remain failures. An unavailable FTP root fails immediately because status 550 cannot distinguish absence from denied access.
