---
name: dokime-suite
description: Author Dokime suites, steps, criteria, execution resources, and cleanup.
---

# Dokime Suite

Plugin ownership belongs to `dokime-plugin`; document composition belongs to `dokime-document`. Read current SDK source for exact signatures. Global guidance owns validation cadence.

## Suite Authoring

- Declare a `#[dokime::catalog] pub enum Suites` in `suites/mod.rs`, one typed tuple variant per suite. Contribute it with `contributions(dokime::Suites<suites::Suites>)` on `#[dokime::plugin]` and serve it with `dokime::execution::serve::<Self, suites::Suites>(table)` in `bind_operations`. Omit empty catalogs.
- Declare suites with `#[dokime::suite]` and templates with `#[dokime::template]`. A suite owns its complete template collection.
- A template's `id` is required and is its public reference: `<suite id>.<segment>`, the segment `[a-z][a-z0-9_]*` (`mini.charger.steady.measurement`). The macro refuses a malformed literal on its line; the suite refuses, while the plugin compiles, a template outside its prefix or two templates with one id. The CLI, the clients and the catalog name templates by this id; the registered UUID (`template_id = …` pins it) is storage and wire identity only, never shown or accepted from users.
- Put every authored template in its own Rust file under the owning suite module. Keep the suite declaration in `mod.rs` and list its templates on it, `templates = Templates`, re-exporting public template types as needed.
- Steps act through `&mut StepContext`; criteria judge through `&CriterionContext`; observers publish through `&mut ObserverContext`, the attempt's own handles and the call the engine made.
- One `Environment` per execution carries what the run was given — its inputs decoded into the template's own `#[dokime::parameters]` struct, settings, secrets, the caller that reaches operations through the host, the attempt's own handles, the artifact grant, the stream grant — and every stage reads it the same way. `InitializationContext`, `ResourceContext`, `TraceabilityContext`, `StepContext`, `ObserverContext` and `CleanupContext` are stage aliases over one `Context`; only a step may `prompt` and `listen`, a step or a template observer may `publish` and reach `stream::<Row>()`, only cleanup has `reason()`, and only the stages that run after `resources` reach `ctx.resources()`. Every one of those six — `initialize`, `resources`, `traceability`, a step, an observer, `cleanup` — also reaches `ctx.document()`, adding to what the template's `document()` declared; a criterion reaches no document at all, deciding only from what the run already produced.
- **A template never names the plugin.** It declares the operations it calls — `requires = Required` on `#[dokime::template]`, a `#[dokime::catalog]` of operation types, the way it already declares `subscribes = Inputs` and `records = Records` — and `ctx.call::<Operation>(request)` is the only road to the plugin from any stage. The dependency runs plugin → suites → templates → operation contracts, so a suite can be served by any plugin that provides or requires what its templates call. There is no `plugin = …` argument and no `ctx.plugin()` on a template stage.
- `ctx.call::<ReadChamber>(()).await?` is a Harmos child call routed by the host under the operation's declared policy; it carries the run's `ExecutionScope { execution, stage, route }` in the call context, from which the serving operation derives idempotency keys (`ExecutionScope::serving::<Self>(ctx)`). Populated payloads are `#[dokime::message]` values; empty requests and responses use `()`. In the harness, `.answering::<ReadChamber, _>(|request| Ok(reply))` answers the same call in process. `ctx.host()` is the same route as the Harmos `Host` surface: `ctx.host().call(op)` and `ctx.host().submit(op)` take a declaration that is its own request (`call_as`/`submit_as` name one with a separate request), and a submitted instance is cancelled when its handle drops unsettled unless detached. `ctx.host()` is not checked against `requires` at compile time; the host refuses an undeclared operation at run time, so prefer `ctx.call` where it suffices.
- Two step built-ins answer on `ctx.host()` without being listed in `requires`. `RunProcess` (`dokime::operations::process`) runs a system process inside the plugin process: its working directory is the execution's artifact folder (`DOKIME_ARTIFACTS_DIR`), its environment carries `DOKIME_EXECUTION_ID`, `DOKIME_CYCLE_ID`, `DOKIME_TEMPLATE_ID`, `DOKIME_PLUGIN_ID`, `DOKIME_STEP_ID` and `DOKIME_ATTEMPT` and never the plugin's host connection, stdout lines reach the execution log at info and stderr at warning unless redirected into a produced artifact, and every exit is recorded as `process.exited`. Called, it runs to its exit; submitted, `cancel` sends SIGTERM, waits `grace_ms` (default 2000) and kills, TerminateProcess on Windows. `WatchFile` (`dokime::operations::files`) is the host's file watch over one local or FTP file: Stream captures only what is written after it starts into `artifact` (or `<output>/<file name>`), Mirror and the snapshots copy the whole file, and on Stream its `matchers` turn each captured line into a row of a declared stream through named capture groups and its `markers` file a declared record per matching line, both inside the operation. It settles with `{ artifacts, gaps }` when its `maximum_seconds` bound elapses or the execution seals; cancelling it seals what it captured. The harness and `Caller::local` serve both in process, `WatchFile` over a local file.

