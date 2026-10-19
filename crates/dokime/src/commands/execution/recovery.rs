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
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_client::{Client, TestExecutionHandle};
use dokime_domain::model::identity::{TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, Default, clap::Args)]
#[group(id = "interrupt_recovery")]
pub(crate) struct Args {
    #[arg(long, requires_all = ["owned_cycle", "recovery_directory"], help = "On Ctrl-C, Stop This Execution And Seal/Export Its Explicitly Owned Cycle.")]
    cancel_on_interrupt: bool,

    #[arg(long, requires_all = ["cancel_on_interrupt", "recovery_directory"], help = "Cycle UUID Created And Exclusively Owned By This Script; Its Pending Executions Will Be Cancelled.")]
    owned_cycle: Option<String>,

    #[arg(long, requires_all = ["cancel_on_interrupt", "owned_cycle"], help = "Existing Directory For Verified execution-<UUID>.zip And cycle-<UUID>.zip; Existing Files Are Refused.")]
    recovery_directory: Option<PathBuf>,
}

pub(crate) struct Plan {
    cycle: TestCycleId,
    execution_output: PathBuf,
    cycle_output: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Args {
    pub(crate) fn prepare(&self, execution: &TestExecutionId) -> crate::Result<Option<Plan>> {
        if !self.cancel_on_interrupt && self.owned_cycle.is_none() && self.recovery_directory.is_none() {
            return Ok(None);
        }
        let (true, Some(cycle), Some(directory)) = (self.cancel_on_interrupt, &self.owned_cycle, &self.recovery_directory) else {
            return Err(crate::Error::Input(
                "interrupt recovery requires --cancel-on-interrupt, --owned-cycle and --recovery-directory".into(),
            ));
        };
        let cycle = TestCycleId::from_uuid(cycle).map_err(|error| crate::Error::Input(error.to_string()))?;
        let execution_name = format!("execution-{execution}.zip");
        let cycle_name = format!("cycle-{cycle}.zip");
        let execution_output = directory.join(execution_name);
        let cycle_output = directory.join(cycle_name);
        dokime_client::validate_download_destination(&execution_output, false)?;
        dokime_client::validate_download_destination(&cycle_output, false)?;
        let plan = Plan { cycle, execution_output, cycle_output };
        Ok(Some(plan))
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
        match execution.checkpoint().await {
            Ok(_) => {}
            Err(dokime_client::Error::Rpc(status)) if status.code() == dokime_client::RpcCode::NotFound => {
                if let Err(error) = execution.stop().await {
                    eprintln!("Execution Stop Was Not Confirmed: {error}. Resolving Through The Owned Cycle Seal.");
                }
            }
            Err(error) => return Err(error.into()),
        }
        let cycle = client.cycle(self.cycle.clone());
        cycle.seal().await?;
        let outcome = execution.wait(None, None).await?;
        let saved = outcome.document.as_ref().and_then(|document| document.execution.as_ref());
        let status = saved.ok_or_else(|| crate::Error::Setup("saved execution outcome is missing after recovery".into()))?.status;
        eprintln!("Execution {} Is Durably Sealed With {status:?}; Saving Collected Evidence", execution.id());

        let saved = execution.export(self.execution_output.clone(), false).await?;
        eprintln!("Execution Evidence Saved To {} (SHA-256 {})", saved.path.display(), saved.sha256);
        let saved = cycle.export(self.cycle_output.clone(), false).await?;
        eprintln!("Cycle Evidence Saved To {} (SHA-256 {})", saved.path.display(), saved.sha256);
        Ok(())
    }
}

/* === Public Functions === */

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
        std::fs::write(&prepared.execution_output, b"previous execution").unwrap();
        assert!(args.prepare(&execution).is_err());
        assert_eq!(std::fs::read(&prepared.execution_output).unwrap(), b"previous execution");

        std::fs::remove_file(&prepared.execution_output).unwrap();
        std::fs::write(&prepared.cycle_output, b"previous cycle").unwrap();
        assert!(args.prepare(&execution).is_err());
        assert_eq!(std::fs::read(&prepared.cycle_output).unwrap(), b"previous cycle");
        assert!(!prepared.execution_output.exists());
    }
}
