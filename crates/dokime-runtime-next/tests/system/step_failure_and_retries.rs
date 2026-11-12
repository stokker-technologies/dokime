/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Step Failure and Retry Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_protocol::v1;
use dokime_runtime_next::Host;
use prost::Message;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn step_failure_and_retries() -> Result<(), Box<dyn std::error::Error>> {
    // Retry One Transient Failure and Exhaust Another Operation's Policy
    let root = tempfile::tempdir()?;
    let host = Host::open(root.path(), super::support::settings())?;
    let entry = host.add_managed(&super::support::execution_fixture()?).await?;
    for failures in [1, 2] {
        let origin = host.materialize(&entry, "execution.capture", super::support::parameters(17.0, failures, true)).await?;
        let cycle = host.start_cycle(vec![origin]).await?;
        let execution = &cycle.executions[0];
        let prompt = host.next_prompt(execution).await?;
        super::support::answer(&host, execution, prompt)?;
        let result = cycle.wait().await?;
        let snapshot = &result.executions[0];
        let expected = if failures == 1 { v1::Termination::Completed } else { v1::Termination::Failed };
        assert_eq!(snapshot.execution.termination, expected as i32);
        assert_eq!(snapshot.logs()?.len(), 2);
        assert!(host.streams_closed(&snapshot.execution.identifier)?);
        let attempts = snapshot
            .records("execution.attempt")
            .into_iter()
            .map(|evidence| {
                let record = v1::PublishRecord::decode(evidence.payload.as_slice())?;
                super::support::Attempt::decode(record.payload.as_slice())
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(attempts.iter().map(|attempt| attempt.attempt).collect::<Vec<_>>(), vec![1, 2]);
        let evidence = snapshot
            .evidence
            .iter()
            .filter(|evidence| evidence.step == "measure" && evidence.kind.starts_with("attempt:"))
            .collect::<Vec<_>>();
        assert_eq!(
            evidence.iter().filter(|evidence| evidence.kind == "attempt:start").map(|evidence| evidence.attempt).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(evidence.iter().filter(|evidence| evidence.kind == "attempt:failed").count(), failures as usize);
        assert_eq!(result.counts.evaluated, 1);
        assert_eq!(result.counts.verdict, Some(if failures == 1 { v1::ResultStatus::Passed } else { v1::ResultStatus::Failed }));
        if failures == 1 {
            let reads = snapshot.evidence.iter().filter(|evidence| evidence.kind == "read:statistics").collect::<Vec<_>>();
            assert_eq!(reads.len(), 2);
            let Some(v1::origin::Kind::PluginRun(run)) = snapshot.execution.origin.as_ref().unwrap().kind.as_ref() else { unreachable!() };
            let operation = &run.test_case.as_ref().unwrap().criteria[0].operation;
            assert!(reads.iter().all(|evidence| &evidence.operation == operation));
            let read = dokime_storage::StatisticsRead::decode(reads[0].payload.as_slice())?;
            let repeated = dokime_storage::StatisticsRead::decode(reads[1].payload.as_slice())?;
            assert_eq!(read.positions, repeated.positions);
            assert_eq!(read.response, repeated.response);
            assert_eq!(read.request.as_ref().unwrap().step, "measure");
            let positions = snapshot
                .samples
                .iter()
                .filter(|sample| snapshot.step_contains("measure", sample.position))
                .map(|sample| sample.position)
                .collect::<Vec<_>>();
            assert_eq!(read.positions, positions);
            assert_eq!(read.response.as_ref().unwrap().count, positions.len() as u64);
            assert!(positions.len() < snapshot.samples.len());
            let boundary = snapshot.evidence.iter().find(|evidence| evidence.kind == "step:start" && evidence.step == "measure").unwrap();
            assert!(snapshot.samples.iter().any(|sample| sample.source_ns == boundary.source_ns && sample.position < boundary.position));
        }
    }
    host.shutdown().await?;
    Ok(())
}
