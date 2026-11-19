/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Run the Starter Suite Through Dokime

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

// Third Party Imports
use dokime::{Result, ResultStatus, connect};

// Local Imports
use starter_project::plugin::{plugin, suites::capture::confirmation::ConfirmationTestTemplate};

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
    // Initialize Client
    let client = connect().await?;
    fs::create_dir_all("_output")?;

    // Initialize & Register Plugin
    client.plugins().register(plugin()?).await?;
    println!("Starter Project");

    // Initialize Test Cycle
    let started = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| dokime::Error::Invalid(error.to_string()))?.as_millis();
    let cycle = client.cycle(format!("Capture {started}")).await?;
    let archive = format!("_output/{started}-{}.zip", cycle.identifier());

    // Initialize Test Cases
    let cases = client.cases().materialize::<ConfirmationTestTemplate>().await?;
    for case in cases {
        // Initialize Test Execution
        let execution = cycle.execution(&case).await?;

        // Run Test Execution
        let mut progress = execution.run();
        while let Some(update) = progress.next().await? {
            println!("{}", update.phase);
        }

        // Seal Test Execution & Report Outcome
        let outcome = execution.seal().await?;
        println!("Passed: {}", outcome.status == ResultStatus::Passed);
    }

    // Seal Test Cycle
    cycle.seal().await?;

    // Save Evidence
    let evidence = cycle.evidence().await?;
    fs::write(&archive, evidence)?;
    println!("Saved {archive}");

    // Close Client
    client.close().await?;
    Ok(())
}

/* === Tests === */
