/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Registered Execution Plans

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, result};

// Third Party Imports
use harmos_protocol::model::{Position, Receipt};
use harmos_runtime::Identifier;
use harmos_runtime::builtins::storage::{
    CompoundOutcome, DeleteObject, DeleteObjectRequest, Mutate, MutateRequest, MutationId, MutationItem,
};
use harmos_runtime::builtins::storage::{Info, InfoRequest};
use harmos_runtime::storage::ObjectValue;
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::{Context, files::transfer, storage::administration};
use crate::{Error, Result};
use dokime_protocol::host::{CasesCreateResponse, PlannedCase};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct Registration {
    #[prost(bytes = "vec", tag = "1")]
    plan: Vec<u8>,

    #[prost(string, tag = "2")]
    cycle: String,

    #[prost(bytes = "vec", repeated, tag = "3")]
    cases: Vec<Vec<u8>>,

    #[prost(string, tag = "4")]
    window: String,

    #[prost(string, tag = "5")]
    mutation: String,
}

#[derive(Clone, PartialEq, Message)]
struct Case {
    #[prost(string, tag = "1")]
    cycle: String,

    #[prost(bytes = "vec", tag = "2")]
    payload: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn registration(context: &Context, request_id: &str, plan: &[u8]) -> Result<Option<CasesCreateResponse>> {
    let key = registration_key(request_id);
    let (saved, revision) = administration::message::<Registration>(context, &key).await?;
    if revision.is_empty() {
        return Ok(None);
    }
    if saved.plan != plan {
        return Err(Error::Conflict("Register Selection: Choose a New Request ID for the Changed Plan".into()));
    }
    commit(context, request_id, &key, saved).await.map(Some)
}

pub(crate) async fn register(
    context: &Context, request_id: &str, plan: &[u8], cycle: &str, mut reply: CasesCreateResponse,
) -> Result<CasesCreateResponse> {
    if request_id.is_empty() {
        return Err(Error::Invalid("Register Selection: Supply a Request ID".into()));
    }
    let _guard = context.administration.lock().await;
    if let Some(saved) = registration(context, request_id, plan).await? {
        return Ok(saved);
    }
    if !cycle.is_empty() && crate::host::execution_state::cycle(context, cycle).await?.sealed {
        return Err(Error::Precondition("Register Selection: Select an Unsealed Cycle".into()));
    }
    let key = registration_key(request_id);
    let digest = Sha256::digest(request_id.as_bytes());
    let identifier = crate::host::views::hex(&digest);
    for (index, case) in reply.cases.iter_mut().enumerate() {
        case.case_id = format!("case-{identifier}-{index}");
    }
    reply.receipt = None;
    let mutation = transfer::mutation(context, "Register Selection", request_id).await?;
    let saved = Registration {
        plan: plan.into(),
        cycle: cycle.into(),
        cases: reply.cases.iter().map(Message::encode_to_vec).collect(),
        window: mutation.window.to_string(),
        mutation: mutation.key.to_string(),
    };
    commit(context, request_id, &key, saved).await
}

pub(crate) async fn read(context: &Context, identifier: &str, cycle: Option<&str>) -> Result<PlannedCase> {
    let key = format!("case/{identifier}");
    let (case, revision) = administration::message::<Case>(context, &key).await?;
    if revision.is_empty() {
        return Err(Error::NotFound("Read Case: Select a Registered Case".into()));
    }
    if cycle.is_some_and(|cycle| !case.cycle.is_empty() && case.cycle != cycle) {
        return Err(Error::Invalid("Read Case: Select a Case Registered for This Cycle".into()));
    }
    Ok(PlannedCase::decode(case.payload.as_slice())?)
}

pub(crate) async fn list(context: &Context, cycle: &str) -> Result<Vec<PlannedCase>> {
    let (entries, _) = administration::scan(context, "case/", None).await?;
    let mut cases = Vec::new();
    for entry in entries {
        let case = Case::decode(entry.payload.as_slice())?;
        if case.cycle == cycle {
            cases.push(PlannedCase::decode(case.payload.as_slice())?);
        }
    }
    cases.sort_by(|left, right| left.ordinal.cmp(&right.ordinal).then_with(|| left.case_id.cmp(&right.case_id)));
    Ok(cases)
}

pub(crate) async fn delete(context: &Context, cycle: &str, identifier: &str, request_id: &str) -> Result<Receipt> {
    let cases = list(context, cycle).await?;
    if !cases.iter().any(|case| case.case_id == identifier) {
        return Err(Error::NotFound("Delete Case: Select an Unexecuted Case in This Cycle".into()));
    }
    let mut after = String::new();
    loop {
        let page = context.host.store().cycle_execution_page(cycle, &after, NonZeroUsize::new(context.settings.page_items).unwrap())?;
        if page.is_empty() {
            break;
        }
        for execution in page {
            let (case, _, _) = context.host.store().execution_presentation(&execution)?;
            if case == identifier {
                return Err(Error::Precondition("Delete Case: Only Unexecuted Cases Can Be Removed".into()));
            }
            after = execution;
        }
    }
    let value = value(context, format!("case/{identifier}"), Vec::new())?;
    let mutation = transfer::mutation(context, &format!("case/{identifier}/delete"), request_id).await?;
    let response = context
        .storage::<DeleteObject>(DeleteObjectRequest {
            mutation: Some(mutation),
            schema: value.schema.id,
            item: value.item,
            expected: None,
        })
        .await?;
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Delete Case: Refresh the Case Before Deleting It".into()));
    }
    let mut receipt = response.receipt.ok_or_else(|| Error::Precondition("Delete Case: Return Its Durable Receipt".into()))?;
    receipt.request_id = request_id.into();
    Ok(receipt)
}

