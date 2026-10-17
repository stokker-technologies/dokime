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
use tempfile::tempdir;
// Local Imports
use super::*;
use dokime::core::{RecordScope, Value};
use dokime::definition::{
    Parameter, ParameterDefault, RustTemplate, Step, TemplateContext,
    authored_criterion as criterion, authored_metadata as metadata, authored_unit as unit,
};
use dokime_app::domains::evidence::RecordSelection;
use dokime_app::domains::execution::authoring::{
    RuntimeResource, SidecarStepContext, StepContextError,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/* === Doc Example: Template Authoring === */

// --8<-- [start:template]
struct TrackingTestTemplate;

struct ToolSession {
    connected: bool,
}

/* === Doc Example: Parameter Spaces And Case Expansion === */

// --8<-- [start:parameter-spaces]
struct MatrixTemplate;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RustTemplate for TrackingTestTemplate {
    fn define(&self, ctx: &mut TemplateContext) {
        ctx.set_metadata(metadata(
            "plt400.ract.tracking",
            "PLT400 RACT Tracking",
            Some("Collect diagnostics while the PLT400 is tracking a prism.".to_owned()),
        ));
        ctx.set_tags(vec!["plt400", "ract", "tracking"]);
        ctx.set_parameters(vec![
            Parameter::text("instrument_ip", "Instrument IP").default("192.168.43.1"),
            Parameter::text("prism", "Prism").default("poa_25"),
            Parameter::number("runtime_minutes", "Runtime Minutes")
                .default(0.0)
                .unit(Some(unit("min"))),
            Parameter::text("operator.name", "Operator Name").default(ParameterDefault::source(
                || std::env::var("USERNAME").ok(),
                "",
            )),
        ]);
        ctx.set_steps(vec![
            Step::new("setup", "Setup"),
            Step::new("collect_tracking", "Collect Tracking"),
        ]);
        ctx.set_criteria(vec![criterion(
            "tracking_samples_captured",
            "Tracking Samples Captured",
            Some(Value::from(true)),
        )]);
    }
}

// --8<-- [end:template]

/* === Doc Example: Runtime Step Bodies === */

// --8<-- [start:steps]
impl TrackingTestTemplate {
    async fn setup(&self, ctx: &mut SidecarStepContext) -> Result<(), StepContextError> {
        let session = open_tool_session(ctx).await?;
        ctx.resources()
            .set_async(RuntimeResource::new(
                "tooling.session",
                "tooling",
                "Tooling Session",
                session,
            ))
            .await?;
        ctx.resources()
            .mark_opened_async("tooling.session", Default::default())
            .await?;
        Ok(())
    }

    async fn collect_tracking(&self, ctx: &mut SidecarStepContext) -> Result<(), StepContextError> {
        ctx.wait_until_stopped_async().await?;
        Ok(())
    }
}

impl RustTemplate for MatrixTemplate {
    fn define(&self, ctx: &mut TemplateContext) {
        ctx.set_metadata(metadata("acme.matrix", "Acme Matrix", None));
        ctx.set_parameters(vec![
            // Scalar: one fixed value in every generated case.
            Parameter::text("operator", "Operator").default("unassigned"),
            // Axis of explicit values via `ParameterDefault::space`. Numeric axes can
            // skip spelling out each value: `ParameterDefault::range(start, stop,
            // step)` (integer), `ParameterDefault::arange`/`linspace` (float).
            Parameter::number("voltage", "Voltage")
                .expansion(dokime::definition::ParameterExpansion::Cartesian)
                .default(ParameterDefault::space([3.3, 5.0]))
                .unit(Some(unit("V"))),
            // A second axis multiplies with the first.
            Parameter::text("mode", "Mode")
                .expansion(dokime::definition::ParameterExpansion::Cartesian)
                .default(ParameterDefault::space(["fast", "slow"])),
        ]);
        ctx.set_steps(vec![Step::new("measure", "Measure")]);
    }
}

/* === Public Functions === */

/* === Private Functions === */
// --8<-- [end:steps]
async fn open_tool_session(_ctx: &SidecarStepContext) -> Result<ToolSession, StepContextError> {
    Ok(ToolSession { connected: true })
}

#[tokio::test(flavor = "multi_thread")]
async fn doc_example_contract() -> Result<(), Box<dyn std::error::Error>> {
    // --8<-- [start:compile]
    let template = TrackingTestTemplate.to_test_case_template()?;
    // --8<-- [end:compile]

    assert_eq!(
        template.metadata.identifier.as_str(),
        "plt400.ract.tracking"
    );
    assert_eq!(template.phases.len(), 2);
    assert_eq!(template.criteria.len(), 1);
    assert_eq!(template.parameters.len(), 4);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn doc_example_context() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempdir()?;
    let (mut runtime, handle) = booted(root.path()).await;
    let scope = RecordScope::default();
    let template = TrackingTestTemplate;

    // --8<-- [start:execution]
    // An execution adapter (a `ExecutionFixture` implementation) receives
    // a runtime handle plus the scope of the scheduled execution, builds the
    // step context, and drives the async step bodies in declaration order.
    let mut ctx = SidecarStepContext::with_source(handle.clone(), scope, "example.tracking");
    template.setup(&mut ctx).await?;
    // --8<-- [end:execution]

    assert!(
        ctx.resources()
            .get::<ToolSession>("tooling.session")
            .is_some_and(|session| session.connected)
    );
    let records = handle
        .selected_records(RecordSelection::default())
        .await
        .unwrap();
    let record_types = records
        .iter()
        .map(|record| record.record_type.as_str())
        .collect::<Vec<_>>();
    assert!(record_types.contains(&"resource.registered"));
    assert!(record_types.contains(&"resource.opened"));

    // --8<-- [start:stop]
    // Stopping the cycle (here: shutting the whole runtime down) makes the
    // cancellation-aware waits return, so `collect_tracking` completes.
    let shutdown = tokio::spawn(async move { runtime.shutdown().await });
    template.collect_tracking(&mut ctx).await?;
    drop(ctx);
    drop(handle);
    shutdown.await??;
    // --8<-- [end:stop]

    Ok(())
}

/* === Tests === */
// --8<-- [end:parameter-spaces]

// --8<-- [start:parameter-spaces-expand]

#[test]
fn doc_example_matrix() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::BTreeMap;

    use dokime::definition::{CaseMaterializationSelection, materialize_template_cases};

    let template = MatrixTemplate.to_test_case_template()?;

    // voltage (2) × mode (2) = 4 cases, one per combination. Expansion is capped
    // at `DEFAULT_MATERIALIZED_CASE_CAP` (1000); a grid past the cap is refused
    // unless the cycle forces it.
    let generated = materialize_template_cases(
        "fixture.registration",
        &template,
        Default::default(),
        Default::default(),
    )?;
    assert_eq!(generated.case_count, 4);
    assert!(!generated.exceeds_default_cap);

    // A fresh admission creates new immutable cases, even for identical inputs.
    let regenerated = materialize_template_cases(
        "fixture.registration",
        &template,
        Default::default(),
        Default::default(),
    )?;
    let case_ids = |output: &dokime::definition::CaseMaterializationOutput| {
        output
            .test_cases
            .iter()
            .map(|case| case.test_case_id.clone())
            .collect::<Vec<_>>()
    };
    assert_ne!(case_ids(&generated), case_ids(&regenerated));

    // Pinning an axis via `case_overrides` (what an operator-supplied global
    // parameter selection does) slices the grid: 2 × 1 = 2. `case_filter`
    // is the softer form — it keeps only the listed values of an axis.
    let selection = CaseMaterializationSelection {
        case_overrides: BTreeMap::from([("mode".to_owned(), vec!["fast".into()])]),
        ..CaseMaterializationSelection::default()
    };
    let sliced = materialize_template_cases(
        "fixture.registration",
        &template,
        selection,
        Default::default(),
    )?;
    assert_eq!(sliced.case_count, 2);

    Ok(())
}

// --8<-- [end:parameter-spaces-expand]
