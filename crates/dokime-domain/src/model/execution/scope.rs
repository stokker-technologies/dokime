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
use serde::Deserialize;
use serde::Serialize;
// Local Imports
use crate::model::identity::id::CaseId;
use crate::model::identity::id::CriterionId;
use crate::model::identity::id::SessionId;
use crate::model::identity::id::StepAttemptId;
use crate::model::identity::id::StepId;
use crate::model::identity::id::SuiteId;
use crate::model::identity::id::TemplateId;
use crate::model::identity::id::TestCycleId;
use crate::model::identity::id::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(default)]
pub struct RecordScope {
    pub session_id: Option<SessionId>,
    pub test_cycle_id: Option<TestCycleId>,
    pub test_case_id: Option<CaseId>,
    pub test_execution_id: Option<TestExecutionId>,
    pub step_id: Option<StepId>,
    pub criterion_id: Option<CriterionId>,
    pub template_id: Option<TemplateId>,
    pub suite_id: Option<SuiteId>,
    pub origin: EvidenceOrigin,
}

/* === Enums === */
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EvidenceOrigin {
    #[default]
    Default,
    Initialization,
    Step {
        attempt_id: StepAttemptId,
    },
    Cleanup,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapturePhase {
    #[default]
    Default,
    Initialization,
    Cleanup,
}

/* === Traits === */

/* === Implementations === */
impl harmos_domain::MemorySize for RecordScope {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.session_id), std::mem::size_of_val(&self.session_id)),
            (harmos_domain::MemorySize::memory_size(&self.test_cycle_id), std::mem::size_of_val(&self.test_cycle_id)),
            (harmos_domain::MemorySize::memory_size(&self.test_case_id), std::mem::size_of_val(&self.test_case_id)),
            (harmos_domain::MemorySize::memory_size(&self.test_execution_id), std::mem::size_of_val(&self.test_execution_id)),
            (harmos_domain::MemorySize::memory_size(&self.step_id), std::mem::size_of_val(&self.step_id)),
            (harmos_domain::MemorySize::memory_size(&self.criterion_id), std::mem::size_of_val(&self.criterion_id)),
            (harmos_domain::MemorySize::memory_size(&self.template_id), std::mem::size_of_val(&self.template_id)),
            (harmos_domain::MemorySize::memory_size(&self.suite_id), std::mem::size_of_val(&self.suite_id)),
            (harmos_domain::MemorySize::memory_size(&self.origin), std::mem::size_of_val(&self.origin)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for EvidenceOrigin {
    fn memory_size(&self) -> usize {
        let dynamic = match self {
            Self::Step { attempt_id } => {
                harmos_domain::MemorySize::memory_size(attempt_id).saturating_sub(std::mem::size_of_val(attempt_id))
            }
            Self::Default | Self::Initialization | Self::Cleanup => 0,
        };
        std::mem::size_of::<Self>().saturating_add(dynamic)
    }
}

impl From<CapturePhase> for EvidenceOrigin {
    fn from(phase: CapturePhase) -> Self {
        match phase {
            CapturePhase::Default => Self::Default,
            CapturePhase::Initialization => Self::Initialization,
            CapturePhase::Cleanup => Self::Cleanup,
        }
    }
}

impl RecordScope {
    pub fn session(session_id: SessionId) -> Self {
        Self { session_id: Some(session_id), ..Self::default() }
    }

    pub fn with_cycle(mut self, test_cycle_id: TestCycleId) -> Self {
        self.test_cycle_id = Some(test_cycle_id);
        self
    }

    pub fn with_test_case(mut self, test_case_id: CaseId) -> Self {
        self.test_case_id = Some(test_case_id);
        self
    }

    pub fn with_test_execution(mut self, test_execution_id: TestExecutionId) -> Self {
        self.test_execution_id = Some(test_execution_id);
        self
    }

    pub fn with_step(mut self, step_id: StepId) -> Self {
        self.step_id = Some(step_id);
        self
    }

    pub fn with_criterion(mut self, criterion_id: CriterionId) -> Self {
        self.criterion_id = Some(criterion_id);
        self
    }

    pub fn with_template(mut self, template_id: TemplateId) -> Self {
        self.template_id = Some(template_id);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.session_id.is_none()
            && self.test_cycle_id.is_none()
            && self.test_case_id.is_none()
            && self.test_execution_id.is_none()
            && self.step_id.is_none()
            && self.criterion_id.is_none()
            && self.template_id.is_none()
            && self.suite_id.is_none()
            && self.origin == EvidenceOrigin::Default
    }

    pub fn is_visible_in(&self, selected: &RecordScope) -> bool {
        scope_visible_in(Some(self), selected)
    }
}

/* === Public Functions === */
pub fn scope_visible_in(record_scope: Option<&RecordScope>, selected: &RecordScope) -> bool {
    if selected.is_empty() {
        return true;
    }

    let Some(record_scope) = record_scope else {
        return false;
    };

    field_visible_in(&record_scope.session_id, &selected.session_id)
        && field_visible_in(&record_scope.test_cycle_id, &selected.test_cycle_id)
        && field_visible_in(&record_scope.test_case_id, &selected.test_case_id)
        && field_visible_in(&record_scope.test_execution_id, &selected.test_execution_id)
        && field_visible_in(&record_scope.step_id, &selected.step_id)
        && field_visible_in(&record_scope.criterion_id, &selected.criterion_id)
        && field_visible_in(&record_scope.template_id, &selected.template_id)
        && field_visible_in(&record_scope.suite_id, &selected.suite_id)
        && (selected.origin == EvidenceOrigin::Default || selected.origin == record_scope.origin)
}

/* === Private Functions === */
fn field_visible_in<T: PartialEq>(record: &Option<T>, selected: &Option<T>) -> bool {
    selected.as_ref().is_none_or(|selected| record.as_ref() == Some(selected))
}

/* === Tests === */
