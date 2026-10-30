---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

- [Plugins](references/plugins.md): runtime ownership, hosting and contribution layout.
- [Suites](references/suites.md): declarations, lifecycle capabilities and shared handles.

Depend on `dokime` with feature `plugin-sidecar` for Dokime authoring. Import product contracts through `dokime::domain`; declare each plugin-owned contract once in its `domain/` owner. Host stream rows use `dokime_host::{System, Network, WifiLink, WifiScan, WifiLatency, Bluetooth}` from a definitions-only git-tag dependency. Host document defaults and the occupancy plot binding live in `dokime_host::domain::document`. Install the host bundle before registering templates that subscribe to its streams; missing publishers receive ordinary per-stream refusals.

Steps contribute and edit traceability through `ctx.traceability()` using the document-style evidence pipeline; no traceability lifecycle hook exists. Criteria and documents read captured evidence only. Preserve collected evidence and the distinct `Invalidated` terminal outcome; a failed report publication never changes the execution verdict.
