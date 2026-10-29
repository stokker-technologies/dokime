/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Describe A Concluded Execution When Its Cycle Did Not Complete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::list_file;
use dokime::client::{Client, contracts};

/* === Constants === */

const LOG_TAIL_BYTES: u32 = 16 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// What A Concluded Execution Says About Itself: Each Step And Criterion With Its Detail, Then Its Log Tail.
pub async fn execution(client: &Client, execution_id: &str, log: &str) -> String {
    let mut lines = Vec::new();
    let steps = client
        .call(contracts::execution::ListExecutionSteps {
            execution_id: execution_id.to_owned(),
            ..Default::default()
        })
        .await
        .unwrap()
        .steps;
    for step in steps {
        let label = step.metadata.as_ref().map_or(step.step_id.clone(), |metadata| metadata.label.clone());
        lines.push(format!("step {label}: {:?} {:?}", step.status, step.detail));
        lines.extend(step.criteria.iter().map(criterion));
    }

    let mut content = client.clone();
    let size = list_file(&mut content, execution_id, log).await.size_bytes;
    let request = contracts::content::ReadExecutionFile {
        execution_id: execution_id.to_owned(),
        relative_path: log.to_owned(),
        offset: size.saturating_sub(u64::from(LOG_TAIL_BYTES)),
        max_bytes: LOG_TAIL_BYTES,
    };
    let window = content.call(request).await.unwrap();
    lines.push(String::from_utf8_lossy(&window.data).into_owned());
    lines.join("\n")
}

/* === Private Functions === */

fn criterion(criterion: &dokime::domain::model::execution::wire::StepCriterion) -> String {
    let identifier = criterion.metadata.as_ref().map_or(String::new(), |metadata| metadata.identifier.clone());
    format!("  criterion {identifier}: {:?} {:?} {:?}", criterion.status, criterion.observed, criterion.message)
}

/* === Tests === */
