# Dokime Rust SDK

Author executable test plugins and call a running host with the `dokime` crate.
The [Capture Suite](examples/starter/src/plugin/suites/capture/confirmation.rs) owns its records, steps and criteria.

Plugin lifecycle hooks use ordinary async operation contexts:

```rust
Plugin::of::<ExamplePlugin>("0.1.0")
    .initialize(async |ctx: &dokime::operations::Context| Ok(()))
    .finalize(async |ctx: &dokime::operations::Context| Ok(()));
```

The SDK encodes protected `<plugin>.initialize` and `<plugin>.finalize` operations
with empty payloads. Hooks share plugin-owned state with other operations.

Operation and template contexts accept the library's typed requests and untyped
JSON calls through the same method:

```rust
use dokime::untyped;
use serde_json::json;

let reply = ctx.operation(untyped("example.measure", json!({"channel": 1}))).await?;
```

The host converts protobuf JSON, validates every field, and returns its structured
failure unchanged. An in-process testing host does not simulate operation calls;
exercise these calls against a real host.

`Plugin::run()` and `Plugin::run_with(address)` register with the host selected by `DOKIME_REMOTE` in the process
environment or working-directory `.env`. Managed launches take precedence.
Registration sends the SDK version and uses the plugin identifier as its stable
entry name. Closing the client stops its listener and retains the entry;
`RegisteredPlugin::unregister()` removes it explicitly.

## Operation Lanes

`dokime::Lane` re-exports the library declaration. Create `let instrument = Lane::serial("instrument");`, assign it with `operation.lane(&instrument)`, and register it with `plugin.lane(instrument)`. `Lane::concurrent("workers", 4)` bounds concurrency; `Lane::unbounded("independent")` is unbounded. Declaration validation rejects undeclared lane references.

## Author a Suite

```rust
use dokime::{Plugin, Result, Suite};
use dokime::templates::{Context, Criterion, Record, Step, Template};

struct CaptureSuite;
struct ExamplePlugin;
struct ConfirmationTestTemplate;
impl ConfirmationTestTemplate {
    fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }
    fn template() -> Template {
        Template::of::<Self>()
            .record(Self::confirmation())
            .step(Step::new("Confirm", Self::confirm))
            .criterion(Criterion::new("Confirmed", Self::confirmed))
    }
    async fn confirm(ctx: &Context) -> Result<()> {
        let answer = ctx.prompt("Continue?", &["Yes", "No"]).await?;
        ctx.record(&Self::confirmation(), answer == "Yes").await?;
        Ok(())
    }
    async fn confirmed(ctx: &Context) -> Result<bool> {
        ctx.records(&Self::confirmation()).step(Self::confirm).last().await
    }
}
let plugin = Plugin::of::<ExamplePlugin>("0.1.0").suite(Suite::of::<CaptureSuite>().template(ConfirmationTestTemplate::template()));
```

This declares template `capture.confirmation`, step `capture.confirmation.confirm`,
criterion `capture.confirmation.confirmed` and record `capture.confirmation.confirmation`.
`Plugin::of::<T>`, `Suite::of::<T>()` and `Template::of::<T>()` derive identifiers and
labels from marker names, removing `Plugin`, `Suite` and `TestTemplate` suffixes.
`.identifier(...)` and `.label(...)` override them explicitly. Steps and criteria
use their function names; anonymous handlers require an explicit identifier.
Builders compose ordinary values and data alone uses derives.

Operation handlers receive `dokime::operations::Context`; template handlers receive
`dokime::templates::Context`. Both read plugin-owned state through `plugin::<T>()`.
Template state uses `set(value)` and `get::<T>()` and lasts for one execution.
Authors do not declare host requirements.
Template observers use `Observer::new` or `Observer::window` with the template
context. Plugin-wide observers use `Observer::operation` or
`Observer::operation_window` with the operation context; they read plugin-owned
state and have no execution-only methods.

## Collections and Durable Plugin State

`Plugin::collection` registers the library `Collection` directly. Declare native
member streams, publish through `CollectionOutput`, select an `interval_setting`,
and acquire/release resources through `on_start`/`on_stop`. The host owns demand,
windows and timestamps. Bind a collection invocation to shared author state with
`operations::Context::from_collection`; never retain an earlier invocation.
Collection sampling requires a product host; the in-process template harness
refuses it rather than running another scheduler.

