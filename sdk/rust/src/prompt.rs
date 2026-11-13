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
use crate::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Prompt {
    pub(crate) definition: v1::Prompt,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Prompt {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            definition: v1::Prompt {
                metadata: Some(v1::Metadata {
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
    pub fn field(mut self, field: v1::PromptField) -> Self {
        self.definition.fields.push(field);
        self
    }
    pub fn action(mut self, identifier: impl Into<String>, label: impl Into<String>) -> Self {
        self.definition.actions.push(v1::PromptAction {
            identifier: identifier.into(),
            label: label.into(),
            ..Default::default()
        });
        self
    }
    pub fn archetype(mut self, archetype: v1::PromptArchetype) -> Self {
        self.definition.archetype = archetype as i32;
        self
    }
    pub fn rule(mut self, rule: v1::AnswerRule) -> Self {
        self.definition.rules.push(rule);
        self
    }
    pub fn condition(mut self, condition: v1::EvidenceCondition) -> Self {
        self.definition.condition = Some(condition);
        self
    }
    pub fn into_message(self) -> v1::Prompt {
        self.definition
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
