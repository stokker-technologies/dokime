<!-- Rendered by stkr project update from .agents/skills/dokime-authoring/references/plugins.md; sha256:6852a234ae08a9e1134bff741b1671f8c83d588eb1e4d91c2eb99cede1995867 -->

# Dokime Plugin

Plugins depend on the `dokime-runtime` SDK. Shared pure contracts belong in `dokime-domain`; installation, supervision, storage, credentials and side effects belong to runtime host services. The bare `dokime` crate is the installable CLI. A plugin may contribute no suites; do not invent suites for continuous producers.

A Dokime plugin is a Harmos plugin. `#[dokime_runtime::plugin]` forwards to `#[harmos::plugin]` and adds only the host operations Dokime's own contributions need:

```rust
#[dokime_runtime::plugin(
    id = "bench",
    description = "Owns the bench operations and evidence",
    provides = domain::operations::Operations,
    requires(),
    contributions(
        dokime_runtime::Suites<suites::Suites>,
        dokime_runtime::Records<Records, listeners::Listeners>,
        dokime_runtime::Evidence<observers::Streams>,
        dokime_runtime::Settings<domain::model::settings::BenchSettings>,
    ),
    observers = observers::Observers,
    observer_control = observers::controls::Observe,
    restart = "never",
)]
impl dokime_runtime::Plugin for Bench {
    type Config = Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;
    fn prepare(config: Configuration) -> Result<Self, Refusal>;
    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal>;
    fn resources(&self) -> Arc<Resources>;
    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<domain::operations::Operations, _>(self.resources())?;
        table.bind_operations::<listeners::Listeners, _>(self.resources())?;
        dokime_runtime::execution::serve::<Self, suites::Suites>(table)
    }
    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        dokime_runtime::execution::interrupt().await;
        Ok(())
    }
    async fn cleanup(&mut self) -> Result<(), Refusal>;
}
fn main() -> std::io::Result<()> { dokime_runtime::plugin::run::<Bench>() }
```

- Payloads, records and reusable measurement rows use `#[dokime_runtime::message]`, `#[dokime_runtime::record(...)]` and `#[dokime_runtime::row]`. These wrappers use one shared message codec/schema implementation; record/row field annotations retain their domain meaning. Do not stack declarations or use the retired `TelemetryRow` derive.
- Built-in host rows use explicit nested `dokime_runtime::host::HostTimestamp { value: String }` and `HostTextList { values: Vec<String> }` message fields. Optional lists preserve absence separately from an empty list. Native ingestion and stored-row decoding project these to ordinary datetime and array values; raw Arrow paths include the nested `value` or `values` member. Use the shared host table/schema and codec, not a second host byte format.
- Device behaviour is a `#[dokime_runtime::operation]` on an inherent impl with `async fn initialize(resources: Arc<Resources>, ctx: &Supervisor) -> Self` (or `Result<Self, Error>`) and `async fn run(&mut self, ctx: &Supervisor, request: &Req) -> Result<Resp, Refusal>`; `stop` and `cleanup` run only after a successful initialization, listed in a `#[dokime_runtime::catalog]` and named by `provides`. Requests and responses are `#[dokime_runtime::message]` values or `()`. There are no routes, route tables, handles or route contexts.
- `requires(...)` names only operations another plugin or the host provides. Requiring an operation grants it; an operation whose request names a channel, stream or source also takes its scopes, `requires(Publish: scope("chamber"))`, and an unscoped operation needs nothing more. A plugin's own operations stay Private (the default) and are callable by its own runs without being listed; Harmos refuses a plugin that requires its own operation. Share an operation with other plugins with `visibility = Shared` on its declaration.
- Contributions are inert, namespaced sections the host reads without running the plugin: `Suites`, `Records`, `Evidence`, `Settings`. List only the domains the plugin contributes.
- Whatever a template declares, the plugin registers. `#[dokime_runtime::plugin]` derives the native stream grant from its suites' templates (`streams = Streams` and each template observer's streams), its plugin observers and `dokime_runtime::Evidence<Streams>`; the prompts and records a template declares reach the host through its suite. Never repeat them on the plugin: there is no `streams(...)` argument and no plugin-level prompt contribution. `Records` names only what no template declares (listener records, records published outside a run), `Evidence` only streams no template or observer publishes, plus analyses.
- Several declarations may name one stream, record or prompt; identical ones register once. Two that disagree about one stream id (row type, schema version, window or metadata) stop the plugin's build; a disagreement the macro cannot see — a record's or prompt's fields, a stream's columns — is refused at admission.
- Listeners are operations: `#[dokime_runtime::listener(id, record = Type)]` on an inherent impl with `hear(&mut self, ctx, record)`. The listener constructs with `async fn initialize(resources: Arc<Resources>, ctx: &Supervisor)`. The host forwards a published record to every listener that declared its type. Publish a record outside a run with `dokime_runtime::plugin::Publisher::new(&supervisor).publish(record)`.
- Settings are host-authored UI declarations; `prepare` decodes them with `config.settings::<S>()` from the admitted `Configuration`. A setting's readiness probe and a discovered default link operations, not routes. Runtime settings and secret references are distinct from test parameters.
- Plugin observers are Harmos observers sharing the original initialized resources through `ObserverAttachment`; never duplicate hardware handles per callback. An observer constructs with `async fn initialize(resources: &Resources) -> Result<Self, Error>` (or `Default`), works in `observe`, and ends with `stop` and `cleanup`. At the current Harmos pin the plugin serves them through one Host control, `observer_control = Observe`, declared with `observers::POLICY` and bound with `bind::<Observe>(table)`; after the Harmos re-pin that control is the built-in `harmos.observe`, bound automatically when a plugin declares observers, and the `cargo dokime new` starter is already written that way.
- `dokime_runtime::plugin::refusal(result)` turns any error into the `Refusal` an operation boundary returns and logs the full diagnostic.

Static binary inspection reads the inert declaration and never initializes a plugin or acquires resources. One lifecycle owner initializes resources and settles work before releasing them, including partial failures. Plugin install/update/remove/enable/disable changes take effect through a full app restart; keep pending-restart feedback. Development-bundle reload is a separate explicit lifecycle.

Read `crates/dokime/src/{lib.rs,plugin/,contributions.rs,execution/serving.rs}`, `crates/dokime-macros/src/plugin.rs`, and the `cargo dokime new` starter under `crates/cargo-dokime/templates/` for current signatures. Use `dokime-authoring` for execution authoring, `dokime-authoring` for evidence/document behavior, and `dokime-testing` when native desktop behavior is affected. Discover current mise tasks rather than copying obsolete smoke commands.
