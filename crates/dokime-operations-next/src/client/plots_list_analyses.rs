/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// List Plot Analyses

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.plots.list_analyses";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlotsListAnalysesRequest, v1::ListAnalysesReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PlotsListAnalysesRequest) -> Result<v1::ListAnalysesReply> {
    super::complete(ID, context.settings.read_timeout, async {
        let mut definitions = std::collections::BTreeMap::new();
        for plugin in context.host.plugins()? {
            for section in plugin.description.and_then(|description| description.catalog).into_iter().flat_map(|catalog| catalog.sections) {
                if section.r#type != "dokime.v1.Analyses" {
                    continue;
                }
                use prost::Message;
                for definition in v1::Analyses::decode(section.payload.as_slice())?.analyses {
                    if let Some(metadata) = &definition.metadata {
                        definitions.insert(metadata.identifier.clone(), definition);
                    }
                }
            }
        }
        let limit = super::projection::limit(request.limit, context.settings.page_items);
        let mut definitions = definitions.into_iter().filter(|(id, _)| id > &request.after_id).collect::<Vec<_>>();
        let next_after_id = (definitions.len() > limit).then(|| definitions[limit - 1].0.clone());
        definitions.truncate(limit);
        Ok(v1::ListAnalysesReply {
            definitions: definitions.into_iter().map(|(_, definition)| definition).collect(),
            next_after_id,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
