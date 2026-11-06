/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::domain::model::soak::Summary;
use dokime::harmos::operations::{OperationPolicy, RetryPolicy};
use dokime::harmos::{Context, Operation, Response};
use dokime::prelude::*;

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

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct EncodedSummary {
    pub json: Vec<u8>,
}

#[dokime::operation(
    id = "{{crate}}.encode_summary",
    response = EncodedSummary,
    description = "Encode the soak summary as JSON evidence.",
    policy = POLICY
)]
#[derive(Clone, PartialEq)]
pub struct EncodeSummary {
    pub summary: Summary,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for EncodeSummary {
    type Error = Refusal;

    async fn run(self, _: Context<'_, Self>) -> Result<Response<EncodedSummary>, Refusal> {
        let json = dokime::plugin::refusal(serde_json::to_vec_pretty(&self.summary))?;
        Ok(Response::new(EncodedSummary { json }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
