/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod preview_selection;
mod register_selection;

/* === Re-Exports === */
pub use preview_selection::PreviewSelection;
pub use register_selection::RegisterSelection;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
use super::cursor::Cursor;
use super::value::Value;
use super::{Access, Endpoint, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredSelectionParametersEntry {
    pub key: String,
    pub value: Option<SelectionValues>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredSelectionFilesEntry {
    pub key: String,
    pub value: Option<SelectedFile>,
}

// Ordered selections of exact registered IDs; no implicit suite or template.
// Every selection in one plan must share a provider source AND workflow; a
// cycle spans several workflows by appending a further plan. Overlapping and
// repeated selections remain separate cases, with deterministic IDs and order.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredSelection {
    pub suite_id: String,
    pub template_id: String,
    pub parameters: Vec<RegisteredSelectionParametersEntry>,
    pub files: Vec<RegisteredSelectionFilesEntry>,
}

// Ordinary parameter values form Cartesian dimensions. A parameter declared
// multiple=true receives the values as one array, following native planning.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionValues {
    pub values: Vec<Value>,
}

// Omission uses the registered default; clear explicitly removes a file slot.
// Values are opaque plugin-defined strings, not uploads or client file paths
// that the backend promises to resolve. No filesystem authority is granted.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedFile {
    pub kind: Option<SelectedFileKind>,
}

// At most 256 selections and 1000 generated cases across the entire plan.
// max_cases=0 uses 1000. No force flag bypasses this resource boundary.
// A plan selects registered case IDs or expands templates, never both.
// ID order and duplicates are preserved; repeated IDs create separate executions.
// Expanded cases are labelled "<Template Label> #<n>", numbered per template.
// A nonempty cycle_id continues that cycle's numbering; empty starts at #1.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionPlan {
    pub selections: Vec<RegisteredSelection>,
    pub max_cases: u32,
    pub case_ids: Vec<String>,
    pub cycle_id: String,
}

// Registration owns case identity independently of cycles. A nonempty key makes
// retries reproduce the same case IDs; conflicting content is rejected.




#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewSelectionReply {
    pub cases: Vec<PlannedCase>,
    pub observed_cursor: Option<Cursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedCase {
    pub case_id: String,
    pub suite_id: String,
    pub template_id: String,
    pub metadata: Option<Metadata>,
    pub parameters: Option<Value>,
    pub ordinal: u64,
    pub selection_index: u64,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectedFileKind {
    Value(String),
    Clear,
}

#[harmos::catalog]
pub enum Operations {
    RegisterSelection(RegisterSelection),
    PreviewSelection(PreviewSelection),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for RegisterSelection {
    const PATH: &'static str = "/dokime.v1.Planning/RegisterSelection";
    const ACCESS: Access = Access::Control;
}

impl Endpoint for PreviewSelection {
    const PATH: &'static str = "/dokime.v1.Planning/PreviewSelection";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
