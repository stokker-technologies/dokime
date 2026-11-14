/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompt

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{AnswerRule, EvidenceCondition, Metadata, Prompt as PromptDefinition, PromptAction, PromptArchetype, PromptField};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Prompt {
    pub(crate) definition: PromptDefinition,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Prompt {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            definition: PromptDefinition {
                metadata: Some(Metadata {
                    identifier: identifier.into(),
                    label: label.into(),
                    description: String::new(),
                    ..Default::default()
                }),
                message: message.into(),
                ..Default::default()
            },
        }
    }
    pub fn field(mut self, field: PromptField) -> Self {
        self.definition.fields.push(field);
        self
    }
    pub fn action(mut self, identifier: impl Into<String>, label: impl Into<String>) -> Self {
        self.definition.actions.push(PromptAction {
            identifier: identifier.into(),
            label: label.into(),
            ..Default::default()
        });
        self
    }
    pub fn archetype(mut self, archetype: PromptArchetype) -> Self {
        self.definition.archetype = archetype as i32;
        self
    }
    pub fn rule(mut self, rule: AnswerRule) -> Self {
        self.definition.rules.push(rule);
        self
    }
    pub fn condition(mut self, condition: EvidenceCondition) -> Self {
        self.definition.condition = Some(condition);
        self
    }
    pub fn into_message(self) -> PromptDefinition {
        self.definition
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
