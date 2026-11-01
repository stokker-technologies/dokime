---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

- [Plugins](references/plugins.md): runtime ownership, hosting and contribution layout.
- [Suites](references/suites.md): declarations, lifecycle capabilities and shared handles.

Depend on `dokime` with feature `plugin-sidecar` for Dokime authoring. Import product data through `dokime::domain` and operation declarations through `dokime::operations`; declare each plugin-owned contract once in its `domain/` owner. Host stream rows use `plugin_host::{System, Network, WifiLink, WifiScan, WifiLatency, Bluetooth}` from a definitions-only git-tag dependency. Host document defaults and the occupancy plot binding live in `plugin_host::domain::document`. Install the host bundle before registering templates that subscribe to its streams; missing publishers receive ordinary per-stream refusals.

Host `System::memory_used_gib()` and `System::memory_total_gib()` expose Memory Used [GiB] and Memory Total [GiB] with structured `GiB` units (1024³ bytes). They share the byte snapshot used for `System::memory_pressure_percent()` on Linux and Windows; pressure is 100 × used / total and absent when total is zero. Keep used-memory plots beside pressure in system sections.

Steps contribute and edit traceability through `ctx.traceability()` using the document-style evidence pipeline; no traceability lifecycle hook exists. Criteria and documents read captured evidence only. Preserve collected evidence and the distinct `Invalidated` terminal outcome; a failed report publication never changes the execution verdict.

Plot definitions declare their data and presentation, never a time window. The
live viewer selects one window for all plots in the view, defaulting to All.
Reports always include the full captured sample range with bounded previews.

Secret declarations read shared Secret Storage identifiers directly: a field
`api_token: dokime::plugin::Secret` requires the identifier `api_token`;
`Option<Secret>` permits absence. No secret defaults, `from_environment`, saved
references or per-plugin secret writes exist. Add missing identifiers in Settings
→ Secret Storage, then restart Dokime or use the existing plugin Retry. Plugin
settings show read-only declaration availability and a prefilled Add Secret action.
