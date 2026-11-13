# Rust SDK

The `dokime` crate at `sdk/rust` contains plugin authoring and a typed client.
Its product dependencies stop at Dokime/Harmos protocol, SDK and derive support;
plugins and clients compile no host. It uses Harmos at
`26010c03516060deb5f70d17af3fbf0e49d9fbe2`.
The module layout follows Harmos's Rust SDK: a curated `prelude`, value composition,
context, data contracts, streams and the separate `client` namespace.

## Use the SDK

Until the combined crate cutover lands, use an explicit path dependency in an
ordinary external Cargo project:

```toml
[dependencies]
dokime = { path = "../dokime/sdk/rust" }
```

```rust
use dokime::prelude::*;

#[derive(dokime::Row)]
#[harmos(package = "thermal.v1")]
struct Temperature { celsius: f64 }

#[derive(dokime::Parameters)]
struct Parameters { limit: f64 }

async fn sample(ctx: &Context) -> Result<Temperature> {
    Ok(Temperature { celsius: ctx.parameter::<f64>("limit")? - 1.0 })
}

fn main() -> Result<()> {
    let temperature = Stream::<Temperature>::new("thermal.temperature", "Temperature [°C]");
    let observer = Observer::new(&temperature, sample).every_ms(1000);
    let template = Template::new("thermal.cycle", "Thermal Cycle")
        .parameters(Parameters { limit: 85.0 })
        .stream(temperature)
        .observer(observer);
    let suite = Suite::new("thermal", "Thermal").template(template);
    Plugin::new("thermal", "1.0.0").suite(suite).serve()
}
```

The complete runnable [thermal suite](../sdk/rust/examples/thermal.rs) declares
its parameter defaults, steps, criterion, prompt, record and host requirements.
`Message`, `Row`, `Record`, `Parameters` and `Settings` are field-reading protobuf
derives. Suite, template, step, criterion, observer, operation, prompt, record and
stream registration use owned values, with no declaration macros.
The data derive is local to the SDK so an author needs only `dokime`.

Scalar, optional, repeated, nested message and byte fields use protobuf encoding.
`#[harmos(tag = 3)]` pins a field tag; `package`, `file` and `name` set its descriptor
identity. Explicit tags are useful for shared cross-language contracts. Parameters
and settings are data messages; `.parameters(defaults)` and `.settings(defaults)`
build their catalogs. Typed parameter reads retain integer signedness, check narrow
integer ranges and reject unknown fields and nonfinite numbers. Settings catalog
number defaults use `f64`, matching the protocol's `NumberSetting`; use `.setting(...)`
for choices, paths, secrets or other explicit metadata. `.parameter(...)`, `.file(...)`,
`.document(...)`, `.tag(...)` and `.parameter_group(...)` accept the generated catalog
values directly. Document/plot builders are available as `dokime::builders`.

## Compose a Plugin

Step, criterion, initialize and cleanup bodies have the fixed `&Context` signature:

```rust
use dokime::prelude::*;

struct Bench { limit: f64 }

async fn initialize(ctx: &Context) -> Result<()> {
    ctx.set(Bench { limit: ctx.parameter("limit")? });
    Ok(())
}

async fn ramp(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    ctx.progress(1, 2, format!("Target {} °C", bench.limit)).await
}

async fn acceptable(ctx: &Context) -> Result<bool> {
    let stats = ctx.statistics("thermal.temperature", "celsius", "hold").await?;
    Ok(stats.count > 0 && stats.max <= ctx.get::<Bench>()?.limit)
}
```

`ctx.set(value)` replaces by Rust type; `ctx.get::<T>()` returns an `Arc<T>` or a
missing-resource error naming the type. A suite shares one store across its
initialize, steps, observers, criteria and cleanup, isolated by execution identity.
Use synchronization inside a resource when it needs shared mutation. A generic
operation joins an existing local suite store when its inherited scope matches;
otherwise its resources last for that invocation. Plugin-wide publication has its
own unscoped store. No resource is serialized or retained by materialization.

The SDK generates Protected `<template>.materialize`, optional
`<template>.initialize`, `<template>.step.<index>`, `<template>.criterion.<index>` and
`<template>.cleanup` operations. Step/criterion evidence identifiers are separate
from operation identifiers; `.identifier(...)` overrides their label-derived defaults.
`.retries(n)` declares replay safety when `n > 0` and advertises host retry policy with a 100 ms backoff when none was
provided; it does not retry inside the plugin.
`.policy(...)` accepts Harmos's full operation policy.
An execution stream also gets a no-op initialize operation when there is no author
hook, so its case is available before the first observer tick.
The SDK always advertises cleanup, even without an author hook, to release the
execution store. Cleanup empties the store even while a pending observer holds its
call Context. Contexts are borrowed for one call; resource handles provide sharing
between calls. Cleanup releases resources after success, failure or cancellation;
`ctx.termination()` exposes the host's termination value. Plugin Stop also clears stores.

