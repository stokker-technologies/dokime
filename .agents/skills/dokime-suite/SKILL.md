---
name: dokime-suite
description: Author Dokime suites, steps, criteria, execution resources, and cleanup.
---

# Dokime Suite

Plugin ownership belongs to `dokime-plugin`; document composition belongs to `dokime-document`. Read current SDK source for exact signatures. Global guidance owns validation cadence.

## Suite Authoring

- Declare a `#[dokime::catalog] pub enum Suites` in `suites/mod.rs`, one typed tuple variant per suite. Contribute it with `contributions(dokime::Suites<suites::Suites>)` on `#[dokime::plugin]` and serve it with `dokime::execution::serve::<Self, suites::Suites>(table)` in `bind_operations`. Omit empty catalogs.
- Declare suites with `#[dokime::suite]` and templates with `#[dokime::template]`. A suite owns its complete template collection.
- Put every authored template in its own Rust file under the owning suite module. Keep the suite declaration in `mod.rs` and list its templates on it, `templates = Templates`, re-exporting public template types as needed.
- Steps act through `&mut StepContext`; criteria judge through `&CriterionContext`; observers publish through `&mut ObserverContext`, the attempt's own handles and the call the engine made.
- One `Environment` per execution carries what the run was given — its inputs decoded into the template's own `#[dokime::parameters]` struct, settings, secrets, the caller that reaches operations through the host, the attempt's own handles, the artifact grant, the stream grant — and every stage reads it the same way. `InitializationContext`, `ResourceContext`, `TraceabilityContext`, `StepContext`, `ObserverContext` and `CleanupContext` are stage aliases over one `Context`; only a step may `prompt` and `listen`, a step or a template observer may `publish` and reach `stream::<Row>()`, only cleanup has `reason()`, and only the stages that run after `resources` reach `ctx.resources()`. Every one of those six — `initialize`, `resources`, `traceability`, a step, an observer, `cleanup` — also reaches `ctx.document()`, adding to what the template's `document()` declared; a criterion reaches no document at all, deciding only from what the run already produced.
- **A template never names the plugin.** It declares the operations it calls — `requires = Required` on `#[dokime::template]`, a `#[dokime::catalog]` of operation types, the way it already declares `subscribes = Inputs` and `records = Records` — and `ctx.call::<Operation>(request)` is the only road to the plugin from any stage. The dependency runs plugin → suites → templates → operation contracts, so a suite can be served by any plugin that provides or requires what its templates call. There is no `plugin = …` argument and no `ctx.plugin()` on a template stage.
- `ctx.call::<ReadChamber>(()).await?` is a Harmos child call routed by the host under the operation's declared policy; it carries the run's `ExecutionScope { execution, stage, route }` in the call context, from which the serving operation derives idempotency keys (`ExecutionScope::serving::<Self>(ctx)`). Populated payloads are `#[dokime::message]` values; empty requests and responses use `()`. In the harness, `.answering::<ReadChamber, _>(|request| Ok(reply))` answers the same call in process.
- Two compile-time checks stand behind `requires = Required`. A `ctx.call` on an operation the template did not list is refused on the line the call is written. And serving a suite asserts, for every template, that each required operation is either named in the plugin's `requires(...)` or provided by the plugin itself with Private visibility, so an operation the plugin cannot reach is refused at build time rather than at run time.
- A record is a type, declared once, in the file of whatever publishes it: `#[dokime::record(id, label, severity)]` on a struct beside the step that files it, named once in `records = Records` on that template's attribute. The struct's fields are the contract's fields, so there is no `records()` method, no hand-built `RecordContract` and no string-id constant. A suite names a record only when every template it owns publishes it; `dokime::Records<Records, Listeners>` on the plugin's `contributions(...)` names the ones a plugin publishes with no run behind them.
- A prompt is a type, declared once with `#[dokime::prompt(id, label, description, submit, cancel)]` on a struct in `prompts/`; author runtime text explicitly with `description` while the repository is below 1.0.0. The struct's fields are the form's fields — kind resolved from the type through `dokime::prompts::PromptFieldValue`, `Option<T>` optional, `Vec<T>` multiple, `#[prompt(label, description, default, min, max, min_length, max_length, pattern, min_items, max_items)]` for the rest, a field's runtime text written explicitly with `description` — and the struct is what the answer decodes into; a unit struct is a confirmation answering `bool`. Name it in `prompts = Prompts` on the template attribute and ask it with `ctx.ask::<SoakCheckpoint>()`. A runtime default goes through the derived builder, `ctx.prompt(SoakCheckpoint::prompt().default("note", seen))`. The hand-built `FormPrompt`/`TextPrompt`/`NumberPrompt`/`ConfirmationPrompt`/`ChoicePrompt` builders are the programmatic path, declared as `prompts = [checkpoint()]` and asked with `ctx.prompt(checkpoint())`; do not write a function that only restates a shape a struct could declare.
- Declare a stream once with `#[dokime::stream(id = "device.samples", label = "Samples", window = 4096)]` on a named struct; `description = "…"`, like every other argument here, forwards whole to Harmos, which owns the shared metadata contract; author runtime text explicitly with `description` below 1.0.0. It supplies Clone, Serialize/Deserialize, protobuf/schema, shared metadata, execution traits, and document column handles; do not stack `message`, `telemetry`, or `derive(TelemetryRow)` on that stream. `#[row(label, description, tags(...), attributes(...), role, unit, format)]` annotates its fields, each forwarded to Harmos the same way, runtime text written explicitly with `description`: Harmos owns generic metadata and Dokime owns format and supported evidence roles. For one shape serving multiple streams, use `#[dokime::message]` with `#[derive(Clone, dokime::TelemetryRow)]` on Shape and `#[dokime::stream(id, label, window, row = Shape)]` on each unit struct. `foreign` declares a typed read-only field subset of another plugin’s stream; name at least one field, and never publish it. Register streams explicitly in `streams = Streams`; an observer declares its output streams with `streams = Streams`. Publication and listening remain typed.
- `ctx.publish(value)` is the one verb for both: a `#[dokime::record]` struct is a record and a `#[dokime::stream]` row is a measurement, and the declaration decides which without the call repeating it. `ctx.sample` and `ctx.emit` are gone. A publication whose severity this occurrence decides says so at the call — `ctx.publish(Measured { … }.severity(RecordSeverity::Error))`. A handle held open across the work is `ctx.stream::<Row>().open()`; it carries the context's lifetime and not its borrow, so pushing a row and logging a line do not compete for one `&mut`. It is infallible, because the stream is the home the row declared. One shape serving several streams declares no home and names its stream at the call with `ctx.stream::<Row>().into::<Stream>()`.
- `ctx.listen::<T>()` is the one way a step waits, for a declared record type or for a row type whose stream was declared. It answers a standard stream and the adapters are `tokio_stream::StreamExt`'s, re-exported through `dokime::prelude`: `.filter(…)`, `.next()`, `.take(n)`, `.collect()`, `.timeout(d)`. Waiting on its own observer is `ctx.listen::<Row>().take(n)` — an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going — and every row an execution produces goes through one funnel, whether an observer measured it or a step pushed it.
- Where a subscription starts follows from what it listens to. A stream is a stream of measurements and has no "before" to reach, so a subscription sees what is measured after it is taken. An execution's record inbox is a bounded queue the host handed that run, so it starts where the run started and a record filed before the step reached its await still satisfies it; an ordinary record a step published itself is passed over, because answering a step with its own words would settle the question it asked somebody else. `.timeout(d)` yields `Result<T, Elapsed>` per item and its `Elapsed` carries into the template's own error through `#[from]`.
- Plugin event observation uses the same observer mechanism as periodic rows, with `records = Records` on the observer and no separate catalog. A plugin observer may publish no rows and only events. Template-local event observation is the same `#[dokime::observer]` associated function as periodic rows, with `records = Records` and `ctx: &mut ObserverContext<'_>`; its lifecycle belongs to the execution and it declares emitted records on the template.
- A criterion reads its evidence typed. `ctx.records::<Measured>()` answers the records of that type the run published, in order, refusing one of that type that does not carry what it declared. `ctx.samples::<C>()` answers the rows of one stream as the shape that stream declared — an own row type, a stream type, `host::System` — so a criterion reads `row.celsius` rather than digging a `Value` out of a map. It is infallible: a row the declared shape cannot read is narrated and passed over.
- `subscribes = Inputs` names the global streams the host leases for every execution of this template, by type. Cadence belongs to the stream's one producer, never to the requester, so nothing here states an interval and every run leasing the same stream shares one running instance and one tick. A stream no installed plugin publishes is refused at admission. A step that wants the same stream only while it is doing something takes `ctx.lease::<C>()` instead, and the hold ends where the guard does.
- Import the execution grants from `dokime::execution`, suite and template declarations from `dokime::definition`, and the plugin lifecycle from `dokime::Plugin` and `dokime::plugin`.
- Read captured ordinary settings with `ctx.setting::<T>("id")?`. Steps and runtime plugin contexts read declared secrets synchronously with `ctx.secret("id")?`; its redacted `Secret` exposes the value only through `.expose()`. Criteria have settings access but no secret accessor. Static catalog methods have neither runtime settings nor secret access.
- Host startup loads dotenv and resolves declared owner-scoped references; do not copy the host environment into a plugin or read it when declaring catalogs.
- Contracts are written by the declarations, not by hand: `#[dokime::record]` and `#[dokime::stream]` build a strict contract from the struct's fields, and naming the type on a template's attribute is what binds it to that template. The `declared`/`strict`/`for_template` constructors remain for the host's own catalogs and for tests about the contract shape itself.
- The runtime installs a plugin's full contribution set atomically and rejects duplicate suite, template, record, and telemetry identifiers.


