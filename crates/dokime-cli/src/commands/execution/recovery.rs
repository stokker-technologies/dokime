/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Save Explicitly Owned Evidence After An Interrupt

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    future::{Future, poll_fn},
    path::PathBuf,
};

// Third Party Imports

// Local Imports
use dokime::client::{Client, TestExecutionHandle};
use dokime::domain::model::identity::{TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Default, clap::Args)]
#[group(id = "interrupt_recovery")]
pub(crate) struct Args {
    #[arg(long, requires_all = ["owned_cycle", "recovery_directory"], help = "On Ctrl-C, Stop The Selected Work And Seal/Export Its Explicitly Owned Cycle.")]
    cancel_on_interrupt: bool,

    #[arg(long, requires_all = ["cancel_on_interrupt", "recovery_directory"], help = "Cycle UUID Created And Exclusively Owned By This Script; Its Pending Executions Will Be Cancelled.")]
    owned_cycle: Option<String>,

    #[arg(long, requires_all = ["cancel_on_interrupt", "owned_cycle"], help = "Existing Directory For Verified execution-<UUID>.zip And cycle-<UUID>.zip; Existing Files Are Refused.")]
    recovery_directory: Option<PathBuf>,
}

pub(crate) struct Plan {
    cycle: TestCycleId,
    executions: Vec<(TestExecutionId, PathBuf)>,
    cycle_output: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Args {
    pub(crate) fn prepare(&self, execution: &TestExecutionId) -> crate::Result<Option<Plan>> {
        self.prepare_many(std::slice::from_ref(execution))
    }

    pub(crate) fn prepare_append(&self, cycle: &TestCycleId, executions: &[TestExecutionId]) -> crate::Result<Option<Plan>> {
        let plan = self.prepare_many(executions)?;
        if let Some(plan) = &plan
            && plan.cycle != *cycle
        {
            let message = "append recovery requires its target cycle to equal --owned-cycle".to_owned();
            return Err(crate::Error::Input(message));
        }
        Ok(plan)
    }

    fn prepare_many(&self, selected: &[TestExecutionId]) -> crate::Result<Option<Plan>> {
        if !self.cancel_on_interrupt && self.owned_cycle.is_none() && self.recovery_directory.is_none() {
            return Ok(None);
        }
        let (true, Some(cycle), Some(directory)) = (self.cancel_on_interrupt, &self.owned_cycle, &self.recovery_directory) else {
            let message = "interrupt recovery requires --cancel-on-interrupt, --owned-cycle and --recovery-directory".to_owned();
            return Err(crate::Error::Input(message));
        };
        let cycle = TestCycleId::from_uuid(cycle).map_err(|error| {
            let message = error.to_string();
            crate::Error::Input(message)
        })?;

        let mut executions = Vec::with_capacity(selected.len());
        let mut seen = std::collections::HashSet::new();
        for execution in selected {
            if !seen.insert(execution) {
                let message = "recovery repeats an execution identity".to_owned();
                return Err(crate::Error::Input(message));
            }
            let output = directory.join(format!("execution-{execution}.zip"));
            dokime::client::validate_download_destination(&output, false)?;
            executions.push((execution.clone(), output));
        }

        let cycle_output = directory.join(format!("cycle-{cycle}.zip"));
        dokime::client::validate_download_destination(&cycle_output, false)?;
        Ok(Some(Plan { cycle, executions, cycle_output }))
    }
}

impl Plan {
    pub(crate) async fn verify(&self, execution: &TestExecutionHandle) -> crate::Result<()> {
        let detail = execution.snapshot().await?;
        if detail.cycle_id != self.cycle.as_str() {
            return Err(crate::Error::Input(format!(
                "execution {} belongs to cycle {}, not explicitly owned cycle {}; no cancellation was requested",
                execution.id(),
                detail.cycle_id,
                self.cycle
            )));
        }
        Ok(())
    }

    pub(crate) async fn recover(&self, client: &Client, execution: &TestExecutionHandle) -> crate::Result<()> {
        self.verify(execution).await?;
        eprintln!("Cancellation Requested For Execution {}. Recovery Cycle: {}", execution.id(), self.cycle);

        // A Stop Reply May Be Lost, Or Admission May Still Be Settling.
        // The Owned Cycle's Durable Seal Closes Admission And Settles Both Cases.
        let stop_failure = match execution.checkpoint().await {
            Ok(_) => None,
            Err(dokime::client::Error::Rpc(status)) if status.code() == dokime::client::RpcCode::NotFound => execution.stop().await.err(),
            Err(error) => return Err(error.into()),
        };
        if let Some(error) = stop_failure {
            eprintln!("Execution Stop Was Not Confirmed: {error}. Resolving Through The Owned Cycle Seal.");
        }
        self.recover_append(client).await
    }

