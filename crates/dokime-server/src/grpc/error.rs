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
use dokime::plugin::Diagnostic;
use dokime::runtime::Error as HostError;

/* === Constants === */

const DETAIL_TYPE_URL: &str = "type.googleapis.com/dokime.v1.CapturedRecordError";
// Binary Trailers Expand By 4/3; Leave Room In The Default 8 KiB Header Budget.
const DETAIL_BYTES: usize = 4096;
const DIAGNOSTIC_BYTES: usize = 2048;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[track_caller]
pub(super) fn detailed(status: Status, error: &HostError) -> Status {
    detailed_error(status, error)
}

#[track_caller]
pub(super) fn detailed_error(status: Status, error: &(dyn std::error::Error + 'static)) -> Status {
    let diagnostic = Diagnostic::capture(error).redacted();
    let code = status.code();
    let message = status.message().to_owned();
    let mut envelope = tonic_types::Status {
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

    if envelope.encoded_len() > DETAIL_BYTES {
        envelope.details.clear();
    }
    Status::with_details(code, message, envelope.encode_to_vec().into())
}

/* === Private Functions === */

fn captured(diagnostic: &Diagnostic) -> dokime::domain::model::evidence::wire::CapturedRecordError {
    let mut diagnostics = diagnostic
        .span_trace
        .iter()
        .map(String::as_str)
        .chain(std::iter::once(diagnostic.backtrace.as_str()))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if diagnostics.len() > DIAGNOSTIC_BYTES {
        diagnostics.truncate(diagnostics.floor_char_boundary(DIAGNOSTIC_BYTES));
        diagnostics.push_str("\n[truncated]");
    }
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
        diagnostics: (!diagnostics.is_empty()).then_some(diagnostics),
    }
}

/* === Tests === */

#[test]
fn captured_failure_retains_causes_and_technical_diagnostics() {
    let diagnostic = Diagnostic {
        message: "fixture-operation-failed".to_owned(),
        chain: vec!["fixture-wrapper".to_owned(), "fixture-root".to_owned()],
        location: "fixture.rs:42".to_owned(),
        span_trace: vec!["fixture-span".to_owned()],
        backtrace: "fixture-frame-1\nfixture-frame-2".to_owned(),
    };
    let detail = captured(&diagnostic);
    let bytes = crate::grpc::transcode::Transcode::wire_bytes(&detail).expect("encode captured error");
    assert_eq!(detail.message, diagnostic.message);
    assert_eq!(detail.source.as_deref(), Some(diagnostic.location.as_str()));
    assert_eq!(detail.cause_chain.iter().map(|cause| cause.message.as_str()).collect::<Vec<_>>(), diagnostic.chain);
    let diagnostics = detail.diagnostics.as_deref().expect("technical diagnostics");
    assert_eq!(diagnostics, "fixture-span\nfixture-frame-1\nfixture-frame-2");
    assert!(bytes.windows(diagnostics.len()).any(|window| window == diagnostics.as_bytes()));
    assert!(captured(&Diagnostic::default()).diagnostics.is_none());
}
