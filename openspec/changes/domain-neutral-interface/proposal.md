# Domain-Neutral Interface

Status: agreed with the owner on 2026-11-10; implementation waits for the
owner's go. Builds on `grpc-plugins` and `plot-pipeline`.

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

- **A generic grid plot replaces the occupancy plot.** It draws discrete
  cells in groups, one frame at a time, with a frame slider and "follow
  latest". The existing heatmap interpolates a surface and cannot do this.

  | The Plugin Declares | The Product Draws |
  | --- | --- |
  | Cells: key, label, order and group | The grid, cell colours and values |
  | The frame column and the value column | The frame slider |
  | How a frame reduces rows (for example the maximum per cell per second) | Tooltips and per-group summaries |
  | The highlighted cell and the wording | The highlight and labels |

  ```python
  dokime.plot.grid(
      stream="host.wifi.scan",
      frame="timestamp",
      cell="wifi_channel",
      value="network_count",
      cells=CHANNELS,                  # key, label, group ("2.4 GHz", "5 GHz", "6 GHz")
      reduce="max",
      highlight=dokime.field("active_channel"),
  )
  ```

- **The host plugin owns the Wi-Fi knowledge:** channel lists, bands, the
  6 GHz encoding, the active channel and the wording. The Wi-Fi analysis and
  its disk-backed variant in the runtime are deleted; the grid's reduction is
  generic.
- **The Wi-Fi channel selector becomes a generic grouped choice.** A
  parameter declares its options in groups; the options come from the
  plugin's option provider.
- **Traceability shows what plugins declare.** The fixed host-system fields,
  categories and titles in the interface and runtime are replaced by fields
  and groups the plugin contributes.
- **Units and labels come from declared field metadata.** Inference from
  field names and the hardcoded stream identifiers in plot grouping are
  removed.
- **Unsupported plot kinds are deleted:** grouped bar, stacked bar, pie,
  donut, Pareto and aggregate table exist as values and are rejected
  everywhere.

## Impact

- Interface plot catalog, document types and adapters; domain and wire plot
  definitions; runtime plot analyses, previews and traceability; the host
  telemetry plugin; tests and the authoring guidance.
- Reports that captured an occupancy plot keep rendering what they captured
  only if the bundled report keeps the old kind; the development storage
  policy allows dropping it with a fresh root.
