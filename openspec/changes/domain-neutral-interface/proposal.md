# Domain-Neutral Interface

Status: agreed with the owner on 2026-11-10; implementation waits for the
owner's go. Builds on `grpc-plugins`, `plot-pipeline` and `plot-kinds`.

## Why

The product contains knowledge that belongs to one plugin. The Wi-Fi channel
occupancy plot is a hardcoded component with matching special cases in the
runtime, about 1,430 lines in all. The interface also names that plugin's
streams and fields, carries a dedicated Wi-Fi channel selector, fixes the
host-system fields shown in traceability, and guesses units from field names.
The rule is that the interface knows kinds of plot and kinds of input and
nothing about a domain. Plugins do not ship interface code: reports render
offline, plugins are written in several languages, and a remote plugin is
only a URL.

## What Changes

- **Plot domain neutrality is owned by `plot-kinds`.** That change owns
  GRID_CHART, occupancy document migration and special-case removal, shared
  playback, supported-kind cleanup and live/report acceptance. This change
  consumes that contract without specifying a second grid or kind catalog.
- **The Wi-Fi channel selector becomes a generic grouped choice.** A
  parameter declares its options in groups; the options come from the
  plugin's option provider.
- **Traceability shows what plugins declare.** The fixed host-system fields,
  categories and titles in the interface and runtime are replaced by fields
  and groups the plugin contributes.
- **Units and labels come from declared field metadata.** Inference from
  field names and the hardcoded stream identifiers in plot grouping are
  removed.

## Impact

- Generic parameter controls, traceability contributions, field metadata and
  plot-group labels in the interface/runtime; plugin declarations, tests and
  authoring guidance. Plot kinds, grid frames and report playback belong to
  `plot-kinds`; its proposed catalog replaces the earlier unsupported-kind
  deletion plan and awaits owner confirmation.
