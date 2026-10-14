/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
const POLICY: dokime::harmos::operations::OperationPolicy =
    dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: dokime::harmos::operations::RetryPolicy::Never,
        restart: dokime::harmos::operations::RestartPolicy::Never,
    };

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Plugin;

struct Marker;

struct Ping;

/* === Enums === */
#[dokime::catalog]
enum Operations {
    Ping(Ping),
}

/* === Traits === */

/* === Implementations === */
impl dokime::Contribution for Marker {
    const NAMESPACE: &'static str = "fixture.marker";
    const SECTION: dokime::harmos::plugin::inventory::Section =
        dokime::harmos::plugin::inventory::Section::new("fixture.marker", "{}");

    fn register(
        registrar: &mut dokime::harmos::plugin::Registrar,
    ) -> Result<(), dokime::harmos::plugin::RegistrationError> {
        if let Some(path) = std::env::var_os("FIXTURE_EXPORT_MARKER") {
            std::fs::write(path, b"registered").unwrap();
        }
        registrar.register(Self::NAMESPACE, 1, &["fixture:read:data"])
    }
}

#[dokime::operation(id = "independent-owner.ping", version = 1, description = "Answers.",
    policy = POLICY
)]
impl Ping {
    async fn initialize(_: std::sync::Arc<()>, _: &dokime::Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &dokime::Supervisor, _: &()) -> Result<(), dokime::Refusal> {
        Ok(())
    }
}

impl dokime::harmos::operations::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = POLICY;
}

// This fixture is only ever built and inspected, never activated.
#[dokime::plugin(
    id = "independent-owner",
    provides = Operations,
    requires(),
    contributions(Marker),
    restart = "never", description = "A plugin whose bundle is built and inspected, never run.")]
impl dokime::Plugin for Plugin {
    type Config = dokime::plugin::Configuration;
    type Error = dokime::Refusal;
    type Context = dokime::Supervisor;
    type Resources = ();

    fn prepare(_: dokime::plugin::Configuration) -> Result<Self, dokime::Refusal> {
        panic!("bundle construction must never prepare the plugin")
    }

    async fn initialize(&mut self, _: &dokime::Supervisor) -> Result<(), dokime::Refusal> {
        Ok(())
    }

    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }

    fn bind_operations(
        &self,
        table: &mut dokime::harmos::operations::WireTable<dokime::Supervisor>,
    ) -> Result<(), dokime::harmos::operations::ServingError> {
        table.bind_operations::<Operations, _>(self.resources())
    }

    async fn stop(&mut self, _: &dokime::Supervisor) -> Result<(), dokime::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), dokime::Refusal> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    if std::env::var_os("FIXTURE_INVALID_EXPORT").is_some() {
        println!("[] trailing output");
        return Ok(());
    }
    dokime::plugin::run::<Plugin>()
}

/* === Tests === */
