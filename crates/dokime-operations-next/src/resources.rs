/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use async_trait::async_trait;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;
use dokime_storage::{Evidence, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Resources {
    pub store: Store,
    pub execution: String,
    pub step: String,
    pub operation: String,
    pub attempt: u32,
    pub clock: Arc<dyn Clock>,
    pub prompts: Arc<dyn Prompts>,
}

/* === Enums === */

/* === Traits === */

#[async_trait]
pub trait Clock: Send + Sync {
    fn now_ns(&self) -> i64;
    async fn sleep(&self, duration_ns: i64) -> Result<()>;
}
#[async_trait]
pub trait Prompts: Send + Sync {
    async fn ask(&self, execution: &str, question: v1::OperatorQuestion) -> Result<v1::OperatorAnswer>;
}

/* === Implementations === */

impl Resources {
    pub fn check_scope(&self, execution: &str) -> Result<()> {
        if self.execution.is_empty() || execution != self.execution {
            return Err(Error::Invalid("Use the Active Execution Scope for This Host Call".into()));
        }
        Ok(())
    }
    pub fn evidence(&self, kind: &str, payload: Vec<u8>, source_ns: Option<i64>) -> Result<u64> {
        self.check_scope(&self.execution)?;
        let receipt_ns = self.clock.now_ns();
        Ok(self.store.append(
            &self.execution,
            Evidence {
                kind: kind.into(),
                step: self.step.clone(),
                payload,
                source_ns: source_ns.unwrap_or(receipt_ns),
                receipt_ns,
                operation: self.operation.clone(),
                attempt: self.attempt,
                ..Default::default()
            },
        )?)
    }
}

/* === Public Functions === */

pub fn timestamp_ns(at: &prost_types::Timestamp) -> Result<i64> {
    if !(0..1_000_000_000).contains(&at.nanos) {
        return Err(Error::Invalid("Supply a Valid Protobuf Source Timestamp".into()));
    }
    at.seconds
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(at.nanos as i64))
        .ok_or_else(|| Error::Invalid("Supply a Timestamp Within the Nanosecond Evidence Range".into()))
}

/* === Private Functions === */

/* === Tests === */
