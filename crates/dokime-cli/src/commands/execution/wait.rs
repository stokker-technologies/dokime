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
use dokime::domain::model::execution::wire::ResultStatus;
use dokime::domain::model::identity::TestExecutionId;

/* === Constants === */

const MAXIMUM_TIMEOUT_MS: u64 = 86_400_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[group(id = "execution_wait")]
pub(crate) struct Args {
    #[arg(help = "Execution UUID Whose Saved Seal Is Required.")]
    execution_id: String,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..), help = "Wait Deadline; Omitted Waits Until Sealed Or Interrupted.")]
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

pub(crate) async fn run(args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    execute(args, None, settings).await
}

pub(super) async fn execute(
    mut args: Args, options: Option<dokime::client::RunOptions>, settings: &dokime::domain::model::settings::application::ClientPreferences,
) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let timeout = args.wait_seconds.map(Duration::from_secs);
    if timeout.is_some_and(|timeout| timeout > Duration::from_millis(MAXIMUM_TIMEOUT_MS)) {
        return Err(crate::Error::Input("wait deadline exceeds the configured maximum".into()));
    }
    let id = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let recovery = args.recovery.prepare(&id)?;
    let client = args.connection.connect().await?;
    let execution = client.execution(id);
    let result = async {
        let observing = async {
            let Some(mut options) = options else {
                return execution.wait(timeout, None).await.map_err(crate::Error::from);
            };
            options.timeout = timeout;
            let mut record_count = 0usize;
            let mut sample_count = 0usize;
            let mut records = |page: &dokime::domain::model::evidence::wire::RecordPage| {
                for record in &page.records {
                    observation("record", record)?;
                    record_count += 1;
                }
                Ok(())
            };
            let mut samples = |page: &dokime::domain::model::telemetry::wire::TelemetrySamplesReply| {
                for sample in &page.samples {
                    observation("sample", sample)?;
                    sample_count += 1;
                }
                Ok(())
            };
            let callbacks = dokime::client::RunCallbacks {
                records: Some(&mut records),
                samples: Some(&mut samples),
            };
            let outcome = execution.run(options, callbacks).await?;
            eprintln!(
                "Drained Evidence: {}",
                serde_json::json!({"execution_id": args.execution_id, "records": record_count, "samples": sample_count})
            );
            Ok::<_, crate::Error>(outcome)
        };
        super::recovery::observe(&client, &execution, recovery.as_ref(), "Saved Outcome", observing).await
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

fn observation(kind: &str, value: &impl serde::Serialize) -> dokime::client::Result<()> {
    use std::io::Write;
    let output = serde_json::json!({"observation": kind, "value": value});
    let result = writeln!(std::io::stderr(), "{output}");
    result.map_err(|error| dokime::client::Error::Rpc(dokime::client::RpcStatus::internal(error.to_string())))
}

/* === Tests === */