## Step Bodies

A step is the procedure an operator reads, so its body reads top to bottom as
that procedure: one linear function, split into phases, each opened by a banner.

```rust
#[dokime::step(label = "Move the Tool")]
async fn move_tool(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
    // === Load Control Points === //

    let job = ListControlPointsRequest { point_type: PointType::Control };
    let points = ctx.call::<ListControlPoints>(job).await?.points;
    dokime::info!(ctx, "loaded {} control points", points.len());

    // === Prompt Operator / Move Tool === //

    let accepted = ctx.ask::<MoveToolCheckpoint>().await?;
    if !accepted {
        dokime::warn!(ctx, "operator declined the move");
        return Ok(());
    }
    ctx.resources().arm.lock().await.move_to(&points).await?;
    dokime::info!(ctx, "tool moved through {} points", points.len());

    Ok(())
}
```

- A banner is `// === Phase Name === //` in Title Case, with a blank line on
  both sides. It names what the phase does for the operator — `Load Control
  Points`, `Prompt Operator / Move Tool` — not the mechanism. A step with one
  phase carries no banner.
- Each phase ends by saying what it learned: a `dokime::info!`/`debug!` line,
  a `ctx.publish`, or an artifact write. The execution log then reads as the
  same list of phases the source does.
- Keep the happy path shallow and readable; no universal indentation cap is required.
  `?`, `let … else`, and an early `return Ok(())` end the step; a `match` on a
  `listen` result is fine, a loop inside a `match` inside an `if` is not. A
  phase that needs a loop keeps the loop body to a few lines.
