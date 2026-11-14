/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Retained Resource Views and Mutation Receipts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1::{PageRequest, ReadCut, Receipt, ResourceRevision, Section};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, client::Context};

use std::future::Future;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn read<T: Message + Default>(
    context: &Context, resource: &str, at: Option<&ReadCut>, current: impl Future<Output = Result<T>>,
) -> Result<(T, ReadCut)> {
    let store = context.host.store();
    let name = format!("{}/{resource}", store.identity());
    if let Some(at) = at {
        if at.revisions.is_empty() && at.positions.len() == 1 {
            let position = store.evidence_position(at.positions.first())?;
            let key = format!("view-at:{name}:{}:{}", position.history.as_ref().unwrap().epoch, position.sequence);
            let saved: Section = store
                .administration(&key)?
                .ok_or_else(|| Error::Precondition(format!("Read {resource}: The Selected View Is Unavailable; Resynchronize")))?;
            return Ok((T::decode(saved.payload.as_slice())?, at.clone()));
        }
        if at.revisions.len() != 1 || at.revisions[0].name != name || at.positions.len() != 1 {
            return Err(Error::Invalid(format!("Read {resource}: Supply Its Returned Cut")));
        }
        store.evidence_position(at.positions.first())?;
        let revision = hex(&at.revisions[0].revision);
        let saved: Section = store
            .administration(&format!("view:{name}:{revision}"))?
            .ok_or_else(|| Error::Precondition(format!("Read {resource}: Retained View Is Unavailable; Resynchronize")))?;
        return Ok((T::decode(saved.payload.as_slice())?, at.clone()));
    }
    let position = store.evidence_position(None)?;
    let value = current.await?;
    let payload = {
        let span = if resource.starts_with("plots/") { tracing::info_span!("plot.encode") } else { tracing::Span::none() };
        let _entered = span.enter();
        value.encode_to_vec()
    };
    let revision = Sha256::digest(&payload).to_vec();
    let saved = Section { r#type: name.clone(), payload };
    store.save_administration(&format!("view:{name}:{}", hex(&revision)), &saved)?;
    let key = format!("view-at:{name}:{}:{}", position.history.as_ref().unwrap().epoch, position.sequence);
    let _ = store.compare_administration(&key, None, &saved)?;

    Ok((
        value,
        ReadCut {
            positions: vec![position],
            revisions: vec![ResourceRevision { name, revision }],
        },
    ))
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn receipt(context: &Context, request_id: &str) -> Result<Receipt> {
    if request_id.is_empty() {
        return Err(Error::Invalid("Mutate Resource: Supply a Nonempty Request ID".into()));
    }
    let position = context.host.store().evidence_position(None)?;
    Ok(Receipt {
        request_id: request_id.into(),
        accepted_at: Some(position.clone()),
        durable_at: Some(position),
        completed: true,
    })
}

pub(super) fn continuation(page: Option<&PageRequest>, at: Option<&ReadCut>) -> Result<()> {
    if page.is_some_and(|page| !page.page_token.is_empty()) && at.is_none() {
        return Err(Error::Invalid("List Resource: Continue with the Returned Cut".into()));
    }
    Ok(())
}

pub(super) fn offset(token: &str, cut: &ReadCut) -> Result<usize> {
    if token.is_empty() {
        return Ok(0);
    }
    let (identity, offset) = token.split_once(':').ok_or_else(|| Error::Invalid("Read Resource: Use the Returned Page Token".into()))?;
    if identity != hex(&Sha256::digest(cut.encode_to_vec())) {
        return Err(Error::Invalid("Read Resource: Continue the Same Selection and Cut".into()));
    }
    offset.parse().map_err(|_| Error::Invalid("Read Resource: Use the Returned Page Token".into()))
}

pub(super) fn token(offset: usize, cut: &ReadCut) -> String {
    format!("{}:{offset}", hex(&Sha256::digest(cut.encode_to_vec())))
}

pub(super) fn page<T: Message>(
    rows: Vec<T>, cut: &ReadCut, page: Option<&PageRequest>, maximum: usize, maximum_bytes: usize,
) -> Result<(Vec<T>, String)> {
    let start = offset(page.map_or("", |page| page.page_token.as_str()), cut)?;
    if start > rows.len() {
        return Err(Error::Invalid("List Resource: Continue Within the Returned View".into()));
    }
    let limit = super::projection::limit(page.map_or(0, |page| page.page_size), maximum);
    let bytes = page.filter(|page| page.max_bytes != 0).map_or(maximum_bytes, |page| (page.max_bytes as usize).min(maximum_bytes));
    // Reserve the Cut, Continuation and Collection Summary Envelope Before Rows.
    let mut retained = cut.encoded_len() + 512;
    let total = rows.len();
    let mut selected = Vec::new();
    for row in rows.into_iter().skip(start).take(limit) {
        let size = row.encoded_len() + 12;
        if retained.saturating_add(size) > bytes {
            if selected.is_empty() {
                return Err(Error::Invalid("List Resource: One Row Exceeds the Requested Byte Bound".into()));
            }
            break;
        }
        retained += size;
        selected.push(row);
    }
    let next = start + selected.len();
    Ok((selected, if next < total { token(next, cut) } else { String::new() }))
}

/* === Private Functions === */

/* === Tests === */
