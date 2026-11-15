/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Suite

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::plugin::Template;
use crate::{Metadata, Suite as SuiteDefinition};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Suite {
    pub(crate) definition: SuiteDefinition,
    pub(crate) templates: Vec<Template>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Suite {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            definition: SuiteDefinition {
                metadata: Some(Metadata {
                    identifier: identifier.into(),
                    label: label.into(),
                    description: String::new(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            templates: Vec::new(),
        }
    }
    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.definition.tags.push(tag.into());
        self
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Suite Metadata").description = description.into();
        self
    }
    pub fn parameter_group(mut self, group: Metadata) -> Self {
        self.definition.parameter_groups.push(group);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
