/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::host::BackendAppearanceRequest;
use dokime_protocol::host::BackendAppearanceResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Backend/Appearance";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: BackendAppearanceRequest) -> Result<BackendAppearanceResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let owner = Scope::Root(&context.host.identity().backend_id).identifier()?;
        let (appearance, cut) =
            crate::host::views::read(context, owner, ID, request.at.as_ref(), async { Ok(context.host.appearance()) }).await?;
        Ok(BackendAppearanceResponse {
            appearance: Some(appearance),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(BackendAppearanceRequest, BackendAppearanceResponse, context, "dokime.host.v1", "Backend", "Appearance");
