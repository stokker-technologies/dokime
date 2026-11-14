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
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::plots::SettingsSetRequest, dokime_protocol::host::plots::SettingsSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::plots::SettingsSetRequest,
) -> Result<dokime_protocol::host::plots::SettingsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Select Its Document Scope".into()))?;
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Supply the Whole Writable Value".into()))?;
        if value.display_points == Some(0) || dokime_protocol::model::Downsample::try_from(value.downsample).is_err() {
            return Err(Error::Invalid("Set Plot Settings: Select a Positive Resolution and Declared Technique".into()));
        }
        let (document, _) = context.host.current_document(scope, &request.document_id)?;
        let mut sections = document.sections.iter().collect::<Vec<_>>();
        let mut found = false;
        while let Some(section) = sections.pop() {
            for card in &section.cards {
                if let Some(dokime_protocol::model::card::Body::Plot(plot)) = card.body.as_ref() {
                    found |= plot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.plot_id);
                }
            }
            sections.extend(section.sections.iter());
        }
        if !found {
            return Err(Error::Invalid("Set Plot Settings: Select a Plot ID from This Document".into()));
        }
        let fingerprint = crate::host::views::hex(&crate::host::paging::bytes(&request));
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            &key,
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &value.encode_to_vec(),
        )?;
        let value = dokime_protocol::host::plots::PlotSettingsValue::decode(payload.as_slice())?;
        context.host.signal_change();
        Ok(dokime_protocol::host::plots::SettingsSetResponse {
            value: Some(value),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
