/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Delete Plugin Secret Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{Bindings, declarations, key, update};
use crate::{
    Error, Result,
    host::{ClientSettings, Context, storage::administration},
};
use dokime_protocol::host::plugins::{SecretsDeleteRequest, SecretsDeleteResponse};
use harmos_protocol::model::{OperationContract, Position, ReadCut};
use prost::Message;
use std::collections::BTreeSet;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plugins.Secrets/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SecretsDeleteRequest, SecretsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: SecretsDeleteRequest) -> Result<SecretsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let identifiers: BTreeSet<_> = request.identifiers.iter().collect();
        if identifiers.len() != request.identifiers.len() || identifiers.iter().any(|name| name.is_empty()) {
            return Err(Error::Invalid(format!("{ID}: Select Each Declared Name Once")));
        }
        let plugins = context.host.plugins().await?;
        let plugin = plugins
            .iter()
            .find(|plugin| plugin.name == request.name)
            .ok_or_else(|| Error::NotFound(format!("{ID}: Select a Plugin Entry")))?;
        let declared = declarations(plugin)?;
        if request.identifiers.iter().any(|name| {
            !declared.iter().any(|declaration| declaration.metadata.as_ref().is_some_and(|metadata| &metadata.identifier == name))
        }) {
            return Err(Error::Invalid(format!("{ID}: Select Declared Secret Names")));
        }
        // The Expected Revision Is the Native Object's Retained Mutation Position.
        let catalog = if request.expected_revision.is_empty() {
            Bindings::default()
        } else {
            let position = Position::decode(request.expected_revision.as_slice())?;
            let at = ReadCut {
                positions: vec![position],
                revisions: Vec::new(),
            };
            let snapshot = administration::read(context, &key(&request.name), &Bindings::default().encode_to_vec(), Some(at)).await?;
            Bindings::decode(snapshot.payload.as_slice())?
        };
        let bindings = catalog
            .entries
            .into_iter()
            .find(|entry| entry.name == request.name)
            .map(|entry| entry.bindings)
            .unwrap_or_default()
            .into_iter()
            .filter(|binding| !identifiers.contains(&binding.identifier))
            .collect();
        let (_, _, receipt) = update(context, &request.name, bindings, request.expected_revision, &request.request_id).await?;
        Ok(SecretsDeleteResponse { receipt: Some(receipt) })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SecretsDeleteRequest, SecretsDeleteResponse, context, "dokime.host.v1.plugins", "Secrets", "Delete");
