/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


const POLICY: sdk::harmos::operations::OperationPolicy = sdk::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: sdk::harmos::operations::RetryPolicy::Never,
};


#[sdk::operation(id = "mismatched.read_voltage", description = "Reads.", policy = POLICY, response = ())]
#[derive(PartialEq)]
struct ReadVoltage {}

struct Probe;

struct Checks;

struct Plugin;

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


impl sdk::Operation for ReadVoltage {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
        Ok(().into())
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

impl sdk::harmos::builtins::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(
    id = "mismatched",
    requires(),
    contributions(sdk::Suites<PluginSuites>),
    restart = "never",
)]
impl sdk::Plugin for Plugin {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[];
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

    async fn initialize(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn stop(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}

impl Plugin {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}


fn main() -> std::io::Result<()> {
    sdk::plugin::run::<Plugin>()
}
