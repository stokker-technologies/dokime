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

/* === Structs === */
struct Probe;

struct Checks;

#[derive(Debug)]
struct Error;

#[sdk::message]
#[derive(Clone, sdk::TelemetryRow)]
struct Row {
    value: f64,
}

#[sdk::stream(id = "samples", window = 4096, row = Row, label = "Samples", description = "One shape, two streams: each of them declares itself and names the shape.")]
struct Samples;

#[sdk::stream(id = "stable.samples", window = 4096, row = Row, label = "Stable Samples")]
struct StableSamples;

/* === Enums === */

/* === Traits === */

#[sdk::catalog]
enum SamplesStreams {
    Samples(Samples),
}

#[sdk::catalog]
enum StableSamplesStreams {
    StableSamples(StableSamples),
}

/* === Implementations === */
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("probe failed")
    }
}

impl std::error::Error for Error {}

#[sdk::catalog]
enum ChecksTemplates {
    Probe(Probe),
}

#[sdk::suite(label = "Checks", templates = ChecksTemplates)]
impl Checks {}

#[sdk::template(label = "Probe")]
impl Probe {
    async fn initialize(_: &mut sdk::execution::InitializationContext<'_>) -> Result<Self, Error> {
        Ok(Self)
    }
    #[sdk::step(label = "Start")]
    fn r#start(&mut self, _: &mut StepContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    #[sdk::step(id = "finished", label = "Finish")]
    fn finish(&mut self, _: &mut StepContext<'_>) -> Result<(), Error> {
        Ok(())
    }

    #[sdk::criterion(label = "Accepted")]
    fn accepted(_: &CriterionContext<'_>) -> Result<bool, Error> {
        Ok(true)
    }

    #[sdk::criterion(id = "stable.acceptance", label = "Stable Acceptance")]
    fn stable(_: &CriterionContext<'_>) -> Result<bool, Error> {
        Ok(true)
    }

    #[sdk::observer(interval_ms = 10, streams = SamplesStreams)]
    async fn samples(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        ctx.stream::<Row>()
            .into::<Samples>()
            .push(Row { value: 1.0 })
            .await;
        Ok(())
    }

    #[sdk::observer(interval_ms = 10, streams = StableSamplesStreams)]
    async fn stable_samples(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
        ctx.stream::<Row>()
            .into::<StableSamples>()
            .push(Row { value: 2.0 })
            .await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

pub(crate) fn item_names_identities() {
    let suite = Checks::definition();
    assert_eq!(suite.suite_id, "Checks");
    assert_eq!(suite.workflow.as_deref(), Some("Checks"));
    let template = Probe::declaration();
    assert_eq!(template.metadata.identifier.as_str(), "Probe");
    assert_eq!(
        Probe::template_id(),
        sdk::core::TemplateId::from_seed("Probe")
    );
    assert_eq!(
        template
            .phases
            .iter()
            .map(|v| v.metadata.identifier.as_str())
            .collect::<Vec<_>>(),
        ["start", "finished"]
    );
    assert_eq!(
        template.phases[0].step_id,
        sdk::core::StepId::from_seed(format!("{}:start", Probe::template_id()))
    );
    assert_eq!(
        template
            .criteria
            .iter()
            .map(|v| v.metadata.identifier.as_str())
            .collect::<Vec<_>>(),
        ["accepted", "stable.acceptance"]
    );
    assert_eq!(
        template.criteria[0].criterion_id,
        sdk::core::CriterionId::from_seed(format!("{}:accepted", Probe::template_id()))
    );
    assert_eq!(
        suite
            .telemetry_contracts
            .iter()
            .map(|v| v.stream.as_str())
            .collect::<Vec<_>>(),
        ["samples", "stable.samples"]
    );
}
