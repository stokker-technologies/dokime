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
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Analyses/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::AnalysesListRequest, dokime_protocol::client::AnalysesListResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::AnalysesListRequest,
) -> Result<dokime_protocol::client::AnalysesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = super::views::read(context, ID, request.at.as_ref(), async {
            let mut definitions = std::collections::BTreeMap::new();
            for plugin in context.host.plugins()? {
                for section in
                    plugin.description.and_then(|description| description.catalog).into_iter().flat_map(|catalog| catalog.sections)
                {
                    if section.r#type != "dokime.v1.Analyses" {
                        continue;
                    }
                    for definition in v1::Analyses::decode(section.payload.as_slice())?.analyses {
                        if let Some(metadata) = &definition.metadata {
                            definitions.insert(metadata.identifier.clone(), definition);
                        }
                    }
                }
            }
            Ok(dokime_protocol::client::AnalysesListResponse {
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
