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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::plots::SettingsGetRequest, dokime_protocol::host::plots::SettingsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::plots::SettingsGetRequest,
) -> Result<dokime_protocol::host::plots::SettingsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Plot Settings: Select Its Document Scope".into()))?;
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let (mut reply, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let (points, downsample) = context
                .host
                .store()
                .plot_preferences(&scope.execution_id, &request.document_id, &request.plot_id)?
                .unwrap_or((None, dokime_protocol::model::Downsample::Auto as i32));
            let (_, revision) = context.host.store().administration_value(&key, &[])?;
            Ok(dokime_protocol::host::plots::SettingsGetResponse {
                value: Some(dokime_protocol::host::plots::PlotSettingsValue { display_points: points, downsample }),
                revision,
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
