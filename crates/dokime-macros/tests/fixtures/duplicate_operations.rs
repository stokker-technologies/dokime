/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod first {
    use super::POLICY;

    #[sdk::operation(id = "duplicated.read", description = "Reads.", policy = POLICY, response = ())]
    #[derive(PartialEq)]
    pub struct Read {}

    impl sdk::Operation for Read {
        type Error = sdk::Refusal;

        async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
            Ok(().into())
        }
    }
}

mod second {
    use super::POLICY;

    #[sdk::operation(id = "duplicated.read", description = "Reads.", policy = POLICY, response = ())]
    #[derive(PartialEq)]
    pub struct Read {}

    impl sdk::Operation for Read {
        type Error = sdk::Refusal;

        async fn run(self, _: sdk::harmos::Context<'_, Self>) -> Result<sdk::harmos::Response<()>, sdk::Refusal> {
            Ok(().into())
        }
    }
}

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
struct Duplicated;

/* === Enums === */
#[sdk::harmos::catalog(harmos_path = sdk::harmos)]
enum DuplicatedOperations {
    First(first::Read),
    Second(second::Read),
}

/* === Traits === */

/* === Implementations === */
impl sdk::harmos::builtins::lifecycle::Policy for Duplicated {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(id = "duplicated", requires(), restart = "never")]
impl sdk::Plugin for Duplicated {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[DuplicatedOperations::CATALOG];
}

impl sdk::harmos::Lifecycle for Duplicated {
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

impl Duplicated {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() {
    // Schema markers link only through a reachable `markers()` call, as the
    // sidecar runtime makes; inspection resolves contracts against them.
    std::hint::black_box(<Duplicated as sdk::harmos::plugin::Declared>::markers()).unwrap();
}

/* === Tests === */
