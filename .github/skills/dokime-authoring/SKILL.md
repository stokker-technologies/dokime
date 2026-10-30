<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/SKILL.md; sha256:58697e9dba8612a67a0abdda3786fbd2ca848c9a0c68ae406098757dc6f6fb45 -->

---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

- [Plugins](references/plugins.md): runtime ownership, hosting and contribution layout.
- [Suites](references/suites.md): declarations, lifecycle capabilities and shared handles.

Depend on `dokime` with feature `plugin-sidecar` for Dokime authoring. Import product data through `dokime::domain` and operation declarations through `dokime::operations`; declare each plugin-owned contract once in its `domain/` owner. Host stream rows use `dokime_host::{System, Network, WifiLink, WifiScan, WifiLatency, Bluetooth}` from a definitions-only git-tag dependency. Host document defaults and the occupancy plot binding live in `dokime_host::domain::document`. Install the host bundle before registering templates that subscribe to its streams; missing publishers receive ordinary per-stream refusals.

Criteria and documents read captured evidence only. Preserve collected evidence and the distinct `Invalidated` terminal outcome; a failed report publication never changes the execution verdict.
