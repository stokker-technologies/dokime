/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Charger Sample Batch

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use super::Chargers;
use crate::domain::model::charger::Sample;
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

#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct Samples {
    pub rows: Vec<Sample>,
}

#[dokime::operation(id = "obc.charger.sample_batch", response = Samples, policy = POLICY, resources = [Chargers])]
#[derive(Clone, PartialEq)]
pub struct SampleChargerBatch {
    pub device_id: u64,
    pub first_index: u64,
    pub count: u64,
    pub interval_s: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl SampleChargerBatch {
    pub const MAX_ROWS: u64 = 256;

    pub fn sample(&self, chargers: &Chargers) -> Result<Samples, crate::Error> {
        if self.count > Self::MAX_ROWS || !self.interval_s.is_finite() || self.interval_s <= 0.0 {
            return Err(crate::Error::Observation("invalid charger sampling batch".into()));
        }
        let end =
            self.first_index.checked_add(self.count).ok_or_else(|| crate::Error::Observation("sample index exceeds capacity".into()))?;
        let mut rows = Vec::new();
        for index in self.first_index..end {
            rows.push(chargers.sample(self.device_id, index as f64 * self.interval_s)?);
        }
        Ok(Samples { rows })
    }
}

impl Operation for SampleChargerBatch {
    type Error = Refusal;
    async fn run(self, ctx: Context<'_, Self>) -> Result<Response<Samples>, Refusal> {
        let rows = dokime::plugin::refusal(self.sample(&ctx.resource::<Chargers>()))?;
        Ok(Response::new(rows))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
