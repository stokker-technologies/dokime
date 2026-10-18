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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[allow(non_camel_case_types)]
struct routeless;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl sdk::harmos::operations::lifecycle::Policy for routeless {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy =
        sdk::harmos::operations::OperationPolicy {
            timeout: std::time::Duration::from_secs(5),
            attempt_timeout: std::time::Duration::from_secs(5),
            retries: sdk::harmos::operations::RetryPolicy::Never,
            restart: sdk::harmos::operations::RestartPolicy::Never,
        };
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy =
        <Self as sdk::harmos::operations::lifecycle::Policy>::INITIALIZATION_POLICY;
}

#[sdk::plugin(id = "routeless", provides = (), requires(), restart = "never", description = "No authored operations, streams or contributions.")]
impl sdk::Plugin for routeless {
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
        _: &mut sdk::harmos::operations::WireTable<sdk::Supervisor>,
    ) -> Result<(), sdk::harmos::operations::ServingError> {
        Ok(())
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
fn main() {}

/* === Tests === */
