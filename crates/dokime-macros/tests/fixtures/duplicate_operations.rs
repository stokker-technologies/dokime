/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


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


const POLICY: sdk::harmos::operations::OperationPolicy = sdk::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: sdk::harmos::operations::RetryPolicy::Never,
};


struct Duplicated;

#[sdk::catalog]
enum DuplicatedOperations {
    First(first::Read),
    Second(second::Read),
}


impl sdk::harmos::builtins::lifecycle::Policy for Duplicated {
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
        &self, table: &mut sdk::harmos::operations::WireTable,
    ) -> Result<(), sdk::harmos::operations::ServingError> {
        table.register(DuplicatedOperations::registration(sdk::harmos::Dependencies::new()))
    }

    async fn stop(&mut self, _: &sdk::Supervisor) -> Result<(), sdk::Refusal> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), sdk::Refusal> {
        Ok(())
    }
}


fn main() {
    // Schema markers link only through a reachable `markers()` call, as the
    // sidecar runtime makes; inspection resolves contracts against them.
    std::hint::black_box(<Duplicated as sdk::harmos::plugin::Declared>::markers()).unwrap();
}
