/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Set Plugin Secret Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::update;
use crate::{
    Result,
    host::{ClientSettings, Context},
};
use dokime_protocol::host::plugins::{SecretsSetRequest, SecretsSetResponse};
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plugins.Secrets/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SecretsSetRequest, SecretsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: SecretsSetRequest) -> Result<SecretsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let (bindings, revision, receipt) =
            update(context, &request.name, request.bindings, request.expected_revision, &request.request_id).await?;
        Ok(SecretsSetResponse {
            bindings,
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
