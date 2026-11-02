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
use crate::model::diagnostics::DebugLevel;
use crate::model::diagnostics::ErrorCause;
use crate::model::execution::RecordScope;
use crate::model::identity::ArtifactId;
use crate::model::identity::Identifier;
use crate::model::identity::SessionId;
use crate::model::identity::TestCycleId;
use crate::model::identity::TestExecutionId;
use crate::model::identity::TraceId;
use crate::model::time::Timestamp;

/* === Constants === */

pub const MISSING_SECRET_CODE: i64 = 10_001;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPluginErrorRecord {
    pub plugin_id: Identifier,
    pub contribution_id: Option<Identifier>,
    pub phase: InstalledPluginPhase,
    pub level: DebugLevel,
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<ErrorCause>,
    pub stacktrace_attachment: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub scope: Option<RecordScope>,
    pub session_id: Option<SessionId>,
    pub test_cycle_id: Option<TestCycleId>,
    pub test_execution_id: Option<TestExecutionId>,
    pub recorded_at: Timestamp,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstalledPluginPhase {
    Entrypoint,
    Action,
    Sidecar,
    Provider,
    Initialize,
    Run,
    Finalize,
    Interrupt,
}

/* === Traits === */

/* === Implementations === */

impl InstalledPluginErrorRecord {
    pub fn new(
        plugin_id: Identifier, contribution_id: Option<Identifier>, phase: InstalledPluginPhase, level: DebugLevel,
        message: impl Into<String>, recorded_at: Timestamp,
    ) -> Self {
        Self {
            plugin_id,
            contribution_id,
            phase,
            level,
            message: message.into(),
            code: None,
            source: None,
            cause_chain: Vec::new(),
            stacktrace_attachment: None,
            trace_id: None,
            scope: None,
            session_id: None,
            test_cycle_id: None,
            test_execution_id: None,
            recorded_at,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_scope(mut self, session_id: SessionId, scope: Option<RecordScope>) -> Self {
        self.session_id = Some(session_id);
        self.test_cycle_id = scope.as_ref().and_then(|scope| scope.test_cycle_id.clone());
        self.test_execution_id = scope.as_ref().and_then(|scope| scope.test_execution_id.clone());
        self.scope = scope;
        self
    }

    pub fn with_trace_id(mut self, trace_id: TraceId) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
