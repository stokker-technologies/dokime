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
describe dependencies; they do not grant access. Suite lifecycle and analysis
adapters retain independent Protected operation identities and authored policies.

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
Scale and exactness documents live in the [OBC plugin](plugin-obc.md) Bench suite.

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
through host waits and observer ticks; no SDK retry loop or wall-clock wait drives
execution. Observer intervals and retry timing belong to declared settings and
operation policies. `Operation::analysis` adapts typed analysis requests and
responses while retaining raw sample axes and outputs.

Plugin settings use `SettingDeclaration`, `SettingValue` and
`SettingsDeclarations` from the library. `.settings(defaults)` derives Boolean,
signed integer, number and text declarations from typed data; `typed_settings`
decodes the corresponding complete value map. Product secrets use `.secret` with
a `SecretDeclaration`, separate from ordinary settings. A plugin reads declared
bindings through `ctx.host().secrets.get(name)`; returned bytes are in `secret`,
and administrative metadata remains client-only.

## Frozen Evidence and Exact Values

`FrozenSchemaCache` is caller-owned. `get` fetches descriptor pages at one issued
cut, verifies frozen bindings and the complete digest, and reuses equal digests
across executions. `decode` resolves captured record and stream identities against
those frozen descriptors. Dropping the cache releases its retained sets.

Statistics and plot summaries carry typed `Value` fields, including exact totals
and sums. `ExactNumber` retains sign, decimal numerator and denominator without
floating conversion. `FromValue` extracts it losslessly; floating extraction
refuses an exact value. Drawing coordinates, standard deviation and RMS remain
floating point. Analysis helpers preserve source-time axes; document replies keep
pending bindings, result summaries and card completion states.

## Focused Proof

```sh
mise -C sdk/rust run format
mise -C sdk/rust run sdk:build
mise -C sdk/rust run sdk:unit
mise -C sdk/rust run sdk:test
mise -C sdk/rust run sdk:lint
mise -C sdk/rust run sdk:tree
```

The external consumer owns installed-consumer compilation, fixture binaries and
integration stories, with a separate lock and the repository's shared target.
The build includes the client, plugin and test example sources as consumer binaries.
Run `sdk:prepare` only when refreshing that consumer's dependency lock.
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
