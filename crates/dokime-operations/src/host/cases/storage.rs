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
use std::{collections::HashSet, num::NonZeroUsize, result};

// Third Party Imports
use arrow_array::{Array, BinaryArray, StringArray};
use harmos_operations::storage::{
    ColumnKind, ColumnSelection, ColumnSource, GetObject, GetObjectRequest, ListColumnRows, ListColumnRowsRequest,
};
use harmos_operations::storage::{CompoundOutcome, DeleteScope, DeleteScopeRequest, Mutate, MutateRequest, MutationId, MutationItem};
use harmos_operations::storage::{Info, InfoRequest};
use harmos_protocol::model::{PageRequest, Position, Receipt};
use harmos_query::arrow::ArrowBatch;
use harmos_runtime::storage::{ObjectDefinition, ObjectValue, ReadLimits, StoredObject};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::host::{Context, execution_state, files::transfer, views};
use crate::{Error, Result};
use dokime_protocol::host::{CasesCreateResponse, PlannedCase};
use dokime_storage::persisted::{self, Case, Plan, Scope};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn registration(context: &Context, request_id: &str, plan: &[u8]) -> Result<Option<CasesCreateResponse>> {
    let key = registration_key(request_id);
    let Some(saved) = read_object::<Plan>(context, &key).await? else {
        return Ok(None);
    };
    if saved.plan != plan {
        return Err(Error::Conflict("Register Selection: Choose a New Request ID for the Changed Plan".into()));
    }
    commit(context, request_id, saved).await.map(Some)
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
    if !cycle.is_empty() && execution_state::cycle(context, cycle).await?.sealed {
        return Err(Error::Precondition("Register Selection: Select an Unsealed Cycle".into()));
    }
    let key = registration_key(request_id);
    let digest = Sha256::digest(request_id.as_bytes());
    let identifier = views::hex(&digest);
    for (index, case) in reply.cases.iter_mut().enumerate() {
        case.case_id = format!("case-{identifier}-{index}");
    }
    reply.receipt = None;
    let mutation = transfer::mutation(context, "Register Selection", request_id).await?;
    let scope =
        if cycle.is_empty() { Scope::Root(&context.host.identity().backend_id).identifier()? } else { Scope::Cycle(cycle).identifier()? };
    let saved = Plan {
        key: key.clone(),
        scope,
        plan: plan.into(),
        cases: reply
            .cases
            .iter()
            .map(|case| {
                Ok(Case {
                    key: case.case_id.clone(),
                    scope: Scope::Case(&case.case_id).identifier()?,
                    cycle: cycle.into(),
                    payload: case.encode_to_vec(),
                })
            })
            .collect::<Result<_>>()?,
        window: mutation.window,
        mutation: mutation.key,
    };
    commit(context, request_id, saved).await
}

pub(crate) async fn read(context: &Context, identifier: &str, cycle: Option<&str>) -> Result<PlannedCase> {
    let case = read_object::<Case>(context, identifier).await?.ok_or_else(|| {
        Error::NotFound(format!("Read Case {identifier}: Reload the Cycle and Register Its Template Before Running Again"))
    })?;
    if cycle.is_some_and(|cycle| !case.cycle.is_empty() && case.cycle != cycle) {
        return Err(Error::Invalid("Read Case: Select a Case Registered for This Cycle".into()));
    }
    Ok(PlannedCase::decode(case.payload.as_slice())?)
}

