/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Settings Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.plots.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::plots::SettingsSetRequest, dokime_protocol::client::plots::SettingsSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::plots::SettingsSetRequest,
) -> Result<dokime_protocol::client::plots::SettingsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Select Its Document Scope".into()))?;
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Supply the Whole Writable Value".into()))?;
        if value.display_points == Some(0) || v1::Downsample::try_from(value.downsample).is_err() {
            return Err(Error::Invalid("Set Plot Settings: Select a Positive Resolution and Declared Technique".into()));
        }
        let (document, _) = context.host.current_document(scope, &request.document_id)?;
        let mut sections = document.sections.iter().collect::<Vec<_>>();
        let mut found = false;
        while let Some(section) = sections.pop() {
            for card in &section.cards {
                if let Some(v1::card::Body::Plot(plot)) = card.body.as_ref() {
                    found |= plot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.plot_id);
                }
            }
            sections.extend(section.sections.iter());
        }
        if !found {
            return Err(Error::Invalid("Set Plot Settings: Select a Plot ID from This Document".into()));
        }
        let fingerprint = super::views::hex(&super::paging::bytes(&request));
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            &key,
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &value.encode_to_vec(),
        )?;
        let value = dokime_protocol::client::plots::PlotSettingsValue::decode(payload.as_slice())?;
        context.host.signal_change();
        Ok(dokime_protocol::client::plots::SettingsSetResponse {
            value: Some(value),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
