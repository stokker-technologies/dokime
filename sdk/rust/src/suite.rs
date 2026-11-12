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
use crate::{Template, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Suite {
    pub(crate) definition: v1::Suite,
    pub(crate) templates: Vec<Template>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Suite {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            definition: v1::Suite {
                metadata: Some(v1::Metadata {
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
    pub fn parameter_group(mut self, group: v1::Metadata) -> Self {
        self.definition.parameter_groups.push(group);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
