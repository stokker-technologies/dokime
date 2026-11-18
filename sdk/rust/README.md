# Dokime Rust SDK

Author executable test plugins and call a running host with the `dokime` crate.
The [Capture Suite](examples/test/src/suite.rs) owns its records, steps and criteria.

## Author a Suite

```rust
use dokime::prelude::*;

struct Capture;
impl Capture {
    fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }
    fn template() -> Template {
        Template::of::<Self>("Capture")
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
let plugin = Plugin::new("example", "0.1.0").suite(Suite::single(Capture::template()));
```

This declares template `capture`, step `capture.confirm`, criterion
`capture.confirmed` and record `capture.confirmation`. The record uses
`google.protobuf.BoolValue`, matching the Python Capture suite.
`Suite::new("example", "Example").template(Capture::template())` instead binds
`example.capture` and prefixes its members with `example.capture.`.

`Template::of::<T>` generates a name from the type, removing a trailing `Template`
or `Suite`. Steps and criteria use their function names. Word boundaries become
underscores; labels never choose identifiers. `Template::new` and member
`.identifier(...)` values are explicit identifiers. Anonymous handlers require an
explicit identifier. Keep record, prompt and stream declarations local to their
template; reuse declarations without changing them when composing another suite.

## Values and Operations

Records and simple operations accept `bool`, `String`, `i64` and `f64`; the SDK
encodes their well-known protobuf wrappers. Structured rows use `#[derive(Message)]`
or `#[derive(dokime::Record)]` with `#[dokime(package = "example")]` when needed.

```rust
use dokime::prelude::*;
let echo = Operation::new("example.echo", Visibility::Public,
    async |_: &Context, request: String| Ok(request));
let plugin = Plugin::new("example", "0.1.0").operation(echo);
```

Declare structured questions with `dokime::plugin::Prompt` on the template and
pass the declaration to `ctx.prompt(&question, &[])`. The call returns the chosen
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

To serve only the sidecar, run `./target/debug/dokime-rust-test --remote`.
`dokime::testing::Harness` runs a template against an in-memory host
with a controlled clock, without hardware.
