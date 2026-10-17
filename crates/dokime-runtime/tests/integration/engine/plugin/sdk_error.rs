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
use std::time::Duration;
// Third Party Imports

// Local Imports
use dokime_runtime::execution::{CriterionContext, InitializationContext, StepContext};
use dokime_runtime::plugin::testing::Harness;
use dokime_runtime::prelude::StreamExt;

/* === Constants === */
const SUITE: &str = "sdk_error.checks";
const TEMPLATE: &str = "sdk_error.checks.boundary";
const PROMPT: &str = "sdk_error.checks.checkpoint";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::prompt(id = PROMPT, label = "Continue?")]
struct Checkpoint;

#[dokime_runtime::record(
    id = "sdk_error.checks.observed",
    label = "Observed",
    description = "One declared fact, published by a step and read back by a criterion — `ctx.records::<Observed>()` is the boundary this exercises."
)]
#[derive(Clone, Debug)]
struct Observed {
    value: i64,
}

struct Checks;

struct Boundary {
    note: String,
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
    Boundary(Boundary),
}

#[dokime_runtime::suite(id = SUITE, label = "SDK Error", templates = ChecksTemplates)]
impl Checks {}

#[dokime_runtime::catalog]
enum BoundaryPrompts {
    Checkpoint(Checkpoint),
}

#[dokime_runtime::catalog]
enum BoundaryRecords {
    Observed(Observed),
}

#[dokime_runtime::template(
    id = TEMPLATE,
    label = "Boundary",
    prompts = BoundaryPrompts,
    records = BoundaryRecords)]
impl Boundary {
    async fn initialize(ctx: &mut InitializationContext<'_>) -> Result<Self, Error> {
        let note: String = ctx.parameter("note")?;
        Ok(Self { note })
    }

    #[dokime_runtime::step(
        label = "Read the secret and publish evidence",
        description = "`ctx.secret` — the plugin's own `Sdk` reaches an unavailable or unknown secret exactly the way it reaches an unknown setting."
    )]
    async fn secret(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let token = ctx.secret("token")?;
        ctx.publish(Observed { value: token.expose().len() as i64 }).await;
        ctx.info(format!("note={}", self.note));
        Ok(())
    }

    #[dokime_runtime::step(
        label = "Ask the operator",
        description = "`ctx.ask` — the operator's cancel/refuse split lives one level under `Sdk`, not as a variant of its own."
    )]
    async fn prompt(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let accepted = ctx.ask(Checkpoint).await?;
        ctx.info(format!("accepted={accepted}"));
        Ok(())
    }

    #[dokime_runtime::step(
        label = "Write an empty artifact, then refuse a name that names no place",
        description = "`ctx.artifacts().create` and the writer's `finish` — both answer `dokime_runtime::Error` directly. The empty name after it is the deliberate failure this test's diagnostic assertion reads, so this step runs before `deadline`: one terminal outcome means the run stops here and the bound below is never actually waited out."
    )]
    async fn artifact(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        let writer = ctx.artifacts().create("evidence.txt")?;
        writer.finish()?;
        ctx.artifacts().create("")?;
        Ok(())
    }

    #[dokime_runtime::step(
        label = "Wait for a record nothing delivers",
        description = "`.timeout(d)` answers a bare `Elapsed` per item, so reaching this plugin's `Sdk`-only error takes the same explicit step an author writes for any other type `dokime_runtime::Error` converts from with `?`. Unreached once `artifact` above has already failed the run; kept short in case that ever changes."
    )]
    async fn deadline(&mut self, ctx: &mut StepContext<'_>) -> Result<(), Error> {
        ctx.listen::<Observed>().timeout(Duration::from_millis(50)).next().await.transpose()?;
        Ok(())
    }

    #[dokime_runtime::criterion(id = "sdk_error.checks.observed_present", label = "Observed present")]
    fn observed_present(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
        Ok(!ctx.records::<Observed>()?.is_empty())
    }
}

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn sdk_error_boundary() {
    let report = Harness::template(TEMPLATE)
        .setting("note", "fixture")
        .secret("token", "sensitive-value")
        .answer(PROMPT, "yes", std::iter::empty::<(&str, dokime_runtime::Value)>())
        .run::<PluginSuites>()
        .await
        .expect("the run completes, whether or not the case passed");

    assert!(!report.passed(), "the artifact step deliberately refuses an empty name: {}", report.diagnostics());

    let diagnostics = report.diagnostics();
    assert!(diagnostics.contains("caused by:"), "a chained dokime_runtime::Error renders more than its own message: {diagnostics}");
    assert!(
        diagnostics.contains("must be a non-empty relative"),
        "the innermost cause — the io::Error `Artifacts::create` names — still renders: {diagnostics}"
    );
}

/* === Tests === */
