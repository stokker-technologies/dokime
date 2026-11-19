/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Seal

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::host::{Context, storage::administration};
use crate::{Error, Result};
use dokime_protocol::host::{ExecutionOutcome, ExecutionsSealRequest, ExecutionsSealResponse};
use dokime_storage::persisted::Scope;
use dokime_storage::{ExecutionCut, ExecutionState};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Seal";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsSealRequest) -> Result<ExecutionsSealResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.request_id.trim().is_empty() {
            return Err(Error::Invalid("Seal Execution: Supply a Nonempty Request ID".into()));
        }
        let admitted = context.host.controls(&request.name)?.request_id.is_some();
        let cut = context.host.store().cut(&request.name)?;
        let outcome = sealed_outcome(&cut, admitted)?;
        let key = format!("execution-seal:{}", request.request_id);
        let scope = Scope::Execution(&request.name).identifier()?;
        let (_, receipt) = administration::replace(context, &key, scope, Vec::new(), &request.request_id, request.encode_to_vec()).await?;
        Ok(ExecutionsSealResponse {
            outcome: Some(outcome),
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

fn sealed_outcome(cut: &ExecutionCut, admitted: bool) -> Result<ExecutionOutcome> {
    let name = &cut.execution.identifier;
    if cut.state != ExecutionState::Sealed {
        if !admitted && cut.frontier == 0 {
            return Err(Error::Precondition(format!(
                "Seal Execution {name}: It Has Not Run; Start It Before Requesting Its Sealed Outcome"
            )));
        }
        return Err(Error::Precondition(format!("Seal Execution {name}: Wait Until Its Run Is Sealed, Then Request Its Outcome")));
    }
    cut.execution
        .outcome
        .clone()
        .filter(|outcome| outcome.attribution.is_some())
        .ok_or_else(|| Error::Precondition(format!("Seal Execution {name}: Restore Its Stored Outcome Before Requesting It")))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::sealed_outcome;
    use dokime_protocol::host::execution_outcome::Attribution;
    use dokime_protocol::host::{EvaluatedOutcome, ExecutionOutcome, ResultStatus};
    use dokime_protocol::model::{Origin, PluginRun, Termination, origin};
    use dokime_storage::{Execution, Store};
    use std::error::Error;

    fn execution(identifier: &str) -> Execution {
        Execution {
            identifier: identifier.into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::PluginRun(PluginRun {
                    plugin: "plugin".into(),
                    template: "template".into(),
                    digest: vec![1],
                    started_at: Some(Default::default()),
                    test_case: Some(Default::default()),
                })),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn seal_returns_the_stored_outcome_repeatedly_after_completion() -> Result<(), Box<dyn Error>> {
        // Complete and Automatically Seal an Execution
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&execution("execution"))?;
        let outcome = ExecutionOutcome {
            attribution: Some(Attribution::Evaluated(EvaluatedOutcome {
                status: ResultStatus::Passed as i32,
                reason: String::new(),
            })),
        };
        store.close("execution", Termination::Completed, outcome.clone())?;
        store.seal("execution", Default::default())?;

        // Return the Immutable Outcome Without Mutating Evidence
        let cut = store.cut("execution")?;
        assert_eq!(sealed_outcome(&cut, false)?, outcome);
        assert_eq!(sealed_outcome(&cut, false)?, outcome);
        assert_eq!(store.cut("execution")?, cut);
        Ok(())
    }

    #[test]
    fn seal_refuses_an_admitted_run_without_changing_it() -> Result<(), Box<dyn Error>> {
        // Admit an Execution Before Its First Evidence
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&execution("running"))?;
        let cut = store.cut("running")?;

        // Refuse Without Cancelling or Sealing the Run
        let error = sealed_outcome(&cut, true).unwrap_err();
        assert!(matches!(error, crate::Error::Precondition(_)));
        assert_eq!(store.cut("running")?, cut);
        Ok(())
    }

    #[test]
    fn seal_refuses_a_never_started_execution() -> Result<(), Box<dyn Error>> {
        // Create Evidence Without Admitting a Run
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        store.create(&execution("unstarted"))?;
        let cut = store.cut("unstarted")?;

        // Leave It Open for a Future Start or Cycle Seal
        let error = sealed_outcome(&cut, false).unwrap_err();
        assert!(matches!(error, crate::Error::Precondition(_)));
        assert_eq!(store.cut("unstarted")?, cut);
        Ok(())
    }
}

crate::native::unary!(ExecutionsSealRequest, ExecutionsSealResponse, context, "dokime.host.v1", "Executions", "Seal");
