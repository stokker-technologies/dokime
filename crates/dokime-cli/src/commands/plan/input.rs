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
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime_contract::operations::planning::{
    PreviewSelectionReply, RegisteredSelection, SelectionPlan,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
pub(super) struct Args {
    #[arg(long, conflicts_with_all = ["suite_id", "template_id", "case_ids", "selections", "max_cases"], help = "Read The Canonical Request JSON Without Flag Overrides.")]
    pub(super) input_file: Option<PathBuf>,
    #[arg(long, requires = "template_id", conflicts_with_all = ["case_ids", "selections"], help = "Exact Registered Suite ID; Omitted Parameters Use Backend Defaults.")]
    suite_id: Option<String>,
    #[arg(long, requires = "suite_id")]
    template_id: Option<String>,
    #[arg(
        long = "case-id",
        conflicts_with = "selections",
        help = "Registered Case ID; Repeat To Preserve Order And Repeated Attempts."
    )]
    case_ids: Vec<String>,
    #[arg(
        long = "selection-json",
        help = "Canonical RegisteredSelection JSON; Repeat For Ordered Selections."
    )]
    selections: Vec<String>,
    #[arg(
        long,
        default_value_t = 0,
        help = "Case Limit; Zero Uses The Backend Limit."
    )]
    max_cases: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Args {
    pub(super) fn plan(self) -> crate::Result<SelectionPlan> {
        let mut selections = self
            .selections
            .into_iter()
            .map(|json| {
                serde_json::from_str::<RegisteredSelection>(&json).map_err(|error| {
                    crate::Error::Input(format!("invalid selection input: {error}"))
                })
            })
            .collect::<crate::Result<Vec<_>>>()?;
        if let (Some(suite_id), Some(template_id)) = (self.suite_id, self.template_id) {
            selections.push(RegisteredSelection {
                suite_id,
                template_id,
                parameters: Vec::new(),
                files: Vec::new(),
            });
        }
        if selections.is_empty() && self.case_ids.is_empty() {
            return Err(crate::Error::Input(
                "a request file, selection or case ID is required".into(),
            ));
        }
        Ok(SelectionPlan {
            selections,
            max_cases: self.max_cases,
            case_ids: self.case_ids,
        })
    }
}

/* === Public Functions === */
pub(super) fn validate(plan: Option<&SelectionPlan>) -> crate::Result<()> {
    if let Some(plan) = plan {
        for id in &plan.case_ids {
            dokime_contract::core::CaseId::from_wire(id)
                .map_err(|error| crate::Error::Input(error.to_string()))?;
        }
    }
    Ok(())
}

pub(super) fn print(reply: &PreviewSelectionReply, json: bool) -> crate::Result<()> {
    let output = if json {
        serde_json::to_string(reply)?
    } else {
        serde_json::to_string_pretty(reply)?
    };
    println!("{output}");
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