Operation contexts expose `state()?`: call `get::<T>().await?` before
`put(&value).await?` on the same handle. Put compares the observed revision;
conflicts require a fresh read and reapplication within the operation's policy.

## Values and Operations

Records and simple operations accept `bool`, `String`, `i64` and `f64`; the SDK
encodes their well-known protobuf wrappers. Structured rows use `#[derive(Message)]`
or `#[derive(dokime::Record)]` with `#[dokime(package = "example")]` when needed.

```rust
use dokime::{Plugin, Result, Visibility};
use dokime::operations::{Context, Operation};
struct ExamplePlugin;
let echo = Operation::new("example.echo", Visibility::Public,
    async |_: &Context, request: String| Ok(request));
let plugin = Plugin::of::<ExamplePlugin>("0.1.0").operation(echo);
```

Declare structured questions with `dokime::templates::Prompt` on the template and
pass the declaration to `ctx.prompt(&question, &[])`. The short text call uses `<template>.question`. The call returns the chosen
label and refuses rejected or unknown actions. `records(&declaration).last()`
returns a typed value and refuses an empty result. `.step(handler)` or
`.step(&step_declaration)` selects one declared step.

## Run the Examples

The [Starter](examples/starter/) is one Cargo package with a plugin binary and a
[Client Flow](examples/starter/src/bin/client.rs). The client connects through
`DOKIME_REMOTE`, registers its owned listener, materializes the confirmation case,
runs it, seals the execution and cycle, saves a timestamped archive in `_output/`
and closes the registration and listener.

```powershell
mise -C sdk/rust run build
mise -C sdk/rust run test -- --layer unit
mise -C sdk/rust run test
# Create a standalone consumer of this checkout:
./target/debug/dokime new my-project --lang rust --from .
cd my-project
mise trust
mise run build
# With DOKIME_REMOTE naming a running host:
mise run run
```

`--from` (or `DOKIME_NEW_FROM`) selects the checkout's starter and writes a Cargo
path dependency on its Rust SDK. The explicit argument wins over the environment.
Without either, the command uses its embedded starter and published dependency.
The plugin binary calls `plugin()?.run()` without parsing arguments. Launch credentials
select library announcement; otherwise `DOKIME_REMOTE` selects self-registration,
and no remote selects standalone serving.
`register_with` accepts a `PluginRegisterOptions` address override; otherwise
`127.0.0.1:0` selects its owned listener. `client.close()` retains the host entries
and joins their listeners; repeated close is safe.

`dokime::testing::Harness` runs a template against an in-memory host
with a controlled clock, without hardware.
Cleanup runs after attempted initialization, including failure and cancellation,
and can inspect resources created before initialization failed.

## Read Bound Resources

`client.plugins().list()`, `client.cases().list()`, `client.cycles().list()` and
`client.executions().list()` return handles owned by that connection.
`CyclesQuery` and `ExecutionsQuery` select filtered lists through `list_with`.
Paged cycle and execution reads preserve the first durable cut and refuse changed
cuts or repeated continuation tokens. Case, cycle and execution handles support
`refresh()` without creating another resource.

`execution.records::<T>(identifier)` supports `all()`, `rows()`, `count()` and
`last()`. `execution.samples::<T>(identifier)` supports `all()` and `rows()`.
Rows preserve their acknowledged identity, ordinal and exact timestamp;
sample rows also retain receipt time, stream and source.
Readers verify the frozen descriptor digest and message binding before decoding
payloads, and reject empty `last()` selections.

Cycle/execution creation, execution controls, cycle sealing, notes and artifacts
offer `_with` methods accepting operation-specific request identity options.
Reuse a request identity with identical inputs when replaying a mutation.
Execution creation derives the same execution UUID from that identity.
Template contexts also support `artifact()` and `artifact_with()`; they share the
client's fenced upload implementation and attach only to the calling execution.
Context records, progress, logs, traceability and document mutations also
accept operation-specific request identity options through `_with` methods.
