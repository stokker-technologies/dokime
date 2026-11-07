/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Encode Summary

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::domain::model::charger::{EncodedSummary, Supply};
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::Refusal;

/* === Constants === */

const POLICY: OperationPolicy = OperationPolicy {
    timeout: Duration::from_secs(2),
    attempt_timeout: Duration::from_secs(2),
    cancellation_timeout: Duration::ZERO,
    retries: RetryPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "obc.charger.encode_summary", response = EncodedSummary, policy = POLICY)]
#[derive(Clone, PartialEq)]
pub struct EncodeChargerSummary {
    pub supply: Supply,
    pub sample_count: u64,
    pub duration_s: Option<f64>,
    pub prompt_action: String,
    pub operator_note: Option<String>,
    pub accepted: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl EncodeChargerSummary {
    pub fn encode(&self) -> Result<EncodedSummary, serde_json::Error> {
        let mut value = serde_json::json!({ "supply": self.supply, "sample_count": self.sample_count, "prompt_action": self.prompt_action, "accepted": self.accepted });
        if let Some(duration_s) = self.duration_s {
            value["duration_s"] = duration_s.into();
        }
        if let Some(note) = &self.operator_note {
            value["operator_note"] = note.clone().into();
        }
        Ok(EncodedSummary { json: serde_json::to_vec_pretty(&value)? })
    }
}

impl Operation for EncodeChargerSummary {
    type Error = Refusal;
    async fn run(self, _: Context<'_, Self>) -> Result<Response<EncodedSummary>, Refusal> {
        let summary = dokime::plugin::refusal(self.encode())?;
        Ok(Response::new(summary))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
