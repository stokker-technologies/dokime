/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Settings Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};

use crate::host::storage::administration;
use dokime_protocol::host::plots::PlotSettingsValue;
use dokime_protocol::host::plots::SettingsGetRequest;
use dokime_protocol::host::plots::SettingsGetResponse;
use dokime_protocol::model::Downsample;
use prost::Message;

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SettingsGetRequest, SettingsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SettingsGetRequest) -> Result<SettingsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Plot Settings: Select Its Document Scope".into()))?;
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let initial = PlotSettingsValue {
            display_points: None,
            downsample: Downsample::Auto as i32,
        }
        .encode_to_vec();
        let saved = administration::read(context, &key, &initial, request.at).await?;
        Ok(SettingsGetResponse {
            value: Some(PlotSettingsValue::decode(saved.payload.as_slice())?),
            revision: saved.revision,
            resolved_at: saved.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SettingsGetRequest, SettingsGetResponse, context, "dokime.host.v1.plots", "Settings", "Get");