```rust
let exit = ctx
    .host()
    .call_as::<RunProcess>(
        RunProcessRequest::new("flash-tool")
            .args(["--port", port])
            .stdout(Output::Artifact("logs/flash.log".into())),
    )
    .await?
    .into_owned(); // ProcessExit { exit_code, duration_ms, cancelled }

let watch = ctx
    .host()
    .submit_as::<WatchFile>(WatchFileRequest {
        execution_id: None,
        source: Some(FileWatchSource {
            payload: Some(FileWatchSourcePayload::Local(LocalWatchSource { root: log })),
        }),
        output: "device".into(),
        artifact: Some("device/charger.log".into()),
        policy: FileWatchPolicy::Stream,
        poll_interval_ms: Some(100),
        maximum_seconds: Some(30),
        maximum_bytes: None,
        matchers: vec![LineMatcher::publishing::<ChargerLog>(
            r"t=(?P<elapsed_s>[\d.]+) input_power_w=(?P<input_power_w>[\d.]+)",
        )],
        markers: vec![LineMarker::new(r"FAULT (?P<code>\w+)", Fault::RECORD_TYPE)],
    })
    .await?;
// … the work the log describes …
let captured = watch.join().await?.into_owned(); // WatchFileOutcome { artifacts, gaps }
```

