/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Construct Canonical Selection Plans From Flags

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_domain::operations::planning::PreviewSelectionReply;
use dokime_domain::operations::planning::RegisteredSelection;
use dokime_domain::operations::planning::SelectionPlan;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[group(id = "plan_input")]
pub(crate) struct Args {
    #[arg(long, conflicts_with_all = ["template", "case_ids", "selections", "max_cases"], help = "Read The Canonical Request JSON Without Flag Overrides.")]
    pub(crate) input_file: Option<PathBuf>,
    #[arg(
        long,
        conflicts_with = "case_ids",
        help = "Template ID As `<Suite ID>.<Segment>`; Omitted Parameters Use Backend Defaults."
    )]
    template: Option<String>,
    #[arg(
        long = "case-id",
        conflicts_with = "selections",
        help = "Registered Case ID; Repeat To Preserve Order And Repeated Attempts."
    )]
    case_ids: Vec<String>,
    #[arg(
        long = "selection-json",
        help = "RegisteredSelection JSON Naming Its Template ID; Repeat For Ordered Selections."
    )]
    selections: Vec<String>,
    #[arg(
        long,
        default_value_t = 0,
        help = "Case Limit; Zero Uses The Backend Limit."
    )]
    max_cases: u32,
}

// A plan read offline. Its selections name templates by their readable ID,
// which the catalog turns into the registered selection the backend plans.
pub(crate) struct Planned {
    plan: Option<SelectionPlan>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Args {
    pub(crate) fn plan(self) -> crate::Result<Planned> {
        let mut selections = self
            .selections
            .into_iter()
            .map(|json| {
                serde_json::from_str::<RegisteredSelection>(&json).map_err(|error| {
                    crate::Error::Input(format!("invalid selection input: {error}"))
                })
            })
            .collect::<crate::Result<Vec<_>>>()?;
        if let Some(template) = self.template {
            selections.push(RegisteredSelection {
                suite_id: dokime_client::template_suite(&template)
                    .unwrap_or_default()
                    .to_owned(),
                template_id: template,
                parameters: Vec::new(),
                files: Vec::new(),
            });
        }
        if selections.is_empty() && self.case_ids.is_empty() {
            return Err(crate::Error::Input(
                "a request file, template, selection or case ID is required".into(),
            ));
        }
        Planned::canonical(Some(SelectionPlan {
            selections,
            max_cases: self.max_cases,
            case_ids: self.case_ids,
            cycle_id: String::new(),
        }))
    }
}

impl Planned {
    // Everything checkable without a backend is refused before connecting.
    pub(crate) fn canonical(plan: Option<SelectionPlan>) -> crate::Result<Self> {
        if let Some(plan) = &plan {
            for id in &plan.case_ids {
                dokime_domain::model::identity::CaseId::from_wire(id)
                    .map_err(|error| crate::Error::Input(error.to_string()))?;
            }
            for selection in &plan.selections {
                let template = &selection.template_id;
                let Some(suite) = dokime_client::template_suite(template) else {
                    return Err(crate::Error::Input(format!(
                        "`{template}` is not a template ID: it is `<suite ID>.<segment>`"
                    )));
                };
                if selection.suite_id != suite {
                    return Err(crate::Error::Input(format!(
                        "template `{template}` belongs to suite `{suite}`, not `{}`",
                        selection.suite_id
                    )));
                }
            }
        }
        Ok(Self { plan })
    }

    // The backend plans registered selections; each readable template ID is
    // read from the catalog once, however many selections sweep it.
    pub(crate) async fn resolve(
        self,
        client: &dokime_client::Client,
    ) -> crate::Result<Option<SelectionPlan>> {
        let Self { mut plan } = self;
        let mut registered = BTreeMap::new();
        for selection in plan.iter_mut().flat_map(|plan| plan.selections.iter_mut()) {
            if !registered.contains_key(&selection.template_id) {
                let (_, definition) = client.template(&selection.template_id).await?;
                registered.insert(selection.template_id.clone(), definition.template_id);
            }
            selection.template_id = registered[&selection.template_id].clone();
        }
        Ok(plan)
    }
}

/* === Public Functions === */
// Planned cases name their template by its readable ID; the registered UUID
// the backend answers with never reaches the operator.
pub(super) async fn print(
    client: &dokime_client::Client,
    reply: &PreviewSelectionReply,
    json: bool,
) -> crate::Result<()> {
    // === Read Template References === //

    let mut references = BTreeMap::new();
    let mut read = BTreeSet::new();
    for case in &reply.cases {
        if references.contains_key(&case.template_id) || !read.insert(case.suite_id.clone()) {
            continue;
        }
        let suite = client.suite(&case.suite_id).await?;
        references.extend(suite.templates.into_iter().filter_map(|template| {
            template
                .metadata
                .map(|metadata| (template.template_id, metadata.identifier))
        }));
    }

    // === Write Readable Cases === //

    let mut value = serde_json::to_value(reply)?;
    let cases = value
        .get_mut("cases")
        .and_then(serde_json::Value::as_array_mut)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_object_mut);
    for case in cases {
        let Some(serde_json::Value::String(id)) = case.get("template_id") else {
            continue;
        };
        let template = references.get(id).ok_or_else(|| {
            crate::Error::Setup("a planned template is no longer registered in its suite".into())
        })?;
        case.insert("template_id".into(), template.clone().into());
    }
    let output = if json {
        serde_json::to_string(&value)?
    } else {
        serde_json::to_string_pretty(&value)?
    };
    println!("{output}");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
