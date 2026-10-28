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

// Third Party Imports

// Local Imports
use super::{Definition, Group, Template, copied_strings, exact_slots, metadata};
use crate::storage::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Definition {
    pub(in crate::storage::objects) fn encoding_memory(source: &crate::definition::TestSuite) -> Result<usize> {
        let crate::definition::TestSuite {
            metadata,
            tags,
            parameter_groups,
            test_case_templates,
        } = source;
        let mut bytes = std::mem::size_of::<Self>();
        bytes = bytes.saturating_add(metadata::Row::encoding_memory(metadata)?);
        bytes = bytes.saturating_add(tags.len().saturating_mul(std::mem::size_of::<String>()));
        for tag in tags {
            bytes = bytes.saturating_add(tag.len());
        }
        for group in parameter_groups {
            bytes = bytes.saturating_add(Group::encoding_memory(group)?);
        }
        for template in test_case_templates {
            bytes = bytes.saturating_add(Template::encoding_memory(template)?);
        }
        Ok(bytes)
    }

    pub(in crate::storage::objects) fn projection_memory(&self) -> Result<usize> {
        use harmos::MemorySize;
        let Self {
            metadata,
            tags,
            parameter_groups,
            test_case_templates,
        } = self;
        let metadata = metadata.as_ref().ok_or(Error::Invalid("suite metadata is missing"))?;
        let mut bytes = std::mem::size_of::<crate::definition::TestSuite>();
        bytes = bytes.saturating_add(metadata.projection_memory()?);
        bytes = bytes.saturating_add(tags.memory_size());
        for group in parameter_groups {
            bytes = bytes.saturating_add(group.projection_memory()?);
        }
        for template in test_case_templates {
            bytes = bytes.saturating_add(template.projection_memory()?);
        }
        Ok(bytes)
    }
}

impl TryFrom<&crate::definition::TestSuite> for Definition {
    type Error = Error;

    fn try_from(source: &crate::definition::TestSuite) -> Result<Self> {
        if Self::encoding_memory(source)? == usize::MAX {
            return Err(Error::Invalid("suite encoding memory overflows"));
        }
        let crate::definition::TestSuite {
            metadata,
            tags,
            parameter_groups,
            test_case_templates,
        } = source;
        let mut groups = exact_slots(parameter_groups.len())?;
        for group in parameter_groups {
            groups.push(Group::try_from(group)?);
        }
        let mut templates = exact_slots(test_case_templates.len())?;
        for template in test_case_templates {
            templates.push(Template::try_from(template)?);
        }
        Ok(Self {
            metadata: Some(metadata::Row::try_from(metadata)?),
            tags: copied_strings(tags)?,
            parameter_groups: groups,
            test_case_templates: templates,
        })
    }
}

impl TryFrom<Definition> for crate::definition::TestSuite {
    type Error = Error;

    fn try_from(row: Definition) -> Result<Self> {
        if row.projection_memory()? == usize::MAX {
            return Err(Error::Invalid("suite projection memory overflows"));
        }
        let Definition {
            metadata,
            tags,
            parameter_groups,
            test_case_templates,
        } = row;
        let mut groups = exact_slots(parameter_groups.len())?;
        for group in parameter_groups {
            groups.push(group.try_into()?);
        }
        let mut templates = exact_slots(test_case_templates.len())?;
        for template in test_case_templates {
            templates.push(template.try_into()?);
        }
        Ok(Self {
            metadata: metadata.ok_or(Error::Invalid("suite metadata is missing"))?.try_into()?,
            tags,
            parameter_groups: groups,
            test_case_templates: templates,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