/* === Private Functions === */

fn registration_key(request_id: &str) -> String {
    let digest = Sha256::digest(request_id.as_bytes());
    format!("case-registration/{}", crate::host::views::hex(&digest))
}

fn value(context: &Context, key: String, payload: Vec<u8>) -> Result<ObjectValue> {
    let bound = NonZeroUsize::new(context.settings.message_bytes)
        .ok_or_else(|| Error::Bound("Register Selection: Configure a Positive Message Bound".into()))?;
    let value = administration::Administration { key, payload };
    administration::catalog().encode(&value, bound).map_err(|source| Error::Invalid(format!("Register Selection: {source}")))
}

async fn commit(context: &Context, request_id: &str, key: &str, saved: Registration) -> Result<CasesCreateResponse> {
    let cases = saved.cases.iter().map(|payload| PlannedCase::decode(payload.as_slice())).collect::<result::Result<Vec<_>, _>>()?;
    let mut reply = CasesCreateResponse { cases, ..Default::default() };
    let mut items = vec![MutationItem::Create(value(context, key.into(), saved.encode_to_vec())?)];
    // Keep the Original Protobuf Bytes: Reencoding Maps Would Change the Native Mutation Fingerprint.
    for (case, payload) in reply.cases.iter().zip(&saved.cases) {
        let stored = Case {
            cycle: saved.cycle.clone(),
            payload: payload.clone(),
        };
        let payload = stored.encode_to_vec();
        items.push(MutationItem::Create(value(context, format!("case/{}", case.case_id), payload)?));
    }
    let mutation = MutationId {
        window: Identifier::new(saved.window).map_err(|source| Error::Invalid(format!("Replay Registration: {source}")))?,
        key: Identifier::new(saved.mutation).map_err(|source| Error::Invalid(format!("Replay Registration: {source}")))?,
    };
    let info = context.storage::<Info>(InfoRequest {}).await?;
    let mut position =
        info.saved.map(Position::from).ok_or_else(|| Error::Precondition("Register Selection: Restore Its Durable History".into()))?;
    position.sequence = u64::MAX;
    reply.receipt = Some(Receipt {
        request_id: request_id.into(),
        accepted_at: Some(position.clone()),
        durable_at: Some(position),
        completed: true,
    });
    if reply.encoded_len() > context.settings.message_bytes {
        return Err(Error::Bound("Register Selection: Reduce the Plan to Fit the Configured Reply Bound".into()));
    }
    let response = context.storage::<Mutate>(MutateRequest { mutation: Some(mutation), items }).await?;
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Register Selection: Refresh the Cases Before Registering Them".into()));
    }
    let mut receipt = response.receipt.ok_or_else(|| Error::Precondition("Register Selection: Return Its Durable Receipt".into()))?;
    receipt.request_id = request_id.into();
    reply.receipt = Some(receipt);
    Ok(reply)
}

/* === Tests === */
