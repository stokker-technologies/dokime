/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Wait For One Durable Execution Outcome

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime_domain::model::execution::wire::ResultStatus;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Execution UUID Whose Saved Seal Is Required.")]
    execution_id: String,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Wait Deadline; Omitted Waits Until Sealed Or Interrupted.")]
    wait_seconds: Option<u64>,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: super::recovery::Args,

    #[arg(long, help = "Emit The Frozen Outcome And Exact Checkpoint As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let recovery = args.recovery.prepare(&id)?;
    let client = args.connection.connect().await?;
    let execution = client.execution(id);
    let timeout = args.wait_seconds.map(Duration::from_secs);
    let result = async {
        if let Some(plan) = &recovery {
            plan.verify(&execution).await?;
        }
        tokio::select! {
            outcome = execution.wait(timeout, None) => outcome.map_err(crate::Error::from),
            interrupted = tokio::signal::ctrl_c() => {
                interrupted?;
                if let Some(plan) = &recovery {
                    plan.recover(&client, &execution).await?;
                    let message = format!("Interrupted execution {}. Collected execution and cycle evidence were saved; no operation was retried.", args.execution_id);
                    return Err(crate::Error::Setup(message));
                }
                let message = format!("Stopped waiting for execution {}. Remote work may continue; reconnect using this identity.", args.execution_id);
                Err(crate::Error::Setup(message))
            }
        }
    }
    .await;
    client.close_and_wait().await;
    let outcome = result?;

    let saved = outcome
        .document
        .as_ref()
        .and_then(|document| document.execution.as_ref())
        .ok_or_else(|| crate::Error::Setup("saved execution outcome is missing".into()))?;
    let status = saved.result.as_ref().ok_or_else(|| crate::Error::Setup("saved execution result is missing".into()))?.status;
    if args.json {
        println!("{}", serde_json::to_string(&outcome)?);
    } else {
        println!("Execution {}: {status:?}", args.execution_id);
    }
    if status != ResultStatus::Passed {
        let error = crate::Error::Setup(format!("execution {} sealed with {status:?}", args.execution_id));
        return Err(crate::Error::Check(Box::new(error)));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
