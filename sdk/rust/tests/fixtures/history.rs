/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Rust Capture Evidence Through Bound Handles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, fs};

// Third Party Imports

// Local Imports
use dokime::{Error, Result, ResultStatus, connect};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the Prepared Host.
    let client = connect().await?;
    let identifier = env::var("DOKIME_TEST_EXECUTION")
        .map_err(|_| Error::Invalid("Read Capture: Supply DOKIME_TEST_EXECUTION From the Shared Story".into()))?;
    let execution = client.executions().get(&identifier).await?;

    // Read the Sealed Outcome and Frozen Typed Records.
    let outcome = execution.seal().await?;
    assert_eq!(outcome.status, ResultStatus::Passed);
    let records = execution.records::<bool>("capture.confirmation.confirmation");
    assert_eq!(records.count().await?, 1);
    assert!(records.last().await?);
    assert_eq!(records.all().await?, [true]);
    let rows = records.rows().await?;
    assert_eq!(rows.len(), 1);
    assert!(rows[0].value);
    assert!(rows[0].timestamp.is_some());

    // Download Both Retained Archives Through Verified Transfers.
    let execution_evidence = execution.evidence().await?;
    let cycle_evidence = execution.cycle.evidence().await?;
    fs::create_dir_all("_output")?;
    fs::write("_output/execution.zip", execution_evidence)?;
    fs::write("_output/cycle.zip", cycle_evidence)?;
    client.close().await?;
    Ok(())
}

/* === Tests === */
