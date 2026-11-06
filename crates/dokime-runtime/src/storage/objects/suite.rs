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
    pub fn new(suite_id: &str, source: &TestSuite) -> Result<harmos::storage::ReadBatch<Self>> {
        if suite_id.trim().is_empty() {
            return Err(Error::Invalid("suite registration ID is required"));
        }
        let seed = harmos::storage::ReadBatch::new(());
        seed.try_map(|_| {
            let mut id = String::new();
            id.try_reserve_exact(suite_id.len()).map_err(|_| Error::Invalid("suite ID allocation failed"))?;
            id.push_str(suite_id);
            Ok(Self {
                suite_id: id,
                definition: Some(templates::Definition::try_from(source)?),
            })
        })
    }

    fn into_suite(self) -> Result<TestSuite> {
        let Self { suite_id, definition } = self;
        if suite_id.trim().is_empty() {
            return Err(Error::Invalid("suite registration ID is required"));
        }
        definition.ok_or(Error::Invalid("suite definition is missing"))?.try_into()
    }
}

/* === Public Functions === */

pub async fn read(storage: &Manager, suite_id: &str) -> Result<Option<harmos::storage::ReadBatch<TestSuite>>> {
    let session = storage.clone();
    let saved = session.read_object::<Suite>(suite_id, ObjectVersion::Latest).await?;
    saved.map(|saved| saved.try_map(|saved| saved.value.into_suite())).transpose()
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::identity::{Identifier, Metadata};

    fn definition() -> TestSuite {
        TestSuite {
            metadata: Metadata::new(Identifier::new("fixture.suite").unwrap(), "Fixture", None),
            tags: vec!["second".into(), "first".into()],
            parameter_groups: Vec::new(),
            test_case_templates: Vec::new(),
        }
    }

    #[test]
    fn typed_definition_preserves_authored_order() {
        let source = definition();
        let row = templates::Definition::try_from(&source).unwrap();
        let restored = TestSuite::try_from(row).unwrap();
        assert_eq!(restored, source);
    }
}
