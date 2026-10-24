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
use harmos::{
    DefinitionId, Identified,
    storage::{ObjectRetention, ObjectVersion, StoredObject},
};

// Local Imports
use super::templates;
use crate::definition::TestSuite;
use crate::storage::{Error, Manager, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Suite {
    suite_id: String,
    definition: Option<templates::Definition>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Identified for Suite {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.suite"), 1)
    }
}

impl StoredObject for Suite {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;
    fn key(&self) -> &str {
        &self.suite_id
    }
}

impl Suite {
    pub fn new(suite_id: &str, source: &TestSuite) -> Result<Self> {
        if suite_id.trim().is_empty() {
            return Err(Error::Invalid("suite registration ID is required"));
        }
        let definition: templates::Definition = templates::convert(source)?;
        let restored: TestSuite = templates::convert(&definition)?;
        let original = persisted_definition(source);
        if original != restored {
            return Err(Error::Invalid("suite definition cannot be represented losslessly"));
        }
        Ok(Self { suite_id: suite_id.into(), definition: Some(definition) })
    }

    pub fn restore(&self) -> Result<TestSuite> {
        let definition = self.definition.as_ref().ok_or(Error::Invalid("suite definition is missing"))?;
        let restored: TestSuite = templates::convert(definition)?;
        let canonical = Self::new(&self.suite_id, &restored)?;
        if canonical != *self {
            return Err(Error::Invalid("suite definition contains noncanonical fields"));
        }
        Ok(restored)
    }
}

/* === Public Functions === */

pub async fn read(storage: &Manager, suite_id: &str) -> Result<Option<TestSuite>> {
    let saved = storage.read_object::<Suite>(suite_id, ObjectVersion::Latest).await?;
    saved.map(|saved| saved.value.restore()).transpose()
}

/* === Private Functions === */

fn persisted_definition(source: &TestSuite) -> TestSuite {
    let mut definition = source.clone();
    for template in &mut definition.test_case_templates {
        let Some(document) = &mut template.document else {
            continue;
        };
        templates::normalize_document(document);
    }
    definition
}

/* === Tests === */
