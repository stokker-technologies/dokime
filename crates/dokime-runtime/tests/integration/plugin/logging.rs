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
use dokime_runtime::execution::log::Level;
use dokime_runtime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime_runtime::plugin::testing::Harness;

/* === Constants === */
const SUITE: &str = "logging.checks";
const TEMPLATE: &str = "logging.checks.levels";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Checks;

struct Levels {
    samples: usize,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime_runtime::Error),
}

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::catalog]
enum PluginSuites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum ChecksTemplates {
    Levels(Levels),
}

#[dokime_runtime::suite(id = SUITE, label = "Logging", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::template(id = TEMPLATE, label = "Levels")]
impl Levels {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        dokime_runtime::trace!(ctx, "initializing with {} samples", 3);
        Ok(Self { samples: 3 })
    }

    #[dokime_runtime::step(label = "Log at every level")]
    async fn every_level(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let samples = self.samples;
        dokime_runtime::debug!(ctx, "sampled {samples} rows");
        dokime_runtime::info!(ctx, "literal only");
        dokime_runtime::warn!(ctx, "retrying after {}", "timeout",);
        dokime_runtime::error!(ctx, "device {id} answered {status:?}", id = 7, status = Some(500));
        Ok(())
    }

    #[dokime_runtime::criterion(id = "logging.checks.attributed", label = "Attributed")]
    fn attributed(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        dokime_runtime::debug!(ctx, "deciding over {} steps", ctx.steps_reached().len());
        Ok(true)
    }
}

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn macros_format_context() {
    let report = Harness::template(TEMPLATE).run::<PluginSuites>().await.expect("the run completes");
    assert!(report.passed(), "{}", report.diagnostics());

    let logged: Vec<(i32, &str)> = report.log().iter().map(|entry| (entry.level, entry.message.as_str())).collect();

    for expected in [
        (Level::Trace, "initializing with 3 samples"),
        (Level::Debug, "sampled 3 rows"),
        (Level::Info, "literal only"),
        (Level::Warn, "retrying after timeout"),
        (Level::Error, "device 7 answered Some(500)"),
        (Level::Debug, "criterion `logging.checks.attributed`: deciding over 1 steps"),
    ] {
        assert!(logged.contains(&(expected.0 as i32, expected.1)), "expected {expected:?} in the execution log, got {logged:?}");
    }
}

/* === Tests === */