Materialize copies the resolved parameters into a `dokime.v1.TestCase` assembled
from the registered steps, criteria and streams. `.materialize(|case, parameters| ...)`
can customize that value synchronously without a Context or execution resources.
It may select, repeat or reorder registered operations; each resulting evidence
identifier must be unique. It cannot reference undeclared operations/streams or
change the host's resolved parameter set. Duplicate identifiers fail catalog
construction before serving. All execution calls must include the frozen case and
matching `PluginRun` origin, and their execution must equal inherited Harmos scope.

Use `Step::replay_safe()` to declare replay safety with a custom retry policy.

Ordinary typed operations compose through `Operation::new(id, visibility, handler)` with
`(&Context, Request) -> Result<Response>`. `OperationRequest` connects a message to
its response and stable operation identifier. `.operation(...)` serves it;
`.requires::<Request>()` declares a child-call dependency. Such calls run through
`ctx.operation(request)` and retain Harmos cancellation and failure behavior.

## Host Calls and Evidence

Generated Dokime messages bind through `dokime::host::Call<T>`:

```rust
use dokime::{host, prelude::*, v1};

async fn note(ctx: &Context) -> Result<()> {
    ctx.operation(host::Call(v1::WriteLog {
        execution: ctx.execution_id().into(),
        severity: v1::Severity::Info as i32,
        message: "Measurement Started".into(),
        at: None,
    })).await?;
    Ok(())
}
```

Declare `.requires_host::<v1::WriteLog>()` on the plugin for that call.
`ctx.host(request)` accepts the generated request directly and returns its generated
reply. `prompt`, `record`, `records`, `statistics`, `progress`, `log`, `secret`,
`settings` and `typed_settings` are thin conveniences. Artifact, file capture,
traceability, document-part, environment and effect calls use `ctx.host(...)` with
the generated messages and their declared requirements.
`ctx.sleep` and `ctx.now` require `dokime::clock::TimeSleepRequest` and `dokime::clock::TimeGetRequest`;
all waits and clock reads go to the host.

`Record::<Door>::new(id, label)` names a derived record message. Pass the record
value to `.record(...)` on Plugin and `ctx.record(&contract, value)` to publish it.
Publication waits for the host receipt. The record type accumulator is inferred;
it contributes record descriptors to the one `dokime.v1.RecordContracts` section.
`Prompt::new(...).field(...).action(...)` composes a catalog prompt;
`ctx.prompt(v1::OperatorQuestion { ... })` uses its identifier or an inline definition.

`Observer::new(&stream, handler).every_ms(...)` answers host ticks with rows stamped
by Harmos using the tick time. `.interval_setting(...)` selects a saved cadence.
A device-driven stream uses `.publish_on(handler)` and `Publisher::publish` or
`publish_at`; demand closure cancels the producer. Template-owned stream names are
included in the TestCase. `.uses_stream(id)` subscribes without declaring another
producer; Plugin-level `.stream(...)` declares a plugin-wide stream.

Managed placement is the default. `plugin.remote().serve()` uses the same gRPC
service and remains available after Stop. Both honor `HARMOS_ADDRESS` and announce
`harmos listening <address>` when ready. `into_harmos()` finishes composition for
Harmos tooling without opening a socket; `catalog()` validates and returns its catalog.

## Use the Client

The client accepts a configured Tonic channel or `harmos::Client`. Callers own TLS,
endpoint selection and wire/queue bounds. Authentication and per-call metadata
are forwarded through Harmos; operation policy stays with the server.

```rust
use dokime::{Client, client::v1};

async fn status(client: &Client) -> Result<v1::StatusGetResponse, tonic::Status> {
    Ok(client.status.get(tonic::Request::new(v1::StatusGetRequest {})).await?.into_inner())
}

```

Follow catalog cursors until the desired contract is found. `Method` checks public
visibility, request/reply names, descriptors and fingerprints. `.with_input::<T>()`
and `.with_output::<T>()` bind declared streaming directions; `client.start(...)`
returns Harmos's managed call with `send`, `finish_input`, `next`, `join` and
cancellation. `take_input` separates concurrent input from output draining.
No SDK transport codec, retry loop or host policy duplicates Harmos.

Legacy product declarations remain for the existing loop's pending atomic cutover.
The new host serves generic identity/health/availability through the library Status
resource and no longer binds product Backend or Runtime services. `HostClient`
accepts explicit client limits and exposes the same library root through its transport.
The new host serves
the subset listed in [Execution Host](execution-host.md), refusing the rest by name.
`client::HostClient::new(channel, limits)?` supplies thin service conveniences:

```rust
let client = dokime::client::HostClient::new(channel, limits)?;
let cases = client.templates.materialize(suite_id, "thermal.cycle", parameters).await?;
let cycle = client.cycles.start(cases).await?;
let mut progress = cycle.watch();
while let Some(frame) = progress.next().await? {
    if frame.phase == "prompt" {
        let pending = client.prompts.pending(cycle.executions[0].clone()).await?;
        client.prompts.respond(pending.prompts[0].prompt_id.clone(), "yes", true).await?;
    }
}
drop(progress);
drop(cycle);
client.close();
```

