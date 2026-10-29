/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use crate::Error;

use prost::Message;
use tonic::Status;

use dokime::plugin::Diagnostic;
use dokime::runtime::Error as HostError;

const DETAIL_TYPE_URL: &str = "type.googleapis.com/dokime.v1.CapturedRecordError";

#[track_caller]
pub(super) fn detailed(status: Status, error: &HostError) -> Status {
    detailed_error(status, error)
}

#[track_caller]
pub(super) fn detailed_error(status: Status, error: &(dyn std::error::Error + 'static)) -> Status {
    let diagnostic = Diagnostic::capture(error).redacted();
    let code = status.code();
    let message = status.message().to_owned();
    let envelope = tonic_types::Status {
        code: code as i32,
        message: message.clone(),
        // A Detail The Transcoder Refuses Is Omitted; The Status Itself Still Reaches The Caller.
        details: crate::grpc::transcode::Transcode::wire_bytes(&captured(&diagnostic))
            .ok()
            .map(|value| prost_types::Any {
                type_url: DETAIL_TYPE_URL.to_owned(),
                value,
            })
            .into_iter()
            .collect(),
    };

    Status::with_details(code, message, envelope.encode_to_vec().into())
}

fn captured(diagnostic: &Diagnostic) -> dokime::domain::model::evidence::wire::CapturedRecordError {
    dokime::domain::model::evidence::wire::CapturedRecordError {
        message: diagnostic.message.clone(),
        code: None,
        source: (!diagnostic.location.is_empty()).then(|| diagnostic.location.clone()),
        cause_chain: diagnostic
            .chain
            .iter()
            .map(|cause| dokime::domain::model::evidence::wire::CapturedErrorCause {
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
