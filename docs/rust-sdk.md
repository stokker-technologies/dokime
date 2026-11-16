# Rust SDK

The `dokime` package provides typed clients and plugin authoring over the pinned
Harmos SDK. Its product dependencies contain the protocol and SDK packages;
execution, storage and native runtime dependencies belong to the host.

## Public Imports

Shared values and data derives live at `dokime`. Client operations and their
options live at `dokime::client`; plugin authoring and host-call options live at
`dokime::plugin`. Generated transport modules remain private.

```rust
use dokime::{Message, Result, Visibility};
use dokime::plugin::{Context, Operation, Plugin};

#[derive(Message)]
struct Ping {
    value: String,
}

fn main() -> Result<()> {
    let ping = Operation::new(
        "example.ping",
        Visibility::Public,
        async |_: &Context, request: Ping| Ok(request),
    );
    Plugin::new("example", "0.1.0").operation(ping).serve()
}
```

Private operations stay inside their plugin, Protected operations admit host
calls, and Public operations can be discovered and bound by clients. Requirements
describe dependencies; they do not grant access. Suite lifecycle adapters retain
independent Protected operation identities and authored policies.

## Examples and New Projects

The runnable projects are [client](../sdk/rust/examples/client),
[plugin](../sdk/rust/examples/plugin) and [test](../sdk/rust/examples/test).
The scaffold embeds these examples; there is no separate starter source tree.
The test example defines a suite and template, asks an operator question, records
its answer and evaluates the retained record.

The [thermal suite](../sdk/rust/examples/thermal.rs) and
[execution fixture](../sdk/rust/examples/execution.rs) retain the shared host
journeys. Both use public imports, execution-local resources and host-driven time.
The thermal suite captures temperature samples and exact integer counters,
acknowledges door records and evaluates typed statistics for the Hold step.
Scale and exactness documents live in the [OBC plugin](../plugins/sidecars/obc/) Benchmark suite, with separate stream, plot, observer and exact-value workloads.

## Clients

Construct `Client::from_channel(channel, settings)` with explicit client limits.
The composed tree exposes library operations beside product operations:

```rust
let entries = client.plugins.list().await?;
let suites = client.suites.list("thermal").await?;
let now = client.time.get().await?;
```

Calls take request content as arguments and own their transport context. Methods
ending in `_with` accept options containing a `CallOptions` value for metadata and
a deadline override. The operation owns its normal deadline and retry policy.
A response with exactly one field returns that payload; replies with receipts,
read cuts or other fields retain their complete typed response.

```rust
use dokime::client::{CallOptions, PluginsListOptions};

let mut context = CallOptions::default();
context.metadata.insert("authorization", token);
let entries = client.plugins.list_with(PluginsListOptions {
    context,
    ..Default::default()
}).await?;
```

`client.operations.list()` supplies admitted discovery contracts. `Method` binds
a contract to typed messages and checks visibility, names, descriptors and schema
fingerprints. Dynamic calls use `call` or `call_with`; managed calls use `start`,
retain explicit input completion, cancellation and joining, and preserve the
original mutation identity when retrying.

Product watches return managed `Watching<T>` values. Consume `next().await?` and
drop the watch to release observation without cancelling the execution. Retain
issued read cuts and continuation tokens unchanged while traversing a selection.
Dropping the client releases its channels after its watches have been dropped.

`client.files.upload(path).await?` uses the returned upload handle and transfer
limits, retains each mutation identity and verifies accepted offsets, length and
SHA-256 before returning the published file reference. Failure abandons the owned
upload and retains the cleanup failure's cause when abandonment fails.

## Plugins and Execution Resources

Compose `Plugin`, `Suite`, `Template`, `Step`, `Criterion`, `Record`, `Stream` and
`Observer` values from `dokime::plugin`. Data derives remain at the root.
`Context::set/get` retains typed execution resources through initialization,
steps, observers and criteria. Cleanup empties the execution store even when the
cleanup operation fails or is cancelled. Materialization cannot allocate execution
resources, change resolved parameters or introduce undeclared operations.

Declare host requirements with the generated `HostOperation` enum, before adding
record descriptor types. Use `ctx.host()` for the generated plugin-admitted tree:

```rust
use dokime::plugin::HostOperation;

let plugin = Plugin::new("example", "0.1.0")
    .requires_host(HostOperation::TimeGet)
    .requires_host(HostOperation::PluginsSettingsList);

let now = ctx.host().time.get().await?;
let settings = ctx.host().plugins.settings.list().await?;
```

`ctx.sleep` and `ctx.now` compose library Time operations. Controlled time advances
through host waits; no SDK retry loop or wall-clock wait drives execution. A scoped
sleep settles after every observer window due by its end has been answered and
stored, so a step that sleeps for `n` intervals finds `n` more samples saved.
Observer intervals and retry timing belong to declared settings and operation
policies.

## Observers and Interval Windows

The host keeps each interval stream's schedule and grants it one window at a time:
`count` samples, the first due at `start`, each `period` later, with `sequence`
numbering the first. The plugin answers the whole window with columns, and the host
stores them without per-sample work. The window length is the host setting
`interval_window_ms` (`PlacementSettings::interval_window`); `.window_setting("id")`
takes it from a plugin Duration setting for one stream instead. The batch size
follows from the window and the interval and is never set directly.

