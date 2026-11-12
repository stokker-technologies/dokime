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

pub const ID: &str = "dokime.planning.materialize_template";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlanningMaterializeTemplateRequest, v1::PreviewSelectionReply>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::PlanningMaterializeTemplateRequest) -> Result<v1::PreviewSelectionReply> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let selection = request.selection.ok_or_else(|| Error::Invalid("MaterializeTemplate Requires a Selection".into()))?;
        if !selection.files.is_empty() {
            return Err(Error::Invalid("MaterializeTemplate Files Are Not Yet Served by the New Host".into()));
        }
        let suites = context.host.suites()?;
        let matches: Vec<_> = suites
            .iter()
            .filter(|(_, suite)| suite.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.suite_id))
            .collect();
        if matches.len() != 1 {
            return Err(Error::Invalid("MaterializeTemplate Requires One Unambiguous Admitted Suite".into()));
        }
        let (plugin, suite) = matches[0];
        if !suite
            .templates
            .iter()
            .any(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.template_id))
        {
            return Err(Error::Invalid("MaterializeTemplate Requires a Template in the Selected Suite".into()));
        }
        let mut combinations = vec![std::collections::HashMap::new()];
        let mut names = std::collections::HashSet::new();
        for parameter in selection.parameters {
            if !names.insert(parameter.key.clone()) {
                return Err(Error::Invalid("MaterializeTemplate Requires Unique Parameter Names".into()));
            }
            let values = parameter.value.ok_or_else(|| Error::Invalid("MaterializeTemplate Requires Parameter Values".into()))?.values;
            if values.is_empty() {
                return Err(Error::Invalid("MaterializeTemplate Requires Nonempty Parameter Values".into()));
            }
            let count = combinations
                .len()
                .checked_mul(values.len())
                .ok_or_else(|| Error::Invalid("MaterializeTemplate Exceeds the Case Bound".into()))?;
            if count > context.settings.max_cases {
                return Err(Error::Invalid("MaterializeTemplate Exceeds the Case Bound".into()));
            }
            let mut expanded = Vec::with_capacity(count);
            for combination in combinations {
                for value in &values {
                    let mut parameters = combination.clone();
                    parameters.insert(parameter.key.clone(), value.clone());
                    expanded.push(parameters);
                }
            }
            combinations = expanded;
        }
        let mut cases = Vec::new();
        for (ordinal, parameters) in combinations.into_iter().enumerate() {
            let origin = context.host.materialize(plugin, &selection.template_id, parameters).await?;
            let metadata = suite
                .templates
                .iter()
                .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == selection.template_id))
                .and_then(|template| template.metadata.clone());
            cases.push(v1::PlannedCase {
                suite_id: selection.suite_id.clone(),
                template_id: selection.template_id.clone(),
                metadata,
                ordinal: ordinal as u64,
                origin: Some(origin),
                ..Default::default()
            });
        }
        Ok(v1::PreviewSelectionReply { cases, ..Default::default() })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
