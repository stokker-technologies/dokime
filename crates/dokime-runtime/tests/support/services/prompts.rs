/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use dokime_runtime::RuntimeHandle;
use dokime_runtime::{PromptId, Value};

pub async fn assert_late_answer_refused(runtime: &RuntimeHandle, id: &PromptId) {
    let error = runtime
        .submit_prompt_response(
            id.clone(),
            "submit".into(),
            "operator".into(),
            BTreeMap::from([("confirmed".into(), Value::Bool(true))]),
            true,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(
            error,
            dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Conflict { .. })
        ),
        "a late answer is a failed precondition, got {error:?}"
    );
}

pub async fn settlement_records(runtime: &RuntimeHandle, id: &PromptId) -> Vec<(String, Value)> {
    runtime
        .prompt_response_records()
        .await
        .unwrap()
        .into_iter()
        .filter(|record| {
            record.details.get("prompt_id") == Some(&Value::from(id.as_str()))
                && record.details.get("responder") == Some(&Value::from("runtime"))
        })
        .map(|record| {
            let reason = record.details.get("reason").cloned().unwrap_or(Value::Null);
            (record.record_type.as_str().to_owned(), reason)
        })
        .collect()
}
