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
const POLICY: sdk::harmos::operations::OperationPolicy = sdk::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: sdk::harmos::operations::RetryPolicy::Never,
    restart: sdk::harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct GetVoltage;

struct GetCurrent;

struct Probe;

struct Checks;

struct Plugin;

/* === Enums === */
#[sdk::catalog]
enum ProbeRequired {
    GetVoltage(GetVoltage),
}

#[sdk::catalog]
enum PluginOperations {
    GetVoltage(GetVoltage),
    GetCurrent(GetCurrent),
}

#[sdk::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[sdk::catalog]
enum PluginSuites {
    Checks(Checks),
}

/* === Traits === */

/* === Implementations === */
#[sdk::operation(id = "undeclared.get_voltage", version = 1, description = "Reads.", policy = POLICY)]
impl GetVoltage {
    pub async fn initialize(_: std::sync::Arc<()>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &sdk::Supervisor, _: &()) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

#[sdk::operation(id = "undeclared.get_current", version = 1, description = "Reads.", policy = POLICY)]
impl GetCurrent {
    pub async fn initialize(_: std::sync::Arc<()>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &sdk::Supervisor, _: &()) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

#[sdk::template(id = "undeclared.checks.probe", label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(
        _: &mut sdk::execution::InitializationContext<'_>,
    ) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    async fn run(
        &mut self,
        ctx: &mut sdk::execution::StepContext<'_>,
    ) -> Result<(), std::convert::Infallible> {
        let _ = ctx.call::<GetVoltage>(()).await;
        // Served by the plugin, and never declared here: the bound reads the
        // template's own `requires`, not the plugin's operation list.
        let _ = ctx.call::<GetCurrent>(()).await;

        Ok(())
    }
}

#[sdk::suite(id = "undeclared.checks", label = "Checks", templates = ChecksTemplates)]
impl Checks {}

impl sdk::harmos::operations::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "undeclared",
    provides = PluginOperations,
    requires(),
    contributions(sdk::Suites<PluginSuites>),
    restart = "never",
)]
impl sdk::Plugin for Plugin {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    type Resources = ();

    fn prepare(_: sdk::plugin::Configuration) -> Result<Self, sdk::Refusal> {
        Ok(Self)
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }

    fn bind_operations(
        &self,
        table: &mut sdk::harmos::operations::WireTable<sdk::Supervisor>,
    ) -> Result<(), sdk::harmos::operations::ServingError> {
        table.bind_operations::<PluginOperations, _>(self.resources())?;
        sdk::execution::serve::<Self, PluginSuites>(table)
    }

    async fn stop(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    sdk::plugin::run::<Plugin>()
}

/* === Tests === */
