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
use std::{io::Write, path::PathBuf, sync::Arc, time::Duration};
// Third Party Imports
use dokime::harmos::operations::{
    Host, OperationDeclaration, OperationPolicy, RestartPolicy, RetryPolicy, ServingError,
    Visibility, WireOperation, WireTable,
};
// Local Imports
use dokime::{Refusal, Supervisor};

/* === Constants === */
const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: RetryPolicy::Never,
    restart: RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::section(
    label = "Fixture",
    description = "This fixture's whole settings card: a lifecycle marker path, test wiring rather than an operator knob.\nOptional because a required path has to declare a default the catalog accepts, and there is no sensible one: the run that wants the trace names the file, and a run that does not just leaves it unset."
)]
#[derive(Debug, Default, PartialEq)]
struct Config {
    lifecycle: Option<PathBuf>,
}

#[dokime::settings]
#[derive(Debug, Default, PartialEq)]
struct DependentSettings {
    config: Config,
}

pub struct CheckHealth;

struct Dependent {
    lifecycle: Option<PathBuf>,
}

/* === Enums === */
#[dokime::catalog]
pub enum Operations {}

/* === Traits === */

/* === Implementations === */
impl OperationDeclaration for CheckHealth {
    type Request = ();
    type Response = ();
    const ID: &'static str = "validation-telemetry.check_health";
    const DESCRIPTION: &'static str =
        "Answers whether the fixture is reachable; a dependent plugin calls it.";
    const VISIBILITY: Visibility = Visibility::Shared;
    const POLICY: OperationPolicy = POLICY;
}

impl WireOperation for CheckHealth {
    const VERSION: u32 = 1;
}

impl dokime::harmos::operations::lifecycle::Policy for Dependent {
    const INITIALIZATION_POLICY: OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: OperationPolicy = POLICY;
}

#[dokime::plugin(
    id = "fixture-dependent",
    provides = Operations,
    requires(CheckHealth),
    contributions(dokime::Settings<DependentSettings>),
    restart = "never", description = "Checks dependency lifecycle routing.")]
impl dokime::Plugin for Dependent {
    type Config = dokime::plugin::Configuration;
    type Error = Refusal;
    type Context = Supervisor;
    type Resources = ();

    fn prepare(config: dokime::plugin::Configuration) -> Result<Self, Refusal> {
        let settings = dokime::plugin::refusal(config.settings::<DependentSettings>())?;
        Ok(Self {
            lifecycle: settings.config.lifecycle,
        })
    }

    async fn initialize(&mut self, host: &Supervisor) -> Result<(), Refusal> {
        dokime::plugin::refusal(self.record(host, "initialize").await)
    }

    fn resources(&self) -> Arc<()> {
        Arc::new(())
    }

    fn bind_operations(&self, table: &mut WireTable<Supervisor>) -> Result<(), ServingError> {
        table.bind_operations::<Operations, _>(self.resources())
    }

    async fn stop(&mut self, _: &Supervisor) -> Result<(), Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Refusal> {
        // Lifecycle Shutdown Does Not Hold The Dependency's Active Operation Lease.
        // Record Cleanup Locally After The Dependency Call Exercised Initialization.
        if let Some(lifecycle) = &self.lifecycle {
            let mut file = dokime::plugin::refusal(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(lifecycle),
            )?;
            dokime::plugin::refusal(writeln!(file, "cleanup:{}", std::process::id()))?;
        }
        Ok(())
    }
}

impl Dependent {
    async fn record(&self, host: &Supervisor, phase: &str) -> std::io::Result<()> {
        host.call_as::<CheckHealth>(())
            .await
            .map(|response| response.into_owned())
            .map_err(std::io::Error::other)?;
        let Some(lifecycle) = &self.lifecycle else {
            return Ok(());
        };
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(lifecycle)?;
        writeln!(file, "{phase}:{}", std::process::id())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    dokime::plugin::run::<Dependent>()
}

/* === Tests === */
