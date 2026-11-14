/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};

use dokime_protocol::host::SecretWritable;
use dokime_protocol::host::SecretsSetRequest;
use dokime_protocol::host::SecretsSetResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Set";

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
        let receipt = crate::host::views::receipt(context, &request.request_id)?;
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Put(request.clone())).await?
        else {
            return Err(Error::Invalid("Set Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(SecretsSetResponse {
            value: Some(SecretWritable {
                metadata: request.value.and_then(|value| value.metadata),
                value: None,
            }),
            revision: snapshot.generation.to_be_bytes().to_vec(),
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
