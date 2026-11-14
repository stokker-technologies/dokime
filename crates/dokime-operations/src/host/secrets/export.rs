/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Export

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};

use dokime_protocol::host::SecretsExportRequest;
use dokime_protocol::host::SecretsExportResponse;
use dokime_protocol::host::SecretsListRequest;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Export";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SecretsExportRequest, SecretsExportResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SecretsExportRequest) -> Result<SecretsExportResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let catalog = crate::host::secrets::list::handle(context, SecretsListRequest::default()).await?;
        let cut = crate::host::secrets::list::cut(context, "", catalog.generation, request.at.as_ref())?;
        let crate::host::secrets::store::Reply::Archive(archive) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Export(request)).await?
        else {
            return Err(Error::Invalid("Export Secrets: Restart the Secret Store".into()));
        };
        Ok(SecretsExportResponse {
            archive: Some(archive),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
