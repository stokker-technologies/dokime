# Dokime Rust SDK

Author executable test plugins and call a running host with the `dokime` crate.
The [Capture Suite](examples/test/src/suite.rs) owns its records, steps and criteria.

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

The [Plugin Entry Point](examples/test/src/main.rs) only serves the sidecar.
The separate [Client Flow](examples/client/src/main.rs) starts a compiled plugin,
attaches it to a host, reads its suites and detaches it before joining the child.

```powershell
mise -C sdk/rust run build
mise -C sdk/rust run test -- --layer unit
mise -C sdk/rust run test
# With a host already running:
./target/debug/dokime-rust-client http://127.0.0.1:7000 ./target/debug/dokime-rust-test
```

To run only the sidecar, run `./target/debug/dokime-rust-test`. Its native entry calls `plugin.run()`.
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