pub(crate) async fn list(context: &Context, cycle: &str) -> Result<Vec<PlannedCase>> {
    let mut bound = HashSet::new();
    let mut after = String::new();
    loop {
        let executions =
            context.host.store().cycle_execution_page(cycle, &after, NonZeroUsize::new(context.settings.page_items).unwrap())?;
        if executions.is_empty() {
            break;
        }
        for execution in executions {
            let (case, _, _) = context.host.store().execution_presentation(&execution)?;
            bound.insert(case);
            after = execution;
        }
    }
    let schema = ObjectDefinition::of::<Case>()
        .schema()
        .map_err(|source| Error::Invalid(format!("List Cases: Restore Their Frozen Schema: {source}")))?;
    let mut at = None;
    let mut token = String::new();
    let mut cases = Vec::new();
    loop {
        let response = context
            .storage::<ListColumnRows>(ListColumnRowsRequest {
                selection: Some(ColumnSelection {
                    source: Some(ColumnSource {
                        kind: ColumnKind::Objects,
                        schema: schema.clone(),
                    }),
                    columns: vec!["payload.cycle".into(), "payload.payload".into()],
                    predicate: None,
                    time: None,
                    start: 0,
                    work: Some(ReadLimits {
                        rows: NonZeroUsize::new(context.settings.page_items).unwrap(),
                        bytes: NonZeroUsize::new(context.settings.message_bytes).unwrap(),
                    }),
                }),
                at,
                page: Some(PageRequest {
                    page_size: context.settings.page_items as u32,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        at = response.resolved_at;
        if at.is_none() {
            return Err(Error::Precondition("List Cases: Return Their Issued Storage Cut".into()));
        }
        if let Some(page) = response.page {
            let batch = ArrowBatch::from_ipc(page.batch.data)
                .map_err(|source| Error::Invalid(format!("List Cases: Restore Their Frozen Values: {source}")))?;
            let batch = batch.as_record_batch();
            let cycles = batch
                .column_by_name("payload.cycle")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Precondition("List Cases: Restore Their Cycle Column".into()))?;
            let payloads = batch
                .column_by_name("payload.payload")
                .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
                .ok_or_else(|| Error::Precondition("List Cases: Restore Their Frozen Payload Column".into()))?;
            for index in 0..cycles.len() {
                if cycles.is_null(index) || payloads.is_null(index) {
                    return Err(Error::Precondition("List Cases: Restore Every Frozen Case Field".into()));
                }
                let case = PlannedCase::decode(payloads.value(index))?;
                if cycles.value(index) == cycle || (cycles.value(index).is_empty() && bound.contains(&case.case_id)) {
                    cases.push(case);
                }
            }
        }
        token = response.next_page_token;
        if token.is_empty() {
            break;
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
    let mutation = transfer::mutation(context, &format!("case/{identifier}/delete"), request_id).await?;
    let mut request = DeleteScopeRequest {
        mutation: Some(mutation),
        scope: Scope::Case(identifier).identifier()?,
    };
    let response = match context.storage::<DeleteScope>(request.clone()).await {
        Err(source) if transfer::expired(&source) => {
            request.mutation = Some(transfer::mutation(context, &format!("case/{identifier}/delete"), request_id).await?);
            context.storage::<DeleteScope>(request).await?
        }
        response => response?,
    };
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Delete Case: Refresh the Case Before Deleting It".into()));
    }
    let mut receipt = response.receipt.ok_or_else(|| Error::Precondition("Delete Case: Return Its Durable Receipt".into()))?;
    receipt.request_id = request_id.into();
    Ok(receipt)
}

/* === Private Functions === */

async fn read_object<T: StoredObject>(context: &Context, key: &str) -> Result<Option<T>> {
    let schema = ObjectDefinition::of::<T>()
        .schema()
        .map_err(|source| Error::Invalid(format!("Read Plan or Case: Restore Its Frozen Schema: {source}")))?;
    let response = context
        .storage::<GetObject>(GetObjectRequest {
            schema: Some(schema),
            item: key.into(),
            at: None,
        })
        .await?;
    let bound = NonZeroUsize::new(context.settings.message_bytes)
        .ok_or_else(|| Error::Bound("Read Plan or Case: Configure a Positive Message Bound".into()))?;
    response
        .value
        .map(|value| persisted::catalog().decode::<T>(&value, bound))
        .transpose()
        .map_err(|source| Error::Invalid(format!("Read Plan or Case: Restore Its Frozen Schema and Payload: {source}")))
}

fn registration_key(request_id: &str) -> String {
    let digest = Sha256::digest(request_id.as_bytes());
    format!("case-registration/{}", views::hex(&digest))
}

fn value<T: StoredObject>(context: &Context, value: &T) -> Result<ObjectValue> {
    let bound = NonZeroUsize::new(context.settings.message_bytes)
        .ok_or_else(|| Error::Bound("Register Selection: Configure a Positive Message Bound".into()))?;
    persisted::catalog().encode(value, bound).map_err(|source| Error::Invalid(format!("Register Selection: {source}")))
}

async fn commit(context: &Context, request_id: &str, mut saved: Plan) -> Result<CasesCreateResponse> {
    let cases = saved.cases.iter().map(|case| PlannedCase::decode(case.payload.as_slice())).collect::<result::Result<Vec<_>, _>>()?;
    let mut reply = CasesCreateResponse { cases, ..Default::default() };
    let mut items = vec![MutationItem::Create(value(context, &saved)?)];
    // Keep the Original Protobuf Bytes: Reencoding Maps Would Change the Native Mutation Fingerprint.
    for case in &saved.cases {
        items.push(MutationItem::Create(value(context, case)?));
    }
    let mutation = MutationId {
        window: saved.window.clone(),
        key: saved.mutation.clone(),
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
    let mut request = MutateRequest { mutation: Some(mutation), items };
    let response = match context.storage::<Mutate>(request.clone()).await {
        Err(source) if transfer::expired(&source) => {
            let mutation = transfer::mutation(context, "Register Selection", request_id).await?;
            saved.window = mutation.window.clone();
            saved.mutation = mutation.key.clone();
            request.items[0] = MutationItem::Create(value(context, &saved)?);
            request.mutation = Some(mutation);
            context.storage::<Mutate>(request).await?
        }
        response => response?,
    };
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Register Selection: Refresh the Cases Before Registering Them".into()));
    }
    let mut receipt = response.receipt.ok_or_else(|| Error::Precondition("Register Selection: Return Its Durable Receipt".into()))?;
    receipt.request_id = request_id.into();
    reply.receipt = Some(receipt);
    Ok(reply)
}

/* === Tests === */
