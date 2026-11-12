/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.documents.update_plot_settings";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::DocumentsUpdatePlotSettingsRequest, v1::ClientEmpty>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::DocumentsUpdatePlotSettingsRequest) -> Result<v1::ClientEmpty> {
    let work = async {
        let scope = request.scope.ok_or_else(|| Error::Invalid("Update Plot Settings: Select Its Execution and Cycle".into()))?;
        if request.document_id.is_empty() || request.plot_id.is_empty() {
            return Err(Error::Invalid("Update Plot Settings: Supply Document and Plot IDs".into()));
        }
        if request.display_points == Some(0) {
            return Err(Error::Invalid("Update Plot Settings: Supply a Positive Display Resolution".into()));
        }
        let technique = v1::Downsample::try_from(request.downsample)
            .map_err(|_| Error::Invalid("Update Plot Settings: Select a Declared Downsampling Technique".into()))?;
        if matches!(technique, v1::Downsample::Raw | v1::Downsample::Lttb) {
            return Err(Error::Invalid("Update Plot Settings: Select Auto or MinMax Until Raw and LTTB Windows Are Connected".into()));
        }
        let (document, _) = context.host.current_document(&scope, &request.document_id)?;
        let mut sections = document.sections.iter().collect::<Vec<_>>();
        let mut found = false;
        while let Some(section) = sections.pop() {
            for card in &section.cards {
                let Some(v1::card::Body::Plot(plot)) = card.body.as_ref() else {
                    continue;
                };
                found |= plot.metadata.as_ref().is_some_and(|metadata| metadata.identifier == request.plot_id);
            }
            sections.extend(section.sections.iter());
        }
        if !found {
            return Err(Error::Invalid("Update Plot Settings: Select a Plot ID from This Document".into()));
        }
        context.host.store().save_plot_preferences(
            &scope.execution_id,
            &request.document_id,
            &request.plot_id,
            request.display_points,
            request.downsample,
        )?;
        Ok(v1::ClientEmpty {})
    };
    super::complete(ID, context.settings.mutation_timeout, work).await
}

/* === Private Functions === */

/* === Tests === */
