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
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[sdk::operation(id = "undeclared.get_voltage", description = "Reads.", policy = POLICY, response = ())]
#[derive(PartialEq)]
struct GetVoltage {}

#[sdk::operation(id = "undeclared.get_current", description = "Reads.", policy = POLICY, response = ())]
#[derive(PartialEq)]
struct GetCurrent {}

struct Probe;

struct Checks;

struct Plugin;

/* === Enums === */
#[sdk::catalog]
enum ProbeRequired {
    GetVoltage(GetVoltage),
}

#[sdk::harmos::catalog(harmos_path = sdk::harmos)]
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
impl sdk::Operation for GetVoltage {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
        Ok(().into())
    }
}

impl sdk::Operation for GetCurrent {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
        Ok(().into())
    }
}

#[sdk::template(id = "undeclared.checks.probe", label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    async fn run(&mut self, ctx: &mut sdk::execution::StepContext<'_>) -> Result<(), std::convert::Infallible> {
        let _ = ctx.call(GetVoltage {}).await;
        // Served by the plugin, and never declared here: the bound reads the
        // template's own `requires`, not the plugin's operation list.
        let _ = ctx.call(GetCurrent {}).await;

        Ok(())
    }
}

#[sdk::suite(id = "undeclared.checks", label = "Checks", templates = ChecksTemplates)]
impl Checks {}

impl sdk::harmos::builtins::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "undeclared",
    requires(),
    contributions(sdk::Suites<PluginSuites>),
    restart = "never",
)]
impl sdk::Plugin for Plugin {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[PluginOperations::CATALOG];
}

impl sdk::harmos::Lifecycle for Plugin {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    async fn prepare(config: sdk::plugin::Configuration, setup: &mut sdk::harmos::Setup<Self>) -> Result<Self, sdk::Refusal> {
        let owner = { Self };
        setup.resource::<()>(owner.resources());
        sdk::execution::prepare::<Self, PluginSuites>(setup)?;
        Ok(owner)
    }
}

impl Plugin {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() -> std::io::Result<()> {
    sdk::plugin::run::<Plugin>()
}

/* === Tests === */
