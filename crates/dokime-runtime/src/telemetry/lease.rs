/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::atomic::{AtomicU64, Ordering};
// Third Party Imports
// Third-Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;

/* === Constants === */
static NEXT_LEASE: AtomicU64 = AtomicU64::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl LeaseRequest {
    pub fn new(stream: Identifier) -> Self {
        Self {
            lease_token: format!("{stream}#{}", NEXT_LEASE.fetch_add(1, Ordering::Relaxed)),
            stream,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
