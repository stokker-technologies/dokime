/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Analyses List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Analyses/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::AnalysesListRequest, dokime_protocol::host::AnalysesListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::AnalysesListRequest,
) -> Result<dokime_protocol::host::AnalysesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, ID, request.at.as_ref(), async {
            let mut definitions = std::collections::BTreeMap::new();
            for plugin in context.host.plugins()? {
                for section in
                    plugin.description.and_then(|description| description.declarations).into_iter().flat_map(|catalog| catalog.sections)
                {
                    if section.r#type != "dokime.model.v1.Analyses" {
                        continue;
                    }
                    for definition in dokime_protocol::model::Analyses::decode(section.payload.as_slice())?.analyses {
                        if let Some(metadata) = &definition.metadata {
                            definitions.insert(metadata.identifier.clone(), definition);
                        }
                    }
                }
            }
            Ok(dokime_protocol::host::AnalysesListResponse {
                definitions: definitions.into_values().collect(),
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
