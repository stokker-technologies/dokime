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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    DefinitionId, Identified,
    storage::{ObjectRetention, ObjectVersion, StoredObject},
};

// Local Imports
use super::metadata;
use crate::core::{CaseId, TemplateId};
use crate::definition::TestCase;
use crate::storage::{Error, Manager, Result, values};

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
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Identified for Case {
    fn definition() -> (DefinitionId, u32) {
        (DefinitionId::new("dokime.object.case"), 1)
    }
}

impl StoredObject for Case {
    type Key = str;
    const RETENTION: ObjectRetention = ObjectRetention::Latest;

    fn key(&self) -> &str {
        &self.case_id
    }
}

impl TryFrom<&TestCase> for Case {
    type Error = Error;

    fn try_from(case: &TestCase) -> Result<Self> {
        if Self::encoding_memory(case)? == usize::MAX {
            return Err(Error::Invalid("case encoding size overflow"));
        }

        Ok(Self {
            suite_id: case.suite_id.clone(),
            case_id: case.test_case_id.as_str().to_owned(),
            template_id: case.test_case_template_id.as_str().to_owned(),
            metadata: Some(metadata::Row::try_from(&case.metadata)?),
            suite: case
                .suite
                .as_ref()
                .map(metadata::Row::try_from)
                .transpose()?,
            parameters: values::encode(&case.parameters)?,
            generation_index: case.generation_index,
            label_number: case.label_number,
        })
    }
}

impl TryFrom<Case> for TestCase {
    type Error = Error;

    fn try_from(case: Case) -> Result<Self> {
        if case.suite_id.trim().is_empty() {
            return Err(Error::Invalid(
                "case requires its owning suite registration",
            ));
        }

        Ok(Self {
            suite_id: case.suite_id,
            test_case_id: CaseId::from_wire(case.case_id)?,
            test_case_template_id: TemplateId::from_uuid(case.template_id)?,
            metadata: case
                .metadata
                .ok_or(Error::Invalid("case metadata is missing"))?
                .try_into()?,
            suite: case.suite.map(TryInto::try_into).transpose()?,
            parameters: {
                let bound = values::owned_memory(&case.parameters)?;
                values::decode_owned(
                    case.parameters,
                    NonZeroUsize::new(bound).ok_or(Error::Invalid("case projection is empty"))?,
                )?
            },
            generation_index: case.generation_index,
            label_number: case.label_number,
        })
    }
}

/* === Public Functions === */
pub async fn read(storage: &Manager, id: &CaseId) -> Result<Option<TestCase>> {
    let Some(stored) = storage
        .read_object::<Case>(id.as_str(), ObjectVersion::Latest)
        .await?
    else {
        return Ok(None);
    };
    Ok(Some(stored.value.try_into()?))
}

/* === Private Functions === */
impl Case {
    pub fn encoding_memory(case: &TestCase) -> Result<usize> {
        if case.suite_id.trim().is_empty() {
            return Err(Error::Invalid(
                "case requires its owning suite registration",
            ));
        }
        let metadata = metadata::Row::encoding_memory(&case.metadata)?;
        let suite = case
            .suite
            .as_ref()
            .map(metadata::Row::encoding_memory)
            .transpose()?;
        let parameters = values::encoding_memory(&case.parameters)?;
        if metadata == usize::MAX || suite == Some(usize::MAX) || parameters == usize::MAX {
            return Ok(usize::MAX);
        }
        let fields = [
            case.suite_id.len(),
            case.test_case_id.as_str().len(),
            case.test_case_template_id.as_str().len(),
            metadata.saturating_sub(std::mem::size_of::<metadata::Row>()),
            suite.map_or(0, |bytes| {
                bytes.saturating_sub(std::mem::size_of::<metadata::Row>())
            }),
            parameters.saturating_sub(std::mem::size_of::<Vec<values::Cell>>()),
        ];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), usize::saturating_add))
    }

    pub fn projection_memory(&self) -> Result<usize> {
        let metadata = self
            .metadata
            .as_ref()
            .ok_or(Error::Invalid("case metadata is missing"))?
            .projection_memory()?;
        let suite = self
            .suite
            .as_ref()
            .map(metadata::Row::projection_memory)
            .transpose()?
            .unwrap_or(0);
        let parameters = values::owned_memory(&self.parameters)?;
        if [metadata, suite, parameters].contains(&usize::MAX) {
            return Ok(usize::MAX);
        }
        let fields = [
            self.suite_id.capacity(),
            self.case_id.capacity(),
            self.template_id.capacity(),
            metadata.saturating_sub(std::mem::size_of::<crate::core::Metadata>()),
            suite.saturating_sub(std::mem::size_of::<crate::core::Metadata>()),
            parameters.saturating_sub(std::mem::size_of::<
                std::collections::BTreeMap<String, crate::core::Value>,
            >()),
        ];
        Ok(fields
            .into_iter()
            .fold(std::mem::size_of::<TestCase>(), usize::saturating_add))
    }
}

/* === Tests === */
