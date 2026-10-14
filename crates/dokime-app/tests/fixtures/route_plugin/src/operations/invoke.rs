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
use std::sync::Arc;
use std::sync::atomic::Ordering;
// Third Party Imports

// Local Imports
use crate::{Error, Resources};
use dokime::execution::ExecutionScope;
use dokime::{Refusal, Supervisor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct RunCommandRequest {
    #[row(description = "Which behaviour to exercise: a slow answer, a refusal, or an echo.")]
    pub(crate) command: Option<String>,
}

#[dokime::message]
#[derive(Clone, PartialEq)]
pub(crate) struct RunCommandResponse {
    pub(crate) scoped_context: bool,
    pub(crate) idempotency_key: String,
    pub(crate) command: Option<String>,
}

pub(crate) struct RunCommand {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[dokime::operation(
    id = "route-fixture.run_command",
    version = 1,
    description = "Answers, slowly, with a refusal, or with an echo of the command it was sent.",
    policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_millis(200),
        attempt_timeout: std::time::Duration::from_millis(200),
        retries: dokime::harmos::operations::RetryPolicy::Never,
        restart: dokime::harmos::operations::RestartPolicy::Never,
    }
)]
impl RunCommand {
    pub(crate) async fn initialize(resources: Arc<Resources>, _: &Supervisor) -> Self {
        Self { resources }
    }

    async fn run(
        &mut self,
        ctx: &Supervisor,
        request: &RunCommandRequest,
    ) -> Result<RunCommandResponse, Refusal> {
        let key = ExecutionScope::serving::<Self>(ctx)
            .map(|scope| scope.key().as_str().to_owned())
            .unwrap_or_default();
        if request.command.as_deref() == Some("slow") {
            std::thread::sleep(std::time::Duration::from_millis(600));
        }
        if request.command.as_deref() == Some("refuse") {
            return dokime::plugin::refusal(Err(Error::InvocationRefused));
        }
        self.resources.calls.fetch_add(1, Ordering::Relaxed);
        Ok(RunCommandResponse {
            scoped_context: self.resources.scoped_context,
            idempotency_key: key,
            command: request.command.clone(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
