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

Plot `unit` is only a label. `plot.scale(1e3)` converts returned time-series y
values, statistics (except count), cursor/tooltips and y references for display
in live plots and frozen HTML. It defaults to 1.0 and is omitted when absent.
Validation requires a positive finite constant and refuses other plot kinds,
custom analyses and alternate x axes, naming the plot. Filters and section repeats
keep stored units; evidence, exports, criteria, query plans and expressions stay
unchanged. Arithmetic expressions such as `column * 1e3` remain unsupported.


Reference bands and lines belong to the plot and always use its Y axis. Bounds,
centers, tolerances and line values accept `f64` or an existing expression handle
(or `DataExpression`); constants do not become data expressions, so
`plot.y(10.0)` is still invalid. All reference values use stored units and scale
with the data when `plot.scale(...)` is set:

```rust
plot.reference_band("Requirement", -10.0, 10.0);
plot.reference_band("Limits", Parameters::lower_limit_c(), Parameters::upper_limit_c());
plot.reference_tolerance("Requirement", 0.0, Parameters::angle_tolerance_arcsec());
plot.reference_tolerance("Target ± Band", Parameters::target_c(), Parameters::band_c());
plot.reference_line("Limit", Parameters::limit_c());
```

Each case resolves its captured parameter values through the same presentation
path in live plots and frozen HTML. Missing or invalid dynamic bounds are undrawn.
Document validation refuses reversed literal bounds and negative literal tolerance,
naming the plot. The development document/storage format stays at v1; existing
incompatible documents require an explicit fresh storage root, preserving old data.

Secret declarations read shared Secret Storage identifiers directly: a field
`api_token: dokime::plugin::Secret` requires the identifier `api_token`;
`Option<Secret>` permits absence. No secret defaults, `from_environment`, saved
references or per-plugin secret writes exist. Add missing identifiers in Settings
→ Secret Storage, then restart Dokime or use the existing plugin Retry. Plugin
settings show read-only declaration availability and a prefilled Add Secret action.
