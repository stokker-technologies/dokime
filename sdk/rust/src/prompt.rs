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
use crate::{AnswerRule, EvidenceCondition, Metadata, PromptAction, PromptArchetype, PromptField, wire::model};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Prompt {
    pub(crate) definition: model::Prompt,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<&str> for Prompt {
    fn from(text: &str) -> Self {
        Self::new("question", text, text).action("0", "OK")
    }
}

impl From<&Prompt> for Prompt {
    fn from(prompt: &Prompt) -> Self {
        prompt.clone()
    }
}

impl Prompt {
    pub fn of<H: Fn() -> Self>(_: H, label: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(crate::authoring::code_name::<H>(), label, message)
    }

    pub fn new(identifier: impl Into<String>, label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            definition: model::Prompt {
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
    pub fn into_message(self) -> model::Prompt {
        self.definition
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