The generated-service client owns no runtime or native host dependency. Finish
owned watches before closing; closing consumes the client's channel handles.

## Focused Proof

Run these tasks from the repository root:

```sh
mise -C sdk/rust run sdk:build
mise -C sdk/rust run sdk:test
mise -C sdk/rust run sdk:tree
```

The current pin refuses template-local bindings at generic reference-host
admission; the exact request is tracked under
[Library Requests](../openspec/changes/operation-grammar/tasks.md#library-requests).

The external consumer in `sdk/rust/tests/consumer` owns the focused test targets and
all test-only reference-host dependencies. Its child-process fixture reuses the
existing standalone `crates/dokime-runtime-next/src/process/` constructor by source path
to retain the repository's console-window policy without depending on that crate.
It compiles the SDK from its actual path and builds the sample plugin. Its standalone lock isolates SDK proof from the
concurrent host/facade cutover; `.prepare` refreshes that consumer's dependency lock
with `mise -C sdk/rust run sdk:prepare`. All artifacts share the repository target.
`cargo tree` checks normal/build product edges and excludes test-only host dependencies.

The thermal story admits and materializes the plugin through the pinned Harmos
runtime, then drives scoped calls through `harmos.plugin.v1.Instance`, following the reference
conformance protocol. Native Harmos calls accept `Scope` through `with_scope(...)`;
unbound calls retain an empty scope. This protocol-focused story keeps its explicit
wire driver, and SDK execution validation reads `Context::scope()`. The client
story uses an in-process fake gRPC server for authentication, discovery, schema
binding/refusal, unary results, managed output/final results and response bounds.
Other focused stories cover catalog/data construction and interleaved execution
resource isolation and deterministic resource release during a pending observer
read. Tests never assert elapsed time or sleep on the wall clock.

## Shared Cross-Language System Suite

The runnable Rust suite is [thermal.rs](../sdk/rust/examples/thermal.rs), built as
`dokime-rust-thermal` by the external consumer. Run it managed or with `--remote`.
It runs through the new host and provides the reference evidence for every SDK:

| Input or Action | Expected Evidence |
| --- | --- |
| Materialize `thermal.cycle` with `limit = 85.0` | Steps `ramp`, `hold`; criterion `within_limit`; stream `thermal.temperature` |
| Initialize | Execution-local Bench resource; one acknowledged log |
| Run `ramp` | Progress 1 of 2 |
| Run `hold`; answer prompt `thermal.door` with accepted action `yes` | One `thermal.door` record, protobuf `thermal.v1.Door { closed: true }`; host sleep of 2.5 s; progress 2 of 2 |
| Tick the observer while Hold waits, at 0 s, 1 s and 2 s after its start | Three `thermal.v1.Temperature { celsius: 84.0 }` samples, each carrying its tick time |
| Evaluate `within_limit` | Statistics for `thermal.temperature.celsius` and records for `thermal.door`, both filtered to `hold`; Passed only with samples, maximum ≤ 85.0 and a closed-door record |
| Cleanup, close and seal | Cleanup log; resources released; streams closed; host-persisted execution and report evidence |

The host must register the sample's catalog requirements, serve materialization,
freeze the returned case and complete PluginRun origin, and carry execution identity
in Harmos Request/Publish scope. It must initialize before ticking resource-using
observers, schedule ticks inside the hold step, acknowledge records before responding,
serve statistics/record reads scoped to execution and step, and own retries, cancellation,
cleanup, stream closure, evidence durability and report sealing. Its virtual clock
must advance sleep and observer ticks deterministically. The shared pull request and Nightly driver
compares persisted evidence across SDKs through the named stories in
[Execution Host](execution-host.md#compare-every-sdk). Native Linux and hosted
matrix proof remain separate from local Windows runs.

## Consumer Cutover

| Owner | Adoption Work |
| --- | --- |
| CLI | Use `dokime::client` and generated messages for remote commands; carry pairing, backend identity validation, endpoint/TLS policy and authorization into the configured channel/client. Move embedded hosting to `dokime-runtime-next`. |
| Desktop | Use SDK typed calls and managed watches; convert presentation data at the GUI boundary. Embed runtime explicitly and keep pairing/identity handling with its current owner. |
| Existing plugins and samples | Replace declaration macros with builder values and data derives, move shared suite state to `ctx.set/get`, declare host requirements, and serve Harmos gRPC. |

These migrations belong to the host/consumer owner. The Rust SDK adds no host-enabling
features and leaves those existing crates and plugins untouched.

## Composed Client Services

`dokime::Client` exposes the library client root, including `client.access.listener`,
`client.access.pairings` and `client.access.certificates`, on its existing transport.
Use the library request wrappers and explicit request metadata.
