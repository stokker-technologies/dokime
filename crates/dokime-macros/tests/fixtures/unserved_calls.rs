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
#[sdk::message]
#[derive(Clone, PartialEq)]
#[sdk::operation(id = "mismatched.read_voltage", version = 1, description = "Reads.", policy = POLICY, response = ())]
struct ReadVoltage {}

struct ReadVoltageHandler;

struct Probe;

struct Checks;

struct Plugin;

/* === Enums === */
#[sdk::catalog]
enum ProbeRequired {
    ReadVoltage(ReadVoltage),
}

#[sdk::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[sdk::catalog]
enum PluginSuites {
    Checks(Checks),
}

#[sdk::catalog]
enum NoOperations {}

/* === Traits === */

/* === Implementations === */
#[sdk::operation]
impl ReadVoltageHandler {
    pub async fn initialize(_: std::sync::Arc<()>, _: &sdk::Supervisor) -> Self {
        Self
    }

    async fn run(&mut self, _: &sdk::Supervisor, _: &mut ReadVoltage) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

#[sdk::template(id = "mismatched.checks.probe", label = "Probe", requires = ProbeRequired)]
impl Probe {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }

    #[sdk::step(label = "Run")]
    async fn run(&mut self, ctx: &mut sdk::execution::StepContext<'_>) -> Result<(), std::convert::Infallible> {
        let _ = ctx.call(ReadVoltage {}).await;

        Ok(())
    }
}

#[sdk::suite(id = "mismatched.checks", label = "Checks", templates = ChecksTemplates)]
impl Checks {}

impl sdk::harmos::operations::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "mismatched",
    provides = NoOperations,
    requires(),
    contributions(sdk::Suites<PluginSuites>),
    restart = "never",
)]
impl sdk::Plugin for Plugin {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    type Resources = ();

    fn prepare(config: sdk::plugin::Configuration) -> Result<Self, sdk::Refusal> {
        Ok(Self)
    }

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }

    fn bind_operations(
        &self, table: &mut sdk::harmos::operations::WireTable,
    ) -> Result<(), sdk::harmos::operations::ServingError> {
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
