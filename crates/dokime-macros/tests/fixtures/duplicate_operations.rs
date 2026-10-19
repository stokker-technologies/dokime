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

    #[sdk::message]
    #[derive(Clone, PartialEq)]
    #[sdk::operation(id = "duplicated.read", version = 1, description = "Reads.", policy = POLICY, response = ())]
    pub struct Read {}

    pub struct ReadHandler;

    #[sdk::operation]
    impl ReadHandler {
        pub async fn initialize(_: std::sync::Arc<()>, _: &sdk::Supervisor) -> Self {
            Self
        }

        async fn run(&mut self, _: &sdk::Supervisor, _: &mut Read) -> Result<(), sdk::Refusal> {
            Ok(())
        }
    }
}

mod second {
    use super::POLICY;

    #[sdk::message]
    #[derive(Clone, PartialEq)]
    #[sdk::operation(id = "duplicated.read", version = 1, description = "Reads.", policy = POLICY, response = ())]
    pub struct Read {}

    pub struct ReadHandler;

    #[sdk::operation]
    impl ReadHandler {
        pub async fn initialize(_: std::sync::Arc<()>, _: &sdk::Supervisor) -> Self {
            Self
        }

        async fn run(&mut self, _: &sdk::Supervisor, _: &mut Read) -> Result<(), sdk::Refusal> {
            Ok(())
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
    restart: sdk::harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Duplicated;

/* === Enums === */
#[sdk::catalog]
enum DuplicatedOperations {
    First(first::ReadHandler),
    Second(second::ReadHandler),
}

/* === Traits === */

/* === Implementations === */
impl sdk::harmos::operations::lifecycle::Policy for Duplicated {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy = POLICY;
}

#[sdk::plugin(id = "duplicated", provides = DuplicatedOperations, requires(), restart = "never")]
impl sdk::Plugin for Duplicated {
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
        &self, table: &mut sdk::harmos::operations::WireTable<sdk::Supervisor>,
    ) -> Result<(), sdk::harmos::operations::ServingError> {
        table.bind_operations::<DuplicatedOperations, _>(self.resources())
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
fn main() {
    // Schema markers link only through a reachable `markers()` call, as the
    // sidecar runtime makes; inspection resolves contracts against them.
    let _ = <Duplicated as sdk::harmos::plugin::Declared>::markers();
}

/* === Tests === */
