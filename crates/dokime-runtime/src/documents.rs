/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Document Declaration Selection

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::Declarations;
use prost::Message;

// Local Imports
use crate::{Error, Host, Result};
use dokime_protocol::host::DocumentScope;
use dokime_protocol::model::{Card, Document, EvidenceCard, Metadata, Section, Suites, card, origin};
use dokime_storage::{ExecutionCut, Report, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Host {
    pub fn current_document(&self, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)> {
        self.current_document_in(&self.store, scope, identifier)
    }
    pub(crate) fn current_document_in(&self, store: &Store, scope: &DocumentScope, identifier: &str) -> Result<(Document, ExecutionCut)> {
        if scope.execution_id.is_empty() {
            if !identifier.is_empty() && identifier != "default" {
                return Err(Error::Invalid("Capture Cycle Document: Select the Default Document".into()));
            }
            let execution = store
                .cycle_execution_page(&scope.cycle_id, "", NonZeroUsize::new(1).unwrap())?
                .into_iter()
                .next()
                .ok_or_else(|| Error::Invalid("Capture Cycle Document: Create or Import an Execution in This Cycle First".into()))?;
            let mut cut = store.cut(&execution)?;
            cut.frontier = store.cycle_evidence_frontier(&scope.cycle_id)?;
            let sections = [
                ("events", "Records", EvidenceCard::Records),
                ("artifacts", "Artifacts", EvidenceCard::Artifacts),
                ("media", "Media", EvidenceCard::Media),
            ]
            .into_iter()
            .map(|(id, label, kind)| Section {
                metadata: Some(Metadata {
                    identifier: format!("cycle.{id}"),
                    label: label.into(),
                    ..Default::default()
                }),
                cards: vec![Card {
                    metadata: Some(Metadata {
                        identifier: format!("cycle.{id}.list"),
                        label: label.into(),
                        ..Default::default()
                    }),
                    body: Some(card::Body::Evidence(kind as i32)),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .collect();
            return Ok((
                Document {
                    metadata: Some(Metadata {
                        identifier: "default".into(),
                        label: "Cycle Report".into(),
                        ..Default::default()
                    }),
                    sections,
                },
                cut,
            ));
        }
        let cut = store.cut(&scope.execution_id)?;
        if cut.execution.cycle != scope.cycle_id {
            return Err(Error::Invalid("Capture Document: Select the Execution in Its Own Cycle".into()));
        }
        let catalog = Declarations::decode(cut.execution.catalog.as_slice())?;
        let mut documents = Vec::new();
        for section in &catalog.sections {
            if section.r#type != "dokime.model.v1.Suites" {
                continue;
            }
            let suites = Suites::decode(section.payload.as_slice())?;
            for suite in suites.suites {
                for template in suite.templates {
                    let matched = match cut.execution.origin.as_ref().and_then(|origin| origin.kind.as_ref()) {
                        Some(origin::Kind::PluginRun(run)) => {
                            template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template)
                        }
                        _ => true,
                    };
                    if matched {
                        documents.extend(template.documents.into_iter().filter(|document| {
                            identifier.is_empty() || document.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier)
                        }));
                    }
                }
            }
        }
        if documents.is_empty() && identifier.is_empty() {
            documents.push(Document {
                metadata: Some(Metadata {
                    identifier: "execution".into(),
                    label: "Execution Report".into(),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        if documents.len() != 1 {
            return Err(Error::Invalid("Capture Document: Select One Document ID from the Frozen Execution Declarations".into()));
        }
        let mut document = documents.remove(0);
        let id = document.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
        apply_preferences(store, &cut.execution.identifier, id, &mut document.sections)?;
        Ok((document, cut))
    }

    pub fn reports(&self, execution: &str) -> Result<Vec<Report>> {
        Ok(self.store.reports(execution)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn apply_preferences(store: &Store, execution: &str, document: &str, sections: &mut [Section]) -> Result<()> {
    for section in sections {
        for card in &mut section.cards {
            if let Some(card::Body::Plot(plot)) = &mut card.body {
                let id = plot.metadata.as_ref().map_or("", |metadata| metadata.identifier.as_str());
                if let Some((points, _)) = store.plot_preferences(execution, document, id)? {
                    plot.display_points = points;
                }
            }
        }
        apply_preferences(store, execution, document, &mut section.sections)?;
    }
    Ok(())
}

/* === Tests === */
