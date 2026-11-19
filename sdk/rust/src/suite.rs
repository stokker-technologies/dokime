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
use crate::{Metadata, exports};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Suite {
    pub(crate) definition: exports::Suite,
    pub(crate) templates: Vec<Template>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Suite {
    pub fn of<T>() -> Self {
        let identifier = crate::authoring::code_name::<T>(true);
        let label = crate::authoring::type_label::<T>();
        Self {
            definition: exports::Suite {
                metadata: Some(Metadata { identifier, label, ..Default::default() }),
                ..Default::default()
            },
            templates: Vec::new(),
        }
    }
    pub fn identifier(mut self, identifier: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Suite Metadata").identifier = identifier.into();
        self
    }
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Suite Metadata").label = label.into();
        self
    }

    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }
    pub(crate) fn bind(mut self) -> Self {
        let identifier = &self.definition.metadata.as_ref().expect("Suite Metadata").identifier;
        self.templates = self.templates.into_iter().map(|template| template.bind(identifier)).collect();
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