```rust
use std::time::Duration;

// One sample per call: the SDK calls the handler once per sample, in order.
let temperature = Observer::new(&stream, read_temperature).every(Duration::from_secs(1));

// One call per window: the columns message repeats every row field (same numbers and types).
#[derive(dokime::Message)]
#[harmos(package = "thermal")]
struct CounterColumns {
    value: Vec<u64>,
}
let counter = Observer::window(&counter_stream, async |_: &Context, window: &SampleWindow| {
    Ok(CounterColumns { value: vec![1; window.count as usize] })
})
.every(Duration::from_millis(1))
.window_setting("counter_window");
```

Use the window form for high-rate streams and for models that compute every sample
from its due time (`window.start + i * window.period`); keep the one-sample form when
a sample depends on state that steps change between samples. Interval rows hold one
value per field: repeated, map, oneof and `Option` fields are refused when the plugin
is admitted. Repeated numeric fields derived with `dokime::Message` encode packed.

Plugin settings use `SettingDeclaration`, `SettingValue` and
`SettingsDeclarations` from the library. `.settings(defaults)` derives Boolean,
signed integer, number and text declarations from typed data; `typed_settings`
decodes the corresponding complete value map. Product secrets use `.secret` with
a `SecretDeclaration`, separate from ordinary settings. A plugin reads declared
bindings through `ctx.host().secrets.get(name)`; returned bytes are in `secret`,
and administrative metadata remains client-only.

## Test Templates Without a Host

`dokime::testing::Harness` runs one template of a composed plugin in-process, with
no host process, storage or hardware. It resolves declared parameter defaults,
applies the materializer, then runs initialization, each frozen step, the
criteria and cleanup in the host's order against an in-memory host with a
controlled clock starting at the epoch (`starting_at` moves it). `ctx.sleep`
advances that clock and, before it returns, grants each `every` observer one window
of every sample due by its end, as the host does for a scoped sleep; replay-safe steps retry within their policy. Declared host requirements
are enforced. Prompts take the answer given with `answer`/`answer_with`,
otherwise the declared prompt's first non-cancel action.

```rust
use dokime::testing::Harness;

#[tokio::test]
async fn soak_holds_the_setpoint() -> dokime::Result<()> {
    let report = Harness::new(plugin(), "oven.soak")
        .parameters(Setpoint { celsius: 85.0, hold_s: 2 })
        .answer("oven.door", "closed")
        .run()
        .await?;
    assert!(report.passed());
    assert_eq!(report.samples::<Temperature>("oven.temperature")?.len(), 4);
    assert_eq!(report.records::<Soaked>("oven.soaked")?[0].celsius, 85.0);
    Ok(())
}
```

`Report` exposes the termination and failure, evaluated criteria, typed records
and samples with their step and virtual time, logs, answered prompts and the
latest progress value. The in-memory host simulates Time, Backend.Environment,
evidence records, logs and statistics (count, extremes, mean, sum, first and
last of one numeric field), Prompts.Create and executions.Progress. Other host
operations, peer operations, watches, `publish_on` producers and observers whose
interval comes from a setting refuse with `testing_host_unsupported` or an
action to adjust the test; prove those against a real host.
[`testing.rs`](../sdk/rust/tests/integration/testing.rs) is the author story.

## Frozen Evidence and Exact Values

`FrozenSchemaCache` is caller-owned. `get` fetches descriptor pages at one issued
cut, verifies frozen bindings and the complete digest, and reuses equal digests
across executions. `decode` resolves captured record and stream identities against
those frozen descriptors. Dropping the cache releases its retained sets.

Statistics and plot summaries carry typed `Value` fields, including exact totals
and sums. `ExactNumber` retains sign, decimal numerator and denominator without
floating conversion. `FromValue` extracts it losslessly; floating extraction
refuses an exact value. Drawing coordinates, standard deviation and RMS remain
floating point. Document replies keep pending bindings, result summaries and
card completion states.

## Focused Proof

```sh
mise -C sdk/rust run format
mise -C sdk/rust run build
mise -C sdk/rust run test -- --layer unit
mise -C sdk/rust run test
mise -C sdk/rust run lint
mise -C sdk/rust run check
```

The external consumer owns installed-consumer compilation, fixture binaries and
integration stories, with a separate lock and the repository's shared target.
The build includes the client, plugin and test example sources as consumer binaries.
Run `prepare` only when refreshing that consumer's dependency lock.
The unit selection covers both SDK crates and reports an empty selection when
their invariants are covered by the integration journeys instead.
Its stories exercise typed declarations, authentication, discovery, schema
refusals, response bounds, managed watches and isolated execution resource
release. Protocol conformance drivers use canonical internal messages only inside
their test owner. They never compile plugins during a story or use fixed waits.

Host-backed execution, durability, secret credential cleanup and cross-language
journeys belong to the runtime system stories listed in
[Execution Host](execution-host.md). Native Windows and Linux proof are separate.
This SDK currently declares no WebAssembly feature or product target.
