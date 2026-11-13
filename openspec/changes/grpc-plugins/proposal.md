# gRPC Plugins

Status: protocol definitions and host-source export are implemented. Runtime,
SDK and catalog adoption remain open; follow the waves in `design.md`. Replaces `remote-plugins`. The Harmos change `grpc-plugins` is
the protocol side.

## Why

Today a plugin owns the run loop of every execution and talks to the host
through a private signal protocol that only Rust can speak. That makes the
plugin side heavy, ties plugins to bundles Dokime starts, and keeps Python
plugins inside a native extension. Moving the loop into the host turns a
plugin into a set of ordinary operations any language can serve over gRPC,
and lets Dokime delete the plumbing in between.

## What Changes

- A plugin is `Managed` (a bundle Dokime installs, starts and supervises) or
  `Remote` (a URL the user adds). Both are the same gRPC connection.
- The flow is `TestCaseTemplate` → expand (host) → `materialize` (plugin,
  pure) → `TestCase` → execute (host) → `Execution`.
- Executions are evidence containers (open, closed, sealed), with frozen schemas
  and Origin containing the run's TestCase or the import's Import definition.
  Run phases are manager-local live state, never a stored run or API type.
- Outcomes distinguish not evaluated, evaluated and importer-declared. Cycles
  fold only evaluated results and never report Passed without an evaluated result.
- Backend capabilities govern desktop controls. Run recovery excludes import
  drafts; report rendering is separate from publication for every execution.
- A `TestCase` names its steps, criteria and streams. Steps, criteria,
  `initialize` and `cleanup` are plugin operations the host calls; the host
  owns ordering, retries, cancellation, cleanup and verdicts.
- Everything a step or criterion needs from Dokime is a host operation called
  through `ctx`: prompts, secrets, settings, records, statistics, files.
- Samples arrive on streams the host opens. Records are acknowledged host
  operations.
- One Dokime protocol package, `dokime.v1`, holds every operation Dokime
  serves and the messages for the operations plugins serve.
- Python APIs offer flat value composition and explicit close, without requiring
  a context manager.
- Each `sdk/<language>` has two halves: provide plugins, and drive Dokime.
  The pilot covers Rust, Python, C# and C++.
- `sdk/rust` is the `dokime` crate itself, with plugin and client halves and
  protocol-only product dependencies; host embedding uses `dokime-runtime`.
- Protocol messages replace the domain model. Host operations co-locate their
  declarations, gRPC mappings and handlers; runtime owns host composition.
- Harmos accepts generated protobuf types in client-facing operation declarations.
  Server behavior moves into operations and the server crate is deleted.
- After the last gRPC wave, `storage-interface` extends Harmos storage first and
  extracts application persistence into `dokime-storage`, with benchmark proof.
- Python becomes a pure Python package, `Remote` only.
- Removed: `<template>.run` and `.signal`, the signal and evidence-page
  protocol, the execution hand-off layer, embedded plugins, the PyO3 SDK and
  Python backend, and recovery of a plugin restart during an execution.
  `design.md` lists the paths.

## Locked Decisions

- The host creates the `Execution`; a plugin never sees a run loop.
- `materialize` is pure: no instrument access and no side effects.
- `initialize`, `cleanup` and plugin state are per execution. Cycles only
  group executions.
- A plugin restart interrupts its live run, closes its execution and finalizes
  that execution and its affected cycles; import drafts are excluded.
  There is no resume. A dropped connection to the same process fails only the
  step in flight, which is retried when the step allows it.
- All samples retain source time, receipt time and durable position with a
  declared stream axis. Live plugin evidence remains arrival-ordered; import
  UTC normalization is owned by `imported-executions`.
- A remote plugin is trusted by whoever adds its URL. Bundles keep the
  existing install path and security roadmap.
- The host telemetry plugin stays a managed plugin.
- No throughput gate: design for simplicity, tune afterwards.
- Calls to other plugins are typed protobuf calls; `dokime proto export`
  provides the definitions. There is no dynamic call layer.
- A plugin reads its settings through a host operation when it needs them;
  nothing is injected at start.
- Dokime's catalog sections are typed `dokime.v1` messages.
- Harmos stays generic. Every Dokime concept is an operation convention or a
  catalog section.

## Impact

- Breaking for every plugin and for data roots with executions in flight; the
  development storage policy allows the clean break with a fresh root.
- Depends on the Harmos change `grpc-plugins`.
- Supersedes `remote-plugins` (deleted with this change) and the 2026-11-07
  decision to build native SDKs over a C ABI.
- Absorbs `template-authoring-simplification` (deleted with this change):
  Rust steps take one context type with no template parameters, and the
  plugin's own operation registry is its requirement inventory.
- Absorbs `dokime-sdk` and `dokime-sdk-cpp` (deleted with this change): the
  runtime-free Rust SDK, the single `dokime` tool and the starters.
  `dokime-sdk-go` is removed; Go follows the pilot.
- `runtime-stream-cadence` is landed and stays; its remaining dependency pin
  closes when this change adopts the Harmos revision. `telemetry-at-record`
  is unaffected.
- Private plugins migrate in their own repositories.
