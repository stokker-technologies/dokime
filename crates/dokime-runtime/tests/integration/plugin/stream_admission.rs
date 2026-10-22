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
use dokime_runtime::execution::{InitializationContext, SidecarTemplate, StepContext, Workflow};
use dokime_runtime::transport::{EvidenceBatch, RunExecution, StepAdmission};

/* === Constants === */

/* === Macros === */
macro_rules! refused_template {
    ($template:ident, $id:literal, $handle:literal, $pending:literal) => {
        struct $template;

        #[dokime_runtime::template(id = $id, label = "Undeclared Stream")]
        impl $template {
            async fn initialize(_ctx: &mut InitializationContext<'_>) -> dokime_runtime::Result<(Self, ())> {
                Ok((Self, ()))
            }

            #[dokime_runtime::step(label = "Publish A Row")]
            async fn publish(&mut self, ctx: &mut StepContext<'_>) -> dokime_runtime::Result<()> {
                if $handle {
                    ctx.stream::<Reading>().open().push(Reading { value: 42.0 }).await;
                } else {
                    ctx.publish(Reading { value: 42.0 }).await;
                }
                if $pending {
                    std::future::pending::<()>().await;
                }
                Ok(())
            }
        }
    };
}

refused_template!(PublishPending, "admission.publish_pending", false, true);

refused_template!(PublishReturn, "admission.publish_return", false, false);

refused_template!(HandlePending, "admission.handle_pending", true, true);

refused_template!(HandleReturn, "admission.handle_return", true, false);

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::stream(id = "admission.reading", window = 4096, label = "Reading")]
struct Reading {
    value: f64,
}

#[dokime_runtime::stream(id = "admission.secondary", window = 4096, row = Reading, label = "Secondary Reading")]
struct Secondary;

struct Declared;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime_runtime::catalog]
enum DeclaredStreams {
    Reading(Reading),
    Secondary(Secondary),
}

#[dokime_runtime::template(
    id = "admission.declared",
    label = "Declared Stream",
    streams = DeclaredStreams
)]
impl Declared {
    async fn initialize(_ctx: &mut InitializationContext<'_>) -> dokime_runtime::Result<(Self, ())> {
        Ok((Self, ()))
    }

    #[dokime_runtime::step(label = "Publish Declared Rows")]
    async fn publish(&mut self, ctx: &mut StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.publish(Reading { value: 1.0 }).await;
        ctx.stream::<Reading>().open().push(Reading { value: 2.0 }).await;
        ctx.stream::<Reading>().into::<Secondary>().push(Reading { value: 3.0 }).await;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn run<T: SidecarTemplate + dokime_runtime::definition::DeclaredTemplate>()
-> (Vec<EvidenceBatch>, Vec<dokime_runtime::telemetry::TelemetrySample>) {
    let engine = Workflow::<T>::new("admission.fixture", "admission-owner");
    let token = dokime_runtime::model::identity::new_id::<dokime_runtime::TestExecutionId>().to_string();
    engine
        .start(RunExecution {
            mode: "automatic".into(),
            execution_token: token.clone(),
            test_cycle_id: dokime_runtime::model::identity::new_id::<dokime_runtime::TestCycleId>().to_string(),
            test_execution_id: token.clone(),
            template_identifier: T::declaration().metadata.identifier.to_string(),
            workflow: "admission.fixture".into(),

            secrets_json: "{}".into(),
            parameters_json: "{}".into(),
        })
        .unwrap();
    let mut cursor = 0;
    let mut frames = Vec::new();
    let result = loop {
        let frame = engine.next(&token, cursor).await.unwrap();
        for encoded in &frame.step_begins {
            let start: dokime_domain::model::execution::result::StepAttemptStart = serde_json::from_str(encoded).unwrap();
            engine.admit(&token, &StepAdmission { attempt_id: start.attempt_id.to_string(), accepted: true, message: String::new() });
        }
        cursor = frame.cursor;
        let complete = frame.outcome.is_some();
        frames.push(frame);
        if complete {
            break frames;
        }
    };
    let samples = engine.take_local_samples(&token).unwrap();
    engine.shutdown().await;
    (
        // A local refusal interrupts even a step pending forever.
        result, samples,
    )
}

async fn assert_refused<T: SidecarTemplate + dokime_runtime::definition::DeclaredTemplate>() {
    let (replies, samples) = run::<T>().await;
    assert!(samples.is_empty(), "no invalid sample enters local evidence");
    let outcome = replies.iter().find_map(|reply| reply.outcome.as_ref()).unwrap();
    assert!(!outcome.passed);
    let diagnostics =
        replies.iter().flat_map(|reply| &reply.diagnostics).map(|entry| entry.message.as_str()).collect::<Vec<_>>().join("\n");
    assert!(diagnostics.contains(T::declaration().metadata.identifier.as_str()), "{diagnostics}");
    assert!(diagnostics.contains("Reading"), "{diagnostics}");
    assert!(diagnostics.contains("`streams` catalog"), "{diagnostics}");
    assert!(diagnostics.contains("admission.reading"), "{diagnostics}");
}

#[tokio::test]
async fn publication_refuses_waits() {
    assert_refused::<PublishPending>().await;
    assert_refused::<PublishReturn>().await;
}

#[tokio::test]
async fn stream_handles_waits() {
    assert_refused::<HandlePending>().await;
    assert_refused::<HandleReturn>().await;
}

#[tokio::test]
async fn declared_stream_publications() {
    let (replies, samples) = run::<Declared>().await;
    assert_eq!(samples.len(), 3);
    assert_eq!(
        samples.iter().map(|sample| sample.stream.as_str()).collect::<Vec<_>>(),
        ["admission.reading", "admission.reading", "admission.secondary"]
    );
    assert!(replies.iter().find_map(|reply| reply.outcome.as_ref()).unwrap().passed);
}

/* === Tests === */
