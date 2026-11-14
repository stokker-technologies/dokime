/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Notes Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.evidence.Notes/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::evidence::NotesSetRequest, dokime_protocol::client::evidence::NotesSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::evidence::NotesSetRequest,
) -> Result<dokime_protocol::client::evidence::NotesSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Note: Supply Its Whole Writable Value".into()))?;
        if value.message.trim().is_empty() {
            return Err(Error::Invalid("Set Note: Supply a Nonempty Message".into()));
        }
        let reply = context.host.store().set_note(request, "local-client")?;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
