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
impl sdk::harmos::builtins::lifecycle::Policy for routeless {
    const INITIALIZATION_POLICY: sdk::harmos::operations::OperationPolicy = sdk::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: sdk::harmos::operations::RetryPolicy::Never,
    };
    const SHUTDOWN_POLICY: sdk::harmos::operations::OperationPolicy =
        <Self as sdk::harmos::builtins::lifecycle::Policy>::INITIALIZATION_POLICY;
}

#[sdk::plugin(id = "routeless", requires(), restart = "never", description = "No authored operations, streams or contributions.")]
impl sdk::Plugin for routeless {
    const CATALOGS: &'static [sdk::harmos::Catalog] = &[];
}

impl sdk::harmos::Lifecycle for routeless {
    type Config = sdk::plugin::Configuration;
    type Error = sdk::Refusal;
    type Context = sdk::Supervisor;
    async fn prepare(_: sdk::plugin::Configuration, setup: &mut sdk::harmos::Setup<Self>) -> Result<Self, sdk::Refusal> {
        let owner = { Self };
        setup.resource::<()>(owner.resources());
        Ok(owner)
    }
}

impl routeless {
    fn resources(&self) -> std::sync::Arc<()> {
        std::sync::Arc::new(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn main() {
    std::hint::black_box(<routeless as sdk::harmos::plugin::Declared>::markers()).unwrap();
}

/* === Tests === */
