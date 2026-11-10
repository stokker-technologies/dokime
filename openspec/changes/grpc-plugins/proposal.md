# gRPC Plugins

Status: shape agreed with the owner on 2026-11-10; implementation waits for the
owner's lock. Replaces `remote-plugins`. The Harmos change `grpc-plugins` is
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
- A `TestCase` names its steps, criteria and streams. Steps, criteria,
  `initialize` and `cleanup` are plugin operations the host calls; the host
  owns ordering, retries, cancellation, cleanup and verdicts.
- Everything a step or criterion needs from Dokime is a host operation called
  through `ctx`: prompts, secrets, settings, records, statistics, files.
- Samples arrive on streams the host opens. Records are acknowledged host
  operations.
- One Dokime protocol package, `dokime.v1`, holds every operation Dokime
  serves and the messages for the operations plugins serve.
- Each `sdk/<language>` has two halves: provide plugins, and drive Dokime.
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
- A plugin restart during an execution seals the execution and its cycles.
  There is no resume. A dropped connection to the same process fails only the
  step in flight, which is retried when the step allows it.
- Evidence is ordered by arrival at the host.
- A remote plugin is trusted by whoever adds its URL. Bundles keep the
  existing install path and security roadmap.
- The host telemetry plugin stays a managed plugin.
- No throughput gate: design for simplicity, tune afterwards.
- Harmos stays generic. Every Dokime concept is an operation convention or a
  catalog section.

## Impact

- Breaking for every plugin and for data roots with executions in flight; the
  development storage policy allows the clean break with a fresh root.
- Depends on the Harmos change `grpc-plugins`.
- Supersedes `remote-plugins` (deleted with this change) and the 2026-11-07
  decision to build native SDKs over a C ABI.
- `dokime-sdk`, `dokime-sdk-cpp`, `dokime-sdk-go`,
  `template-authoring-simplification`, `runtime-stream-cadence` and
  `telemetry-at-record` overlap and are reconciled before implementation
  starts (`design.md`, Open Points).
- Private plugins migrate in their own repositories.
