/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Error Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::predicate::Operator;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Failure {
    pub reason: Reason,
    pub operation: String,
    pub message: String,
    pub field: Option<String>,
    pub operator: Option<Operator>,
    pub snapshot: Option<String>,
    pub limit: Option<EffectiveLimit>,
    pub available_versions: Vec<u32>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveLimit {
    pub name: String,
    pub value: u64,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reason {
    InvalidSelection,
    UnknownField,
    InvalidOperand,
    TypeMismatch,
    InvalidWindow,
    PermissionDenied,
    ScopeNotFound,
    ContractNotFound,
    IncompatibleSchema,
    SnapshotExpired,
    SnapshotUnavailable,
    InvalidContinuation,
    UnsupportedSource,
    WorkLimit,
    ResourceLimit,
    BucketLimit,
    Deadline,
    Corruption,
    StorageUnavailable,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    InvalidArgument,
    PermissionDenied,
    NotFound,
    FailedPrecondition,
    ResourceExhausted,
    DeadlineExceeded,
    DataLoss,
    Unavailable,
}

/* === Traits === */

/* === Implementations === */

impl Failure {
    pub fn new(reason: Reason, operation: impl Into<String>, message: impl Into<String>) -> Box<Self> {
        Box::new(Self {
            reason,
            operation: operation.into(),
            message: message.into(),
            field: None,
            operator: None,
            snapshot: None,
            limit: None,
            available_versions: Vec::new(),
        })
    }

    pub fn status(&self) -> Status {
        match self.reason {
            Reason::PermissionDenied => Status::PermissionDenied,
            Reason::ScopeNotFound | Reason::ContractNotFound => Status::NotFound,
            Reason::IncompatibleSchema | Reason::SnapshotExpired | Reason::SnapshotUnavailable | Reason::UnsupportedSource => {
                Status::FailedPrecondition
            }
            Reason::WorkLimit | Reason::ResourceLimit | Reason::BucketLimit => Status::ResourceExhausted,
            Reason::Deadline => Status::DeadlineExceeded,
            Reason::Corruption => Status::DataLoss,
            Reason::StorageUnavailable => Status::Unavailable,
            _ => Status::InvalidArgument,
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Failure {}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_classification() {
        for (reason, status) in [
            (Reason::UnknownField, Status::InvalidArgument),
            (Reason::ContractNotFound, Status::NotFound),
            (Reason::PermissionDenied, Status::PermissionDenied),
            (Reason::SnapshotUnavailable, Status::FailedPrecondition),
            (Reason::WorkLimit, Status::ResourceExhausted),
            (Reason::Deadline, Status::DeadlineExceeded),
            (Reason::Corruption, Status::DataLoss),
            (Reason::StorageUnavailable, Status::Unavailable),
        ] {
            let error = Failure::new(reason, "test.query", "safe message");
            assert_eq!(error.status(), status);
            let json = serde_json::to_string(&error).unwrap();
            let restored: Failure = serde_json::from_str(&json).unwrap();
            assert_eq!(*error, restored);
        }
    }
}
