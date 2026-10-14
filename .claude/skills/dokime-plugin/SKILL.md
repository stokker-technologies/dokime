---
name: dokime-plugin
description: Author Dokime plugin ownership, operations, contributions, settings, and lifecycle boundaries.
---
<!-- Generated from .agents/skills/dokime-plugin/SKILL.md; edit .agents/skills/dokime-plugin/SKILL.md and run `stkr project update` -->

# Dokime Plugin

Plugins depend on the `dokime` SDK facade, never `dokime-app`. Shared portable contracts belong in `dokime-contract`; application installation, supervision, storage, credentials and side effects belong in `dokime-app`. A plugin may contribute no suites; do not invent suites for continuous producers.

A Dokime plugin is a Harmos plugin. `#[dokime::plugin]` forwards to `#[harmos::plugin]` and adds only the host operations Dokime's own contributions need:

```rust
#[dokime::plugin(
    id = "bench",
    description = "Owns the bench operations and evidence",
    provides = operations::Operations,
    requires(),
    contributions(
        dokime::Suites<suites::Suites>,
        dokime::Records<Records, listeners::Listeners>,
        dokime::Evidence<observers::Streams>,
        dokime::Settings<settings::BenchSettings>,
    ),
    streams(suites::soak::SoakRow),
    observers = observers::Observers,
    observer_control = observers::controls::Observe,
    restart = "never",
)]
impl dokime::Plugin for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;
    fn prepare(config: Configuration) -> Result<Self, Refusal>;
    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal>;
    fn resources(&self) -> Arc<Resources>;
    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<operations::Implemented, _>(self.resources())?;
        table.bind_operations::<listeners::Listeners, _>(self.resources())?;
        dokime::execution::serve::<Self, suites::Suites>(table)
    }
    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime::execution::interrupt().await;
        Ok(())
    }
    async fn cleanup(&mut self) -> Result<(), Refusal>;
}
fn main() -> std::io::Result<()> { dokime::plugin::run::<Bench>() }
```

- Payloads, records and reusable measurement rows use `#[dokime::message]`, `#[dokime::record(...)]` and `#[dokime::row]`. These wrappers use one shared message codec/schema implementation; record/row field annotations retain their domain meaning. Do not stack declarations or use the retired `TelemetryRow` derive.
- Device behaviour is a `#[dokime::operation]` on an inherent impl with `async fn initialize(resources: Arc<Resources>, ctx: &Supervisor) -> Self` (or `Result<Self, Error>`) and `async fn run(&mut self, ctx: &Supervisor, request: &Req) -> Result<Resp, Refusal>`; `stop` and `cleanup` run only after a successful initialization, listed in a `#[dokime::catalog]` and named by `provides`. Requests and responses are `#[dokime::message]` values or `()`. There are no routes, route tables, handles or route contexts.
- `requires(...)` names only operations another plugin or the host provides, with their grants. A plugin's own operations stay Private (the default) and are callable by its own runs without being listed; Harmos refuses a plugin that requires its own operation. Share an operation with other plugins with `visibility = Shared` on its declaration.
- Contributions are inert, namespaced sections the host reads without running the plugin: `Suites`, `Records`, `Prompts`, `Evidence`, `Settings`. List only the domains the plugin contributes.
- Listeners are operations: `#[dokime::listener(id, record = Type)]` on an inherent impl with `hear(&mut self, ctx, record)`. The listener constructs with `async fn initialize(resources: Arc<Resources>, ctx: &Supervisor)`. The host forwards a published record to every listener that declared its type. Publish a record outside a run with `dokime::plugin::Publisher::new(&supervisor).publish(record)`.
- Settings are host-authored UI declarations; `prepare` decodes them with `config.settings::<S>()` from the admitted `Configuration`. A setting's readiness probe and a discovered default link operations, not routes. Runtime settings and secret references are distinct from test parameters.
- Plugin observers are Harmos observers sharing the original initialized resources through `ObserverAttachment`; never duplicate hardware handles per callback. An observer constructs with `async fn initialize(resources: &Resources) -> Result<Self, Error>` (or `Default`), works in `observe`, and ends with `stop` and `cleanup`. The plugin serves them through one Host control, `observer_control = Observe`, declared with `observers::POLICY` and bound with `bind::<Observe>(table)`.
- `dokime::plugin::refusal(result)` turns any error into the `Refusal` an operation boundary returns and logs the full diagnostic.

Static binary inspection reads the inert declaration and never initializes a plugin or acquires resources. One lifecycle owner initializes resources and settles work before releasing them, including partial failures. Plugin install/update/remove/enable/disable changes take effect through a full app restart; keep pending-restart feedback. Development-bundle reload is a separate explicit lifecycle.

Read `crates/dokime/src/{lib.rs,plugin/,contributions.rs,execution/serving.rs}`, `crates/dokime-macros/src/plugin.rs`, and the `cargo dokime new` starter under `crates/cargo-dokime/templates/` for current signatures. Use `dokime-suite` for execution authoring, `dokime-document` for evidence/document behavior, and `dokime-desktop-validation` when native desktop behavior is affected. Discover current mise tasks rather than copying obsolete smoke commands.
