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

impl TryFrom<&crate::definition::TestSuite> for Definition {
    type Error = Error;

    fn try_from(source: &crate::definition::TestSuite) -> Result<Self> {
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
