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
use prost::Message;

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::AnalysesListRequest;
use dokime_protocol::host::AnalysesListResponse;
use dokime_protocol::model::Analyses;

use crate::host::ClientSettings;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Analyses/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<AnalysesListRequest, AnalysesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: AnalysesListRequest) -> Result<AnalysesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, ID, request.at.as_ref(), async {
            let mut definitions = std::collections::BTreeMap::new();
            for plugin in context.host.plugins().await? {
                for section in
                    plugin.description.and_then(|description| description.declarations).into_iter().flat_map(|catalog| catalog.sections)
                {
                    if section.r#type != "dokime.model.v1.Analyses" {
                        continue;
                    }
                    for definition in Analyses::decode(section.payload.as_slice())?.analyses {
                        if let Some(metadata) = &definition.metadata {
                            definitions.insert(metadata.identifier.clone(), definition);
                        }
                    }
                }
            }
            Ok(AnalysesListResponse {
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

crate::native::unary!(AnalysesListRequest, AnalysesListResponse, context, "dokime.host.v1", "Analyses", "List");
