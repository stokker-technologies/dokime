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

// Third Party Imports

// Local Imports
use dokime_domain::model::query::error::{Failure, Reason};

use dokime_operations::query::ListSamples;

use dokime_runtime::testing::query_failure as status;

use harmos::OperationDeclaration;

use tonic::Code;

use tonic_types::StatusExt;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn failure_mapping_preserves_typed_details_and_redacts_paths() {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap();
    for (reason, code) in [
        (Reason::UnknownField, Code::InvalidArgument),
        (Reason::PermissionDenied, Code::PermissionDenied),
        (Reason::ScopeNotFound, Code::NotFound),
        (Reason::SnapshotUnavailable, Code::FailedPrecondition),
        (Reason::WorkLimit, Code::ResourceExhausted),
        (Reason::Deadline, Code::DeadlineExceeded),
        (Reason::Corruption, Code::DataLoss),
        (Reason::StorageUnavailable, Code::Unavailable),
    ] {
        let mut failure = Failure::new(reason, ListSamples::ID, format!("Cannot read {home}/evidence"));
        failure.field = Some(format!("{home}/field"));
        failure.limit = Some(dokime_domain::model::query::error::EffectiveLimit { name: "rows".into(), value: 2 });
        let error = status(failure);
        assert_eq!(error.code(), code);
        assert!(!error.message().contains(&home));
        let details = error.get_error_details();
        let info = details.error_info().unwrap();
        assert_eq!(info.domain, "dokime.query");
        let decoded: Failure = serde_json::from_str(&info.metadata["query"]).unwrap();
        assert_eq!(decoded.reason, reason);
        assert_eq!(decoded.operation, ListSamples::ID);
        assert_eq!(decoded.limit.unwrap().value, 2);
        assert!(!decoded.field.unwrap().contains(&home));
    }
}
