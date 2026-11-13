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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.plots.Settings/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::plots::SettingsGetRequest, v1::plots::SettingsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::plots::SettingsGetRequest) -> Result<v1::plots::SettingsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Get Plot Settings: Select Its Document Scope".into()))?;
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let (mut reply, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            let (points, downsample) = context
                .host
                .store()
                .plot_preferences(&scope.execution_id, &request.document_id, &request.plot_id)?
                .unwrap_or((None, v1::Downsample::Auto as i32));
            let (_, revision) = context.host.store().administration_value(&key, &[])?;
            Ok(v1::plots::SettingsGetResponse {
                value: Some(v1::plots::PlotSettingsValue { display_points: points, downsample }),
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
