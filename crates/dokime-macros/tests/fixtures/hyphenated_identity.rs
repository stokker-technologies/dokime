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
    cancellation_timeout: std::time::Duration::ZERO,
    retries: sdk::harmos::operations::RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Uppercase;

#[sdk::operation(id = "hyphenated.frobnicate", description = "Invalid.", policy = POLICY, response = ())]
#[derive(PartialEq)]
struct Frobnicate {}

/* === Enums === */
#[sdk::harmos::catalog(harmos_path = sdk::harmos)]
enum UppercaseOperations {
    Frobnicate(Frobnicate),
}

/* === Traits === */

/* === Implementations === */
impl sdk::Operation for Frobnicate {
    type Error = sdk::Refusal;

    async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
        Ok(().into())
    }
}

impl sdk::harmos::builtins::lifecycle::Policy for Uppercase {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(id = "upper-case", requires(), restart = "never")]
impl sdk::Plugin for Uppercase {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[UppercaseOperations::CATALOG];
}

impl sdk::harmos::Lifecycle for Uppercase {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    async fn prepare(_: sdk::plugin::Configuration, setup: &mut sdk::harmos::Setup<Self>) -> Result<Self, sdk::Refusal> {
        let owner = { Self };
        setup.resource::<()>(owner.resources());
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

impl Uppercase {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() {}

/* === Tests === */
