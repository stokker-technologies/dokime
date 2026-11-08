/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Step Failure Story: a cause the host knows becomes the failed step's detail.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;
use dokime_domain::model::execution::termination::TerminationReason;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct RefusedChecks;

struct Undeclared;

#[dokime_runtime::prompt(id = "fixture.refused.setup", label = "Set Up the Fixture")]
struct SetupQuestion;

struct Refused;

/* === Enums === */

#[dokime_runtime::catalog]
enum RefusedTemplates {
    Undeclared(Undeclared),
}

/* === Traits === */

/* === Implementations === */

#[dokime_runtime::suite(id = "fixture.refused.checks", label = "Refused", templates = RefusedTemplates)]
impl RefusedChecks {}

// The template declares no prompts, so the question its step asks is refused by the host.
#[dokime_runtime::template(id = "fixture.refused.checks.undeclared", label = "Undeclared")]
impl Undeclared {
    async fn initialize(_: &mut dokime_runtime::execution::InitializationContext<'_>) -> dokime_runtime::Result<Self> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Ask")]
    async fn ask(&mut self, ctx: &mut dokime_runtime::execution::StepContext<'_>) -> dokime_runtime::Result<()> {
        ctx.ask(SetupQuestion).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl dokime_runtime::runtime::EmbeddedProvider for Refused {
    fn catalog(&self) -> dokime_runtime::InstalledPlugin {
        dokime_runtime::InstalledPlugin {
            source: "fixture_refused".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([RefusedChecks::definition()]),
            ..Default::default()
        }
    }

    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }

    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn dokime_runtime::execution::Runnable>>> {
        Ok(vec![Arc::new(dokime_runtime::execution::Workflow::<Undeclared>::new("fixture.refused.checks", "fixture_refused"))
            as Arc<dyn dokime_runtime::execution::Runnable>])
    }

    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn undeclared_question_fails_its_step() {
    // === Run A Step That Asks A Question Its Template Did Not Declare === //

    let root = tempdir().unwrap();
    let (mut owner, runtime) = started(config(root.path()).register(Arc::new(Refused))).await;
    let suite = RefusedChecks::definition().suite;
    let cases = materialize_cases("fixture.refused.checks", &suite);
    let cycle = start_cases(&runtime, metadata("refused.question", "Refused Question"), cases).await;
    let receipt = runtime.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_run_settled(&runtime, &receipt.command_id).await;

    // === The Step Shows The Host's Cause, Not The Generic Receipt Text === //

    let ended = cycle_snapshot(&runtime, &cycle).await;
    let execution = &ended.test_executions[0];
    assert!(
        matches!(execution.status, ExecutionStatus::Failed | ExecutionStatus::Interrupted),
        "a refused question ends the execution: {:?}",
        execution.status
    );
    assert!(runtime.pending_prompt_projection().await.is_empty(), "nothing reaches the operator");
    let attempt = execution.step_attempts.last().expect("the step attempt is recorded");
    assert_eq!(attempt.termination, TerminationReason::Failed);
    let detail = attempt.detail.as_ref().expect("a failed attempt keeps its detail");
    assert!(detail.message.contains("fixture.refused.setup"), "{detail:?}");
    assert!(detail.message.contains("not declared by this template"), "{detail:?}");
    assert!(detail.message.contains("fixture.refused.checks.undeclared"), "{detail:?}");
    let step = execution.steps.iter().find(|step| step.step_id == attempt.step_id).expect("the step row is presented");
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.detail.as_ref().map(|detail| &detail.message), Some(&detail.message));

    drop(runtime);
    owner.shutdown().await.unwrap();
}
