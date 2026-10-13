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
pub use crate::Error;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use prost::Message;
use tonic::Status;
// Local Imports
use dokime_app::Error as HostError;
use dokime_app::plugin::Diagnostic;
use dokime_contract::operations::evidence as v1;

/* === Constants === */
const DETAIL_TYPE_URL: &str = "type.googleapis.com/dokime.v1.CapturedRecordError";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[track_caller]
pub(super) fn detailed(status: Status, error: &HostError) -> Status {
    let diagnostic = Diagnostic::capture(error).redacted();
    let code = status.code();
    let message = status.message().to_owned();
    let envelope = tonic_types::Status {
        code: code as i32,
        message: message.clone(),
        details: vec![prost_types::Any {
            type_url: DETAIL_TYPE_URL.to_owned(),
            value: harmos::Protobuf::encode_to_vec(&captured(&diagnostic)),
        }],
    };

    Status::with_details(code, message, envelope.encode_to_vec().into())
}

/* === Private Functions === */
fn captured(diagnostic: &Diagnostic) -> v1::CapturedRecordError {
    v1::CapturedRecordError {
        message: diagnostic.message.clone(),
        code: None,
        source: (!diagnostic.location.is_empty()).then(|| diagnostic.location.clone()),
        cause_chain: diagnostic
            .chain
            .iter()
            .map(|cause| v1::CapturedErrorCause {
                message: cause.clone(),
                code: None,
                source: None,
            })
            .collect(),
        stacktrace_attachment: None,
        trace_id: None,
        correlation_id: None,
    }
}

/* === Tests === */
