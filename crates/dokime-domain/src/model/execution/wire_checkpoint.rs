/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

// Exact inspected sealed revision and captured evidence in one native history.
// Reading this identity never creates or publishes a newer checkpoint.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub execution_identifier: String,
    pub position: Option<StorageCursor>,

    #[row(description = "Original canonical Harmos ObjectVersionToken bytes.")]
    pub version: Vec<u8>,

    pub evidence_through: Option<StorageCursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageCursor {
    pub history: Option<StoreHistory>,
    pub sequence: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreHistory {
    pub store: String,
    pub epoch: String,
}
