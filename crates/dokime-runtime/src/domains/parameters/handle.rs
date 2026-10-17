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
use crate::definition::ParameterSource;
use crate::{
    RuntimeHandle,
    transactions::parameters::{UpsertScopedParameter, apply_upsert_scoped_parameter},
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::MeasurementUnit;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct ScopedParameterUpdate {
    key: String,
    value: Value,
    source: ParameterSource,
    metadata: Option<Metadata>,
    group_id: Option<Identifier>,
    unit: Option<MeasurementUnit>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ScopedParameterUpdate {
    pub(crate) fn new(
        key: impl Into<String>,
        value: impl Into<Value>,
        source: ParameterSource,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            source,
            metadata: None,
            group_id: None,
            unit: None,
        }
    }

    pub(crate) fn with_group_id(mut self, group_id: Option<Identifier>) -> Self {
        self.group_id = group_id;
        self
    }
}

impl RuntimeHandle {
    pub(crate) async fn upsert_scoped_parameter(
        &self,
        scope: &RecordScope,
        update: ScopedParameterUpdate,
    ) -> bool {
        let transaction = UpsertScopedParameter {
            scope: scope.clone(),
            key: update.key,
            value: update.value,
            source: update.source,
            metadata: update.metadata,
            group_id: update.group_id,
            unit: update.unit,
        };
        let mut projected = (*self.state_read()).clone();
        if !apply_upsert_scoped_parameter(&mut projected, transaction.clone()) {
            return false;
        }
        self.commit(transaction).await.is_ok()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
