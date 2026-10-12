/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::{Mutex, PoisonError};
// Third Party Imports

// Local Imports
use crate::core::Identifier;
use crate::document::{Card, DocumentAddition, DocumentDefinition, PlotDefinition, Section};
use crate::execution::log::Level;
use crate::transport::ExecutionTransport;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct DocumentIndex {
    document: DocumentDefinition,
}

pub struct DocumentHandle<'a> {
    index: &'a Mutex<DocumentIndex>,
    transport: &'a dyn ExecutionTransport,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DocumentIndex {
    pub(crate) fn new(declared: Option<DocumentDefinition>) -> Self {
        Self {
            document: declared.unwrap_or_else(|| DocumentDefinition::new("default", "Document")),
        }
    }
}

impl<'a> DocumentHandle<'a> {
    pub(crate) fn new(
        index: &'a Mutex<DocumentIndex>,
        transport: &'a dyn ExecutionTransport,
    ) -> Self {
        Self { index, transport }
    }

    pub fn section(&mut self, section: Section) {
        self.add(DocumentAddition::section(section));
    }

    pub fn plot(&mut self, section_id: &str, plot: PlotDefinition) {
        self.card(section_id, Card::plot(plot));
    }

    pub fn card(&mut self, section_id: &str, card: Card) {
        let Some(section) = self.section_id(section_id) else {
            return;
        };

        self.add(DocumentAddition::card(section, card));
    }

    pub fn note(&mut self, section_id: &str, text: impl Into<String>) {
        let Some(section) = self.section_id(section_id) else {
            return;
        };
        let Some(ordinal) = self.locked(|index| index.document.section_card_count(&section)) else {
            self.refuse(format!(
                "section \"{section}\" is not in this document; add the section before the note it \
                 carries"
            ));
            return;
        };

        self.add(DocumentAddition::card(
            section.clone(),
            Card::text(format!("{section}.note.{ordinal}"), "Note", text),
        ));
    }

    fn add(&mut self, addition: DocumentAddition) {
        if let Err(refusal) = self.locked(|index| index.document.add(addition.clone())) {
            self.refuse(refusal);
            return;
        }

        self.transport.document(addition);
    }

    fn section_id(&self, section_id: &str) -> Option<Identifier> {
        match Identifier::new(section_id) {
            Ok(identifier) => Some(identifier),
            Err(error) => {
                self.refuse(format!(
                    "\"{section_id}\" is not a section identifier: {error}"
                ));
                None
            }
        }
    }

    fn locked<T>(&self, read: impl FnOnce(&mut DocumentIndex) -> T) -> T {
        read(&mut self.index.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn refuse(&self, reason: String) {
        self.transport.log(
            Level::Error,
            format!("the document was not changed: {reason}"),
        );
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
