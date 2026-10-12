/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod error;
mod operations;
mod settings;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;
// Third Party Imports
use dokime::harmos::operations::{OperationPolicy, RetryPolicy, ServingError, WireTable};
// Local Imports
use dokime::{Refusal, Supervisor};
use error::Error;
use settings::RoutePluginSettings;

/* === Constants === */
const HOME_VARIABLE: &str = if cfg!(windows) { "USERPROFILE" } else { "HOME" };

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    calls: AtomicU64,
    scoped_context: bool,
    connected: bool,
}

struct RoutePlugin {
    resources: Arc<Resources>,
}

/* === Enums === */
#[dokime::catalog]
pub enum Streams {}

#[dokime::catalog]
pub enum Records {
    Observed(operations::publications::Observed),
}

/* === Traits === */

/* === Implementations === */
impl dokime::harmos::operations::lifecycle::Policy for RoutePlugin {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::plugin(
    id = "route-fixture",
    provides = operations::Operations,
    requires(),
    contributions(
        dokime::Records<Records>,
        dokime::Evidence<Streams, operations::analysis::Analyses>,
        dokime::Settings<RoutePluginSettings>,
    ),
    restart = "never", description = "Exercises independently installed operation-local domain bindings.")]
impl dokime::Plugin for RoutePlugin {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = Resources;

    #[tracing::instrument(level = "debug", skip_all)]
    fn prepare(config: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<RoutePluginSettings>())?;
        let scoped_context = settings.fixture.images_directory
            == std::path::Path::new("fixture-images")
            && config.secret("other.owner").is_err()
            && config.setting::<String>("home").is_err()
            && std::env::var(HOME_VARIABLE).is_err()
            && settings.fixture.home.is_some();
        tracing::info!("route fixture initialized");
        Ok(Self {
            resources: Arc::new(Resources {
                calls: AtomicU64::new(0),
                scoped_context,
                connected: settings.fixture.token.is_some(),
            }),
        })
    }

    async fn initialize(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    fn resources(&self) -> Arc<Resources> {
        Arc::clone(&self.resources)
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<operations::Operations, _>(self.resources())
    }

    async fn shutdown(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    dokime::plugin::run::<RoutePlugin>()
}

/* === Tests === */
