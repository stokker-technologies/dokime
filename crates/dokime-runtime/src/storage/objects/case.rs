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
use harmos::storage::{ObjectRetention, ObjectVersion, StoredObject};

// Local Imports
use super::metadata;
use crate::definition::TestCase;
use crate::storage::{Error, Result, values};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TemplateId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    suite_id: String,

    #[serde(rename = "test_case_id")]
    case_id: String,

    #[serde(rename = "test_case_template_id")]
    template_id: String,

    metadata: Option<metadata::Row>,
    suite: Option<metadata::Row>,

    #[serde(with = "values::map")]
    parameters: Vec<values::Cell>,

    generation_index: u64,
    label_number: u64,
    definition_revision: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct Registration {
    pub(crate) id: String,
    pub(crate) cases: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

crate::storage::object_identity!(Case, "dokime.object.case");

impl StoredObject for Case {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &str {
        &self.case_id
    }
}

crate::storage::object_identity!(Registration, "dokime.object.case_registration");

impl StoredObject for Registration {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &str {
        &self.id
    }
}

impl TryFrom<&TestCase> for Case {
    type Error = Error;

    fn try_from(case: &TestCase) -> Result<Self> {
        if case.suite_id.trim().is_empty() {
            return Err(Error::Invalid("case requires its owning suite registration"));
        }
        Ok(Self {
            suite_id: case.suite_id.clone(),
            case_id: case.test_case_id.as_str().to_owned(),
            template_id: case.test_case_template_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&case.metadata)?),
            suite: case.suite.as_ref().map(metadata::Row::try_from).transpose()?,
            parameters: values::encode(&case.parameters)?,
            generation_index: case.generation_index,
            label_number: case.label_number,
            definition_revision: String::new(),
        })
    }
}

impl TryFrom<Case> for TestCase {
    type Error = Error;

    fn try_from(case: Case) -> Result<Self> {
        if case.suite_id.trim().is_empty() {
            return Err(Error::Invalid("case requires its owning suite registration"));
        }

        Ok(Self {
            suite_id: case.suite_id,
            test_case_id: CaseId::from_wire(case.case_id)?,
            test_case_template_id: TemplateId::from_uuid(case.template_id)?,
            metadata: case.metadata.ok_or(Error::Invalid("case metadata is missing"))?.try_into()?,
            suite: case.suite.map(TryInto::try_into).transpose()?,
            parameters: { values::decode_owned(case.parameters)? },
            generation_index: case.generation_index,
            label_number: case.label_number,
        })
    }
}

impl Case {
    #[cfg(feature = "native")]
    pub(crate) fn capture_revision(&mut self, revision: String) {
        self.definition_revision = revision;
    }
}

/* === Public Functions === */

pub async fn read(storage: &crate::storage::Manager, id: &CaseId) -> Result<Option<harmos::storage::ReadBatch<TestCase>>> {
    let Some(stored) = storage.read_object::<Case>(id.as_str(), ObjectVersion::Latest).await? else {
        return Ok(None);
    };
    Ok(Some(stored.try_map(|stored| stored.value.try_into())?))
}

#[cfg(feature = "native")]
pub(crate) async fn read_registered(
    storage: &crate::storage::Manager, id: &CaseId,
) -> Result<Option<harmos::storage::ReadBatch<(TestCase, String)>>> {
    let Some(stored) = storage.read_object::<Case>(id.as_str(), ObjectVersion::Latest).await? else {
        return Ok(None);
    };
    Ok(Some(stored.try_map(|stored| {
        let revision = stored.value.definition_revision.clone();
        Ok::<_, Error>((stored.value.try_into()?, revision))
    })?))
}

/* === Private Functions === */

/* === Tests === */
