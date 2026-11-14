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
use harmos_protocol::plugin::InstanceGetResponse;

// Local Imports
use crate::{Error, Result};
use dokime_storage::{Evidence, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Resources {
    pub plugin: Arc<InstanceGetResponse>,
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
    async fn ask(
        &self, execution: &str, question: dokime_protocol::host::PromptsCreateRequest,
    ) -> Result<dokime_protocol::host::PromptsCreateResponse>;
}

/* === Implementations === */

impl Resources {
    pub fn resolve_execution<'a>(&'a self, selected: Option<&'a str>, action: &str) -> Result<&'a str> {
        resolve_execution(Some(&self.execution), selected, action)
    }
    pub fn check_scope(&self, execution: &str) -> Result<()> {
        if self.execution.is_empty() || execution != self.execution {
            return Err(Error::Invalid("Use the Active Execution Scope for This Host Call".into()));
        }
        Ok(())
    }
    pub fn check_action_scope(&self, execution: &str, action: &str) -> Result<()> {
        self.check_scope(execution).map_err(|error| Error::Invalid(format!("{action}: {error}")))?;
        self.store.require_run_open(execution).map_err(|error| Error::Invalid(format!("{action}: {error}")))?;
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

impl Resources {
    pub fn evidence_requested(
        &self, action: &str, request_id: &str, fingerprint: &[u8], kind: &str, payload: Vec<u8>, source_ns: Option<i64>,
    ) -> Result<(u64, harmos_protocol::model::Receipt)> {
        self.check_scope(&self.execution)?;
        let receipt_ns = self.clock.now_ns();
        Ok(self.store.append_requested(
            &self.execution,
            action,
            request_id,
            fingerprint,
            Evidence {
                kind: kind.into(),
                step: self.step.clone(),
                operation: self.operation.clone(),
                attempt: self.attempt,
                source_ns: source_ns.unwrap_or(receipt_ns),
                receipt_ns,
                payload,
                ..Default::default()
            },
        )?)
    }
}

/* === Public Functions === */

/// Resolve a selector against trusted invocation identity before accessing storage.
pub fn resolve_execution<'a>(trusted: Option<&'a str>, selected: Option<&'a str>, action: &str) -> Result<&'a str> {
    let selected = match trusted {
        Some(trusted) if !trusted.is_empty() => {
            if selected.is_some_and(|selected| selected != trusted) {
                return Err(Error::Permission(format!("{action}: Select the Calling Execution")));
            }
            trusted
        }
        Some(_) => return Err(Error::Permission(format!("{action}: Enter an Active Execution"))),
        None => selected.filter(|selected| !selected.is_empty()).ok_or_else(|| Error::Invalid(format!("{action}: Select an Execution")))?,
    };
    Ok(selected)
}

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

#[cfg(test)]
mod tests {
    use super::resolve_execution;
    use crate::Error;

    #[test]
    fn trusted_execution_selectors_cannot_escape_or_claim_client_authority() {
        let action = "dokime.host.v1.executions.Progress/Get";
        assert_eq!(resolve_execution(Some("active"), None, action).unwrap(), "active");
        assert_eq!(resolve_execution(Some("active"), Some("active"), action).unwrap(), "active");
        assert!(matches!(resolve_execution(Some("active"), Some("foreign"), action), Err(Error::Permission(_))));
        assert!(matches!(resolve_execution(Some("active"), Some(""), action), Err(Error::Permission(_))));
        assert!(matches!(resolve_execution(Some(""), None, action), Err(Error::Permission(_))));
        assert_eq!(resolve_execution(None, Some("selected"), action).unwrap(), "selected");
        assert!(matches!(resolve_execution(None, None, action), Err(Error::Invalid(_))));
    }
}
