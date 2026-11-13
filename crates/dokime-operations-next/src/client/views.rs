/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
use harmos_protocol::v1 as h;
use prost::Message;
use sha2::{Digest, Sha256};
use crate::{Error, Result, client::Context};

pub(super) async fn read<T: Message + Default>(context: &Context, resource: &str, at: Option<&h::ReadCut>, current: impl std::future::Future<Output=Result<T>>) -> Result<(T,h::ReadCut)> {
    let store = context.host.store();
    let name = format!("{}/{resource}", store.identity());
    if let Some(at) = at {
        if at.revisions.len()!=1 || at.revisions[0].name != name || at.positions.len()!=1 {
            return Err(Error::Invalid(format!("Read {resource}: Supply Its Returned Cut")));
        }
        store.evidence_position(at.positions.first())?;
        let revision = hex(&at.revisions[0].revision);
        let saved: h::Section = store.administration(&format!("view:{name}:{revision}"))?
            .ok_or_else(|| Error::Precondition(format!("Read {resource}: Retained View Is Unavailable; Resynchronize")))?;
        return Ok((T::decode(saved.payload.as_slice())?,at.clone()));
    }
    let position = store.evidence_position(None)?;
    let value = current.await?;
    let payload = value.encode_to_vec();
    let revision = Sha256::digest(&payload).to_vec();
    let saved = h::Section { r#type: name.clone(), payload };
    store.save_administration(&format!("view:{name}:{}",hex(&revision)), &saved)?;
    Ok((value,h::ReadCut {positions:vec![position],revisions:vec![h::ResourceRevision {name,revision}]}))
}

pub(super) fn hex(bytes:&[u8])->String { bytes.iter().map(|byte|format!("{byte:02x}")).collect() }

pub(super) fn receipt(request_id:&str)->Result<h::Receipt> {
    if request_id.is_empty() { return Err(Error::Invalid("Mutate Resource: Supply a Nonempty Request ID".into())); }
    Ok(h::Receipt {request_id:request_id.into(),completed:true,..Default::default()})
}

pub(super) fn page<T:Message>(rows:Vec<T>,page:Option<&h::PageRequest>,maximum:usize)->Result<(Vec<T>,String)> {
    let start=page.filter(|page|!page.page_token.is_empty()).map(|page|page.page_token.parse::<usize>().map_err(|_|Error::Invalid("List Resource: Use the Returned Page Token".into()))).transpose()?.unwrap_or(0);
    if start>rows.len() { return Err(Error::Invalid("List Resource: Continue Within the Returned View".into())); }
    let limit=super::projection::limit(page.map_or(0,|page|page.page_size),maximum);
    let more=rows.len()>start+limit;
    Ok((rows.into_iter().skip(start).take(limit).collect(),if more {(start+limit).to_string()}else{String::new()}))
}

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
