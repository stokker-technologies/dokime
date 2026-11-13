/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Concurrent Execution Isolation

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
async fn execution_isolation() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start Three Executions Before Releasing Any Measurement
        let root = super::support::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::execution_fixture()?).await?;
        let first = host.materialize(&entry, "execution.capture", super::support::parameters(11.0, 0, true)).await?;
        let second = host.materialize(&entry, "execution.capture", super::support::parameters(22.0, 0, true)).await?;
        let third = host.materialize(&entry, "execution.capture", super::support::parameters(33.0, 0, true)).await?;
        let cycle = host.start_cycle(vec![first, second]).await?;
        let other = host.start_cycle(vec![third]).await?;
        let executions = [&cycle.executions[0], &cycle.executions[1], &other.executions[0]];
        let mut prompts = Vec::new();
        for execution in executions {
            prompts.push(host.next_prompt(execution).await?);
        }
        assert_ne!(cycle.identifier, other.identifier);
        for (execution, prompt) in executions.into_iter().zip(prompts) {
            super::support::answer(&host, execution, prompt)?;
        }

        // Inspect Isolated Evidence and Execution Resources After Cleanup
        let mut snapshots = cycle.wait().await?.executions;
        snapshots.extend(other.wait().await?.executions);
        for (snapshot, value) in snapshots.iter().zip([11.0, 22.0, 33.0]) {
            assert_eq!(snapshot.execution.termination, v1::Termination::Completed as i32);
            assert!(matches!(snapshot.execution.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
            Some(v1::execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == v1::ResultStatus::Passed as i32));
            assert_eq!(snapshot.logs()?.iter().map(|log| log.message.parse::<f64>()).collect::<Result<Vec<_>, _>>()?, vec![value, value]);
            assert!(!snapshot.samples.is_empty());
            for sample in &snapshot.samples {
                assert_eq!(super::support::Reading::decode(sample.row.as_slice())?.value, value);
            }
            let records = snapshot.records("execution.attempt");
            assert_eq!(records.len(), 1);
            let record = dokime_protocol::context::evidence::RecordsAppendRequest::decode(records[0].payload.as_slice())?;
            let attempt = super::support::Attempt::decode(record.payload.as_slice())?;
            assert_eq!(attempt.value, value);
            assert!(attempt.scope_refused);
            assert!(host.cancel(&snapshot.execution.identifier).is_err());
        }
        host.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
