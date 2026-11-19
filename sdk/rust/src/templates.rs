/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Template Authoring Surface

/* === Modules === */

/* === Re-Exports === */

pub use crate::context::{Context, RecordsQuery};
pub use crate::prompt::Prompt;
pub use crate::record::Record;
pub use crate::stream::{Observer, Stream};
pub use crate::template::{Criterion, Step, Template};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::model::Section;
use crate::{CaptureGap, CapturedArtifact, Card};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureResult {
    pub artifacts: Vec<CapturedArtifact>,
    pub gaps: Vec<CaptureGap>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Environment {
    pub backend_pid: u32,
    pub in_step: bool,
}

/* === Enums === */

pub enum DocumentPart {
    Section(Section),
    Card(Card),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