    pub(crate) async fn recover_append(&self, client: &Client) -> crate::Result<()> {
        let cycle = client.cycle(self.cycle.clone());
        // Seal First: An Uncertain Append Must Not Keep Admission Open.
        eprintln!("Cancellation Requested For Owned Cycle {}. Sealing Before Reconciliation.", self.cycle);
        cycle.seal().await?;
        let members = cycle.executions().await?;
        let mut failure = None;
        for (id, output) in &self.executions {
            let matches_id = |member: &&TestExecutionHandle| member.id() == id;
            let Some(execution) = members.iter().find(matches_id) else {
                eprintln!("Execution {id} Was Not Registered In Owned Cycle {}; No Mutation Was Retried.", self.cycle);
                continue;
            };
            let result = async {
                execution.wait(None, None).await?;
                let saved = execution.export(output.clone(), false).await?;
                eprintln!("Execution Evidence Saved To {} (SHA-256 {})", saved.path.display(), saved.sha256);
                Ok::<_, crate::Error>(())
            }
            .await;
            let error = match result {
                Ok(()) => continue,
                Err(error) => error,
            };
            eprintln!("Execution {id} Evidence Recovery Failed: {error}");
            failure.get_or_insert(error);
        }
        match cycle.export(self.cycle_output.clone(), false).await {
            Ok(saved) => eprintln!("Cycle Evidence Saved To {} (SHA-256 {})", saved.path.display(), saved.sha256),
            Err(error) => {
                failure.get_or_insert(error.into());
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

/* === Public Functions === */

pub(crate) async fn observe<T>(
    client: &Client, execution: &TestExecutionHandle, recovery: Option<&Plan>, subject: &str,
    operation: impl Future<Output = crate::Result<T>>,
) -> crate::Result<T> {
    let work = async {
        if let Some(plan) = recovery {
            plan.verify(execution).await?;
        }
        operation.await
    };
    tokio::select! {
        biased;
        signal = interrupted(execution.id().as_str(), subject, recovery.is_some()) => {
            signal?;
            eprintln!("Cancellation Requested For {subject} In Execution {}.", execution.id());
            if let Some(plan) = recovery {
                plan.recover(client, execution).await?;
                return Err(crate::Error::Setup(format!("Interrupted {subject} in execution {}. Collected execution and cycle evidence were saved; no operation was retried.", execution.id())));
            }
            Err(crate::Error::Setup(format!("Interrupted {subject} in execution {}. Remote work may continue; reconnect using this identity.", execution.id())))
        }
        outcome = work => outcome,
    }
}

pub(crate) async fn interrupted(identity: &str, subject: &str, recovery: bool) -> std::io::Result<()> {
    let interrupt = tokio::signal::ctrl_c();
    tokio::pin!(interrupt);
    let mut announced = false;
    poll_fn(|context| {
        let outcome = interrupt.as_mut().poll(context);
        if outcome.is_pending() && !announced {
            let action = if recovery { "Stop It And Save Evidence" } else { "Stop Waiting" };
            eprintln!("Waiting For {subject}: {identity}. Press Ctrl+C To {action}.");
            announced = true;
        }
        outcome
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflicts_are_refused_before_connection_or_mutation() {
        let directory = tempfile::tempdir().unwrap();
        let execution = TestExecutionId::from_uuid("00000000-0000-0000-0000-000000000001").unwrap();
        let cycle = "00000000-0000-0000-0000-000000000002";
        let args =
            Args { cancel_on_interrupt: true, owned_cycle: Some(cycle.into()), recovery_directory: Some(directory.path().to_owned()) };
        let prepared = args.prepare(&execution).unwrap().unwrap();
        std::fs::write(&prepared.executions[0].1, b"previous execution").unwrap();
        assert!(args.prepare(&execution).is_err());
        assert_eq!(std::fs::read(&prepared.executions[0].1).unwrap(), b"previous execution");

        std::fs::remove_file(&prepared.executions[0].1).unwrap();
        std::fs::write(&prepared.cycle_output, b"previous cycle").unwrap();
        assert!(args.prepare(&execution).is_err());
        assert_eq!(std::fs::read(&prepared.cycle_output).unwrap(), b"previous cycle");
        assert!(!prepared.executions[0].1.exists());
    }

    #[test]
    fn append_recovery_checks_every_requested_destination_and_cycle_owner() {
        let directory = tempfile::tempdir().unwrap();
        let cycle = TestCycleId::from_seed("owned append");
        let first = TestExecutionId::from_seed("first append");
        let second = TestExecutionId::from_seed("second append");
        let args =
            Args { cancel_on_interrupt: true, owned_cycle: Some(cycle.to_string()), recovery_directory: Some(directory.path().to_owned()) };
        let executions = [first.clone(), second];
        let plan = args.prepare_append(&cycle, &executions).unwrap().unwrap();
        assert_eq!(plan.executions.len(), 2);
        assert!(args.prepare_append(&TestCycleId::from_seed("foreign"), &executions).is_err());
        assert!(args.prepare_append(&cycle, &[first.clone(), first]).is_err());

        std::fs::write(&plan.executions[1].1, b"previous second execution").unwrap();
        assert!(args.prepare_append(&cycle, &executions).is_err());
        assert_eq!(std::fs::read(&plan.executions[1].1).unwrap(), b"previous second execution");
        assert!(!plan.executions[0].1.exists());
        assert!(!plan.cycle_output.exists());
    }
}
