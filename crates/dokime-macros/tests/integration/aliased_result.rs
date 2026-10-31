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
use sdk::definition::DeclaredTemplate;
use sdk::execution::{CriterionContext, ObserverContext, StepContext};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */
#[derive(Debug)]
struct Error;

struct Probe;

struct Handles;

#[sdk::stream(id = "samples", window = 4096, label = "Samples")]
struct Row {
    value: f64,
}

/* === Enums === */

/* === Traits === */

#[sdk::catalog]
enum SamplesStreams {
    Row(Row),
}

/* === Implementations === */
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("probe failed")
    }
}

impl std::error::Error for Error {}

#[sdk::template(id = "aliased.checks.probe", label = "Probe")]
impl Probe {
    async fn initialize(_ctx: &mut sdk::execution::InitializationContext<'_>) -> Result<(Self, Handles)> {
        let template = Self;
        let shared = Handles;
        Ok((template, shared))
    }

    #[sdk::step(label = "Run")]
    fn run(&mut self, _: &mut StepContext<'_>) -> Result<()> {
        Ok(())
    }

    #[sdk::criterion(label = "Accepted")]
    fn accepted(_: &CriterionContext<'_>) -> Result<bool> {
        Ok(true)
    }

    #[sdk::observer(interval_ms = 10, streams = SamplesStreams)]
    async fn samples(ctx: &mut ObserverContext<'_>) -> Result<()> {
        ctx.publish(Row { value: 1.0 }).await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[test]
fn aliased_result_template() {
    let template = Probe::declaration();
    assert_eq!(template.phases.len(), 1);
    assert_eq!(template.criteria.len(), 1);
}