- Do not extract thin helpers. A function called once, from one step, is that
  step's phase and lives inline under its banner. Extract shared behavior, cohesive responsibilities, or meaningful invariant owners, and put it on a
  second inherent `impl` below the declared one, never in a nested closure.
- The same banners apply to `resources`, `traceability` and `cleanup` when a
  hook has more than one phase. A criterion is one read over captured evidence
  and takes none.
- Log through the `dokime::trace!` … `dokime::error!` macros, never
  `tracing::*` inside a template: only the context reaches the execution log.


## Layout

```text
plugins/<name>/
  Cargo.toml
  src/lib.rs
  src/operations/
  src/suites/
  src/prompts/
  src/records/
  src/settings/
```

Create only populated feature modules. App-backed plugins use top-level `app/`,
`app-runtime/`, and `app-plugin/` crates: application definitions and Harmos
assembly belong to `app-runtime`; the thin `app` facade and Dokime plugin depend
on it. Do not add a separate definitions crate or global ids file. Keep records
and stream producers/listeners beside their feature, with suite-only observation
inside that suite.

- Concern modules exist only when the integration contributes that domain.
- Generate a `cargo dokime new` starter for the worked example of a complete suite: one template that owns its data, opens its handles in `resources()`, states traceability, declares the record it publishes beside the step that files it, listens for a host record by type, holds on its own observer by listening for its own row type, prompts, and decides two criteria. Use `plugins/mini` for the maintained onboard charger demonstrator and its embedded testbench runtime; `crates/dokime-app/tests/fixtures/route_plugin` exercises host plumbing rather than teaching product suite structure.
- Use `snake_case` for Rust module folders/files and `PascalCase` for types.
- Keep assets next to the domain that declares them.


## Execution And Lifecycle

- Every authored runnable suite executes through the plugin that contributed it: each template is served as a `<template>.run` operation that pumps evidence to the host and a `<template>.signal` operation through which the host cancels, commands and delivers records.
- Sidecar crashes and supervised process restarts do not unload the active catalog. Plugin install/update/remove/enable/disable changes become active only after a full Dokime app restart; do not reintroduce live catalog replacement.
- Admission freezes resolved parameters, definitions, settings and secret references, and provider source/content digest in the immutable execution snapshot. Later plugin or settings changes cannot retarget admitted work or alter its inputs.
- `Invalidated` is a distinct terminal outcome. Preserve collected results and evidence when an admitted execution becomes invalidated; do not turn invalidation into failure, cancellation, or an invented completion.
- Preserve replay-safe prompt, telemetry, record, and artifact identities.
- Use `dokime-plugin` for lifecycle, operations, publications, and supervision.