- Anything a step leaves running is cancelled when the step ends: a handle dropped unsettled cancels its instance at once, and a detached instance, or one still held past its step, is cancelled by the step-end sweep with a `test.step.handle_cancelled { step, kind, name }` warning, `kind` being `process`, `watch` or `operation`.
- Two compile-time checks stand behind `requires = Required`. A `ctx.call` on an operation the template did not list is refused on the line the call is written. And serving a suite asserts, for every template, that each required operation is either named in the plugin's `requires(...)` or provided by the plugin itself with Private visibility, so an operation the plugin cannot reach is refused at build time rather than at run time.
- `#[dokime::message]`, `#[dokime::record(...)]` and `#[dokime::row]` are domain-specific declarations over one shared Harmos message implementation. Their codecs/schema are shared; record and row wrappers add their product evidence traits and handles. Preserve `#[record(...)]` and `#[row(...)]` field annotations. The separate `TelemetryRow` derive is retired; the runtime trait remains available through `dokime::telemetry`.
- A record is a type, declared once, in the file of whatever publishes it: `#[dokime::record(id, label, severity)]` on a struct beside the step that files it, named once in `records = Records` on that template's attribute. The struct's fields are the contract's fields, so there is no `records()` method, no hand-built `RecordContract` and no string-id constant. A suite names a record only when every template it owns publishes it; `dokime::Records<Records, Listeners>` on the plugin's `contributions(...)` names the ones a plugin publishes with no run behind them.
- A prompt is a type, declared once with `#[dokime::prompt(id, label, description, submit, cancel, level)]` on a struct in `prompts/`; author runtime text explicitly with `description` while the repository is below 1.0.0. `level = warning` (the record severity scale: `debug`, `info`, `warning`, `error`; default `info`) is declared per prompt type, not per asking: it colours the operator's banner and the request record. The hand-built `FormPrompt` and `ConfirmationPrompt` take `.level(RecordSeverity::Warning)`. The struct's fields are the form's fields — kind resolved from the type through `dokime::prompts::PromptFieldValue`, `Option<T>` optional, `Vec<T>` multiple, `#[prompt(label, description, default, min, max, min_length, max_length, pattern, min_items, max_items)]` for the rest, a field's runtime text written explicitly with `description` — and the struct is what the answer decodes into; a unit struct is a confirmation answering `bool`. Name it in `prompts = Prompts` on the template attribute and ask it with `ctx.ask::<SoakCheckpoint>()`. A runtime default goes through the derived builder, `ctx.prompt(SoakCheckpoint::prompt().default("note", seen))`. The hand-built `FormPrompt`/`TextPrompt`/`NumberPrompt`/`ConfirmationPrompt`/`ChoicePrompt` builders are the programmatic path, declared as `prompts = [checkpoint()]` and asked with `ctx.prompt(checkpoint())`; do not write a function that only restates a shape a struct could declare.
- Declare a stream once with `#[dokime::stream(id = "device.samples", label = "Samples", window = 4096)]` on a named struct; `description = "…"`, like every other argument here, forwards whole to Harmos, which owns the shared metadata contract; author runtime text explicitly with `description` below 1.0.0. It supplies the shared protobuf/Postcard message codecs and schema, metadata, execution traits, and document column handles, which compare straight into document filters (`SampleRow::column_voltage_v().gt(0.5)`); derive Clone explicitly when the authored consumer needs it. Do not stack `message`, `record` or `row` on that stream. `#[row(label, description, tags(...), attributes(...), role, unit, format)]` annotates its fields, each forwarded to Harmos the same way, runtime text written explicitly with `description`: Harmos owns generic metadata and Dokime owns format and supported evidence roles. For one shape serving multiple streams, use `#[dokime::row]` on Shape (with `#[derive(Clone)]` when needed) and `#[dokime::stream(id, label, window, row = Shape)]` on each unit struct. `foreign` declares a typed read-only field subset of another plugin’s stream; name at least one field, and never publish it. Register streams explicitly in `streams = Streams`; an observer declares its output streams with `streams = Streams`. The plugin serving the suite registers every stream, record and prompt its templates declare; nothing is repeated on `#[dokime::plugin]`, and two templates sharing a stream, record or prompt declare it identically. Publication and listening remain typed.
- `ctx.publish(value)` is the one verb for both: a `#[dokime::record]` struct is a record and a `#[dokime::stream]` row is a measurement, and the declaration decides which without the call repeating it. `ctx.sample` and `ctx.emit` are gone. A publication whose severity this occurrence decides says so at the call — `ctx.publish(Measured { … }.severity(RecordSeverity::Error))`. A handle held open across the work is `ctx.stream::<Row>().open()`; it carries the context's lifetime and not its borrow, so pushing a row and logging a line do not compete for one `&mut`. It is infallible, because the stream is the home the row declared. One shape serving several streams declares no home and names its stream at the call with `ctx.stream::<Row>().into::<Stream>()`.
- `ctx.listen::<T>()` is the one way a step waits, for a declared record type or for a row type whose stream was declared. It answers a standard stream and the adapters are `tokio_stream::StreamExt`'s, re-exported through `dokime::prelude`: `.filter(…)`, `.next()`, `.take(n)`, `.collect()`, `.timeout(d)`. Waiting on its own observer is `ctx.listen::<Row>().take(n)` — an exact number of rows rather than a length of time, so what the run measured does not depend on how quickly it got going — and every row an execution produces goes through one funnel, whether an observer measured it or a step pushed it.
- Where a subscription starts follows from what it listens to. A stream is a stream of measurements and has no "before" to reach, so a subscription sees what is measured after it is taken. An execution's record inbox is a bounded queue the host handed that run, so it starts where the run started and a record filed before the step reached its await still satisfies it; an ordinary record a step published itself is passed over, because answering a step with its own words would settle the question it asked somebody else. `.timeout(d)` yields `Result<T, Elapsed>` per item and its `Elapsed` carries into the template's own error through `#[from]`.
- Plugin event observation uses the same observer mechanism as periodic rows, with `records = Records` on the observer and no separate catalog. A plugin observer may publish no rows and only events. Template-local event observation is the same `#[dokime::observer]` associated function as periodic rows, with `records = Records` and `ctx: &mut ObserverContext<'_>`; its lifecycle belongs to the execution and it declares emitted records on the template.
- A criterion reads its evidence typed. `ctx.records::<Measured>()` answers the records of that type the run published, in order, refusing one of that type that does not carry what it declared. `ctx.samples::<C>()` answers the rows of one stream as the shape that stream declared — an own row type, a stream type, `host::System` — so a criterion reads `row.celsius` rather than digging a `Value` out of a map. It is infallible: a row the declared shape cannot read is narrated and passed over. It answers `dokime::evidence::Samples<Row>`, which reads as a slice of rows (`iter`, `len`, `last`, `for row in …`) and turns one numeric field into a Harmos series: `ctx.samples::<Samples>().column(|row| row.voltage_rms_v)` is a `dokime::evidence::Series`, so the criterion reads `.median()`, `.percentile(95.0)`, `.std()`, `.all_below(limit)` or `.all_within(min, max)` off it, or `.aggregate(Aggregate::Rms)` through the `Aggregates` trait. A column skips non-finite values the way every aggregate does, so `.count()` is how many values were measured.
- The aggregate vocabulary is Harmos's and Dokime only names it: `dokime::evidence::{Aggregate, Aggregates, Series, Summary}` re-export `harmos::Aggregate` (`Count`, `Sum`, `Mean`, `Median`, `Min`, `Max`, `Rms`, `Std`, `Percentile(u8)`, wire names `count` … `p95`) and `harmos::signal`. `Std` is the population deviation and percentiles interpolate between ranks. Never hand-roll a mean, a deviation or a quantile in a criterion, a host path or a plugin.
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
- Each phase ends by saying what it learned: a `dokime::info!(ctx, …)`/`ctx.debug(…)` line,
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
- Log through the `dokime::trace!(ctx, …)` … `dokime::error!(ctx, …)` macros,
  never `tracing::*` nor the bare context methods inside a template: only the
  context reaches the execution log, and the macros carry the format string.


