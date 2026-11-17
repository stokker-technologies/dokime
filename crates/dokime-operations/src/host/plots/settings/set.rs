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
use prost::Message;

// Local Imports
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::plots::PlotSettingsValue;
use dokime_protocol::host::plots::SettingsSetRequest;
use dokime_protocol::host::plots::SettingsSetResponse;
use dokime_protocol::model::Downsample;
use dokime_protocol::model::card;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Settings/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SettingsSetRequest) -> Result<SettingsSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Select Its Document Scope".into()))?;
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Plot Settings: Supply the Whole Writable Value".into()))?;
        if value.display_points == Some(0) || Downsample::try_from(value.downsample).is_err() {
            return Err(Error::Invalid("Set Plot Settings: Select a Positive Resolution and Declared Technique".into()));
        }
        let (document, _) = context.host.current_document(scope, &request.document_id)?;
        let mut sections = document.sections.iter().collect::<Vec<_>>();
        let mut found = false;
        while let Some(section) = sections.pop() {
            for card in &section.cards {
                if let Some(card::Body::Plot(plot)) = card.body.as_ref() {
                    found |= plot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.plot_id);
                }
            }
            sections.extend(section.sections.iter());
        }
        if !found {
            return Err(Error::Invalid("Set Plot Settings: Select a Plot ID from This Document".into()));
        }
        let key = format!("plot-settings:{}/{}/{}", scope.execution_id, request.document_id, request.plot_id);
        let (saved, receipt) = administration::replace(
            context,
            &key,
            Scope::Execution(&scope.execution_id).identifier()?,
            request.expected_revision,
            &request.request_id,
            value.encode_to_vec(),
        )
        .await?;
        let payload = saved.payload;
        let revision = saved.revision;
        let value = PlotSettingsValue::decode(payload.as_slice())?;
        context.host.signal_change();
        Ok(SettingsSetResponse {
            value: Some(value),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SettingsSetRequest, SettingsSetResponse, context, "dokime.host.v1.plots", "Settings", "Set");
