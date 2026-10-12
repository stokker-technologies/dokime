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
use std::marker::PhantomData;
// Third Party Imports

// Local Imports
use crate::expressions::{DataExpression, telemetry};
use crate::telemetry::TelemetryStream;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Column<R: ?Sized> {
    field: &'static str,
    row: PhantomData<fn() -> R>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<R: ?Sized> Column<R> {
    pub const fn new(field: &'static str) -> Self {
        Self {
            field,
            row: PhantomData,
        }
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    pub fn on(self, stream: impl Into<String>) -> DataExpression {
        telemetry(stream, self.field)
    }
}

impl<R: ?Sized> Clone for Column<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: ?Sized> Copy for Column<R> {}

impl<R: ?Sized> std::fmt::Debug for Column<R> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Column")
            .field("field", &self.field)
            .finish()
    }
}

impl<R: TelemetryStream> From<Column<R>> for DataExpression {
    fn from(column: Column<R>) -> Self {
        let mut expression = telemetry(R::ID, column.field);
        expression.label = <R::Row as crate::telemetry::TelemetryRow>::columns()
            .into_iter()
            .find(|field| field.metadata.identifier.as_str() == column.field)
            .map(|field| field.metadata.label);
        expression
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
