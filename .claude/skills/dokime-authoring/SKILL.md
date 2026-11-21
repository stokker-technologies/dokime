---
name: dokime-authoring
description: Author Dokime plugins, suites, steps, criteria, evidence and documents.
---

# Dokime Authoring

Read [Architecture](../../../docs/project/architecture.md) for current invariants and the
locked operation grammar. Author plugins with the product SDKs; Rust product
SDK dependencies stop at protocol. The host owns execution sequencing and cleanup.

- [Plugins](references/plugins.md): SDKs, catalog operations and bundle admission.
- [Suites](references/suites.md): materialization, lifecycle and evidence ownership.

The [Rust SDK](../../../docs/sdks/rust.md), [Python SDK](../../../sdk/python/README.md),
[C# SDK](../../../docs/sdks/csharp.md) and [C++ SDK](../../../docs/sdks/cpp.md) own
language examples. Generate projects with `dokime new <path> --lang <language>`.
Use explicit invocation scope and operation visibility, pure materialization,
execution-local resources and required cleanup. Query/document builders retain
protocol values; supported variants are documented in the execution-host guide.

## Rust

Use `plugins/sidecars/obc/src/` as the reference layout for all plugins. Keep the
root limited to `main.rs`, `lib.rs`, `settings.rs` and an owned `error.rs` only when
needed. `lib.rs` composes inert values from each global module's `catalog()`.
Use today's `initialize`, steps, criteria and `cleanup` lifecycle; materialization
never creates a bench or opens hardware.

```text
src/
  main.rs
  lib.rs
  settings.rs
  device/                 Bench, Charger, Supply, Reading, Power, Fault
  operations/             mod.rs registers every operation; one file per family
  streams/measurement.rs  Shared row, stream and sampler
  records/charger.rs      Events shared by several templates
  prompts/inspection.rs   Form shared by several templates
  suites/
    mod.rs                Suite modules and shared execution glue
    steady_state/
      mod.rs              Suite value listing its templates
      single_phase.rs     Complete template slice
      three_phase.rs
    transient/
      mod.rs
      startup.rs
      overcurrent.rs
      overvoltage.rs
    benchmark/
      mod.rs
      streams.rs
      plots.rs
      observers.rs
      exact_values.rs
```

A template file owns its identifier, `template()`, parameters, steps, criteria,
document and any prompts, records, streams, observers or samplers unique to it.
A suite `mod.rs` defines the Suite and lists templates; `suites/mod.rs` contains
module declarations and shared supply, timing, context, wait and capture glue.
Global folders contain only genuinely shared declarations or plugin-lived items.
An `observers/` folder is only for plugin-level observers. Do not lift a unique
thermal survey into a global catalog merely because it is an observer.
Every present global module exposes `catalog()`, including settings and suites.
Register unique template contracts explicitly from `lib.rs`; keep their definitions
in the template file.

Device ownership follows initialization, not the folder name. Create a fresh
`Bench` in template initialization and store it in execution context. Keep its
state, model clock and locks private behind top-level device interfaces; cleanup
powers down and the execution's resource store releases the instance. A manual
charger used by public operations may be plugin-lived and uses plugin stop cleanup.

Compose builder values by value, with no authoring macros or plugin-local catalog
shorthands. The SDK's protocol-backed values preserve typed lists, Cartesian
sweeps and exact unsigned integers:

```rust
let voltage = Parameter::number("ac_voltage_v", "AC RMS Voltage")
    .unit("V").default(230.0).range(0.0..=300.0);
let samples = Parameter::count("samples", "Samples").default(100_u64);
let loads = Parameter::sweep("load_ohm", "Load Resistance", [80.0, 100.0]);
let ladder = Parameter::numbers("steps", "Steps", [100.0, 50.0]).items(1..=16);
let plot = Plot::line("voltage", "AC Voltage").unit("V");
let card = Card::plot(plot).columns(6);
let records = Card::records("records");
```

Pass a parameter builder directly to `Template::parameter`, or use `.build()` for
an array of canonical parameter messages. `Parameter::flag` declares a boolean.
IDs use `<plugin>.<suite>.<template>`; OBC deliberately has no aliases for retired
suites. Single- and three-phase templates take resistance and reactance, test
conversion efficiency and power factor, and share the Measurement sampler.
Balanced three-phase readings expose per-phase RMS values and total input/DC power.
Benchmark templates isolate streams, plots, heavy observers and exact values.

## Python

Use ordinary values from `dokime_sdk.authoring`, `dokime_sdk.parameters`, `dokime_sdk.plot`
and `dokime_sdk.documents`. Keep execution resources on the template instance and
close them explicitly in cleanup. Keep unique evidence definitions with the
owning template and share only definitions used by several templates.

```python
from dokime_sdk.parameters import Parameter
from dokime_sdk.plot import Plot
from dokime_sdk.documents import plot_card, records_card

voltage = Parameter.number("ac_voltage_v", "AC RMS Voltage").unit("V").default(230.0).range(0.0, 300.0).build()
loads = Parameter.sweep("load_ohm", "Load Resistance", [80.0, 100.0]).build()
plot = Plot.line("voltage", "AC Voltage").unit("V")
card = plot_card(plot, columns=6)
records = records_card("records")
```

`count`, `flag` and `numbers` have the same roles as in Rust. Pass built messages
in `Template.parameters`; a list is one case's ordered input, while a sweep expands
cases. Do not redirect Python environments or require context managers.

## C#

Use `Dokime.Plugins.Parameters`, `Dokime.Plugins.Plots` and `Cards`. Qualify names
when they collide with generated service names. Builders return canonical messages
accepted by templates; no generated protocol source is hand-edited.

```csharp
var voltage = Parameters.Number("ac_voltage_v", "AC RMS Voltage")
    .WithUnit("V").WithDefault(230.0).Range(0.0, 300.0);
var loads = Parameters.Sweep("load_ohm", "Load Resistance", new[] { 80.0, 100.0 });
var plot = Dokime.Plugins.Plots.Line("voltage", "AC Voltage").WithUnit("V");
var card = Cards.Plot(plot).Columns(6);
var records = Cards.Records("records");
```

`Count`, `Flag`, `Numbers` and `Items` preserve their Rust meanings. Use `ulong`
for exact counters. Execution resources belong to template initialization/cleanup;
plugin operations own only plugin-lived resources.

## C++

Include `dokime/dokime.hpp`; compose values in `dokime::authoring`. Pass built
parameter messages to the template definition and plot/card values to documents.
Keep execution benches behind their device interface and explicit cleanup.

```cpp
auto voltage = dokime::authoring::Parameter::number("ac_voltage_v", "AC RMS Voltage")
    .unit("V").default_value(230.0).range(0.0, 300.0).build();
auto loads = dokime::authoring::Parameter::sweep("load_ohm", "Load Resistance", {80.0, 100.0}).build();
auto plot = dokime::authoring::Plot::line("voltage", "AC Voltage").unit("V");
auto card = dokime::authoring::plot_card(plot, 6);
auto records = dokime::authoring::records_card("records");
```

`count`, `flag`, `numbers` and `items` mirror the other SDKs. Use `std::uint64_t`
for exact counters; avoid namespace/type renaming aliases.
