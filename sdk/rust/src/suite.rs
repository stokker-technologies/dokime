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
use crate::{Metadata, wire::model};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Suite {
    pub(crate) definition: model::Suite,
    pub(crate) templates: Vec<Template>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Suite {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            definition: model::Suite {
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
    pub fn single(template: Template) -> Self {
        let identifier = template.identifier().to_owned();
        let mut suite = Self::new(&identifier, template.label());
        suite.templates.push(template.bind(&identifier, true));
        suite
    }

    pub fn template(mut self, template: Template) -> Self {
        let identifier = &self.definition.metadata.as_ref().expect("Suite Metadata").identifier;
        self.templates.push(template.bind(identifier, false));
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