## Template File Style

A template file is one Rust file in the suite module, laid out in the repository skeleton with every marker kept, even empty. Its items appear in this order and nowhere else:

- `Type Aliases`: an optional `type Context<'a> = StepContext<'a, Template, Resources, Parameters>;` when the full context type is spelled more than once.
- `Structs`: the `Template` struct first, holding the run's own state as plain `pub` fields; then `#[dokime::parameters] pub struct Parameters`; then each `#[dokime::prompt(id, label, description)] pub struct …` as a unit struct; then each `#[dokime::stream(id, label, window)] pub struct …Row`.
- `Enums`: the catalogs, always named `Required`, `Subscriptions`, `Streams`, `Prompts`, listed in that order and only when non-empty. Catalog variants are grouped under `// <Group> Operations` comment lines (`// Navigation Operations`, `// Measurement Operations`) with a blank line between groups, and a variant is named after the operation type it wraps.
- `Implementations`: the one `#[dokime::template(...)]` impl with hooks in lifecycle order: `initialize`, `traceability`, the steps in execution order, the criteria, `cleanup`, `document`. Private helpers go under `Private Functions`.

Declarations:

- `#[parameter(label = "Title Case", unit = "s", default = 30, min = 1, max = 60)]` on every parameter, one blank line between fields. The label is what the operator reads; the unit is a symbol.
- A stream row groups its fields under short Title Case comment lines (`// Reconnection Timing`, `// Measurement`), gives every physical column `#[row(unit = "…")]`, and types a measurement that can be missing as `Option<T>`; never a sentinel value.
- `#[dokime::template]` carries `id = "<suite>.<segment>"`, a `label` whose hierarchy reads `"Area / Name"` (each materialized case is labelled `<label> #<n>`, numbered per template within its cycle, so never encode parameter values in it), a `description` written as full sentences for the operator, lowercase `tags`, and the four catalog bindings by name.
- A prompt's `description` tells the operator what to do and when to continue, in full sentences.

Bodies:

- Every operation call builds its request first and passes it by name: `let operation = StartMeasurementRequest { … };` then `ctx.call::<StartMeasurement>(operation).await?;`, with the reply bound as `let result = …`. Fields are listed one per line.
- A step's phases are banners as described under Step Bodies. Inside a loop body, shorter Title Case comment lines (`// Move To Reference Angles`, `// Emit Result`) mark its sub-steps; banners stay at phase level.
- Iterate with `ctx.progress("Label", range)` so the cockpit shows the loop's progress.
- A measurement that may legitimately fail inside a loop is matched: the `Ok` arm yields `Some(value)`, the `Err` arm warns through `dokime::warn!(ctx, format!(…))` and yields `None`, and the row is published either way.
- Rows and records are published with `ctx.publish(row).await`, the one infallible verb.
- Log through `dokime::trace!(ctx, …)` … `dokime::error!(ctx, …)`, format arguments inline; never the bare `ctx.info(…)` methods.
- A criterion's label states the rule the way an operator would (`"Reconnection Time < 30s"`). Its body reads the evidence once, returns `Ok(false)` when there is none, then applies one iterator predicate or one aggregate over a `.column(…)` series.
- `cleanup` logs the reason from `ctx.reason()` and restores the bench through operations, in the reverse order of what the steps changed.
- `traceability` builds each section by statements (`let mut dut = TraceabilitySection::new(…); dut.row(…);`) under one banner per section and returns the sections in order.
- `document` builds each section by statements too (`let mut signals = Section::new("Soak"); signals.plot(temperature);`): the label derives the section id as it does a plot's, and `signals.id(…)` overrides it only when something else addresses the section by id.

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
