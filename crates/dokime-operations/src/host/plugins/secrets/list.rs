/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Plugin Secret Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::{ReadCut, ResourceRevision};

// Local Imports
use super::{requirements, selected};
use crate::Error;
use crate::{Result, host::Context};
use dokime_protocol::host::plugins::{SecretsListRequest, SecretsListResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plugins.Secrets/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SecretsListRequest) -> Result<SecretsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let name = selected(request.name.as_deref(), ID)?;
        let mut response = SecretsListResponse {
            secrets: requirements(context, name).await?,
            resolved_at: None,
        };
        // The View Is Derived From Declarations and Secret Storage; Its Cut Is the View Itself.
        let cut = ReadCut {
            positions: Vec::new(),
            revisions: vec![ResourceRevision {
                name: format!("{}/plugin-secrets-view/{name}", context.host.identity().backend_id),
                revision: crate::host::paging::bytes(&response),
            }],
        };
        if request.at.as_ref().is_some_and(|at| at != &cut) {
            return Err(Error::Precondition(format!("{ID}: Its Declarations or Secret Storage Have Changed; Resynchronize")));
        }
        response.resolved_at = Some(cut);
        Ok(response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SecretsListRequest, SecretsListResponse, context, "dokime.host.v1.plugins", "Secrets", "List");
