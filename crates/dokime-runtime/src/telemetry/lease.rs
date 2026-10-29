/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;

static NEXT_LEASE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseRequest {
    pub lease_token: String,
    pub stream: Identifier,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseGap {
    pub stream: Identifier,
    pub owner: String,
    pub observed_at: Timestamp,
    pub reason: String,
}

impl LeaseRequest {
    pub fn new(stream: Identifier) -> Self {
        Self {
            lease_token: format!("{stream}#{}", NEXT_LEASE.fetch_add(1, Ordering::Relaxed)),
            stream,
        }
    }
}
