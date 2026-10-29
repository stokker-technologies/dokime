/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::marker::PhantomData;

use crate::model::document::DocumentFilter;
use crate::model::expressions::DataExpression;
use crate::model::expressions::telemetry;
use crate::model::telemetry::TelemetryStream;
use crate::model::values::Value;

pub struct Column<R: ?Sized> {
    field: &'static str,
    row: PhantomData<fn() -> R>,
}

impl<R: ?Sized> Column<R> {
    pub const fn new(field: &'static str) -> Self {
        Self { field, row: PhantomData }
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
        formatter.debug_struct("Column").field("field", &self.field).finish()
    }
}

// A Stream Column Compares Directly, Without An Explicit Conversion.
impl<R: TelemetryStream> Column<R> {
    pub fn eq(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).eq(value)
    }

    pub fn ne(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).ne(value)
    }

    pub fn gt(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).gt(value)
    }

    pub fn gte(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).gte(value)
    }

    pub fn lt(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).lt(value)
    }

    pub fn lte(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).lte(value)
    }

    pub fn contains(self, value: impl Into<Value>) -> DocumentFilter {
        DataExpression::from(self).contains(value)
    }

    pub fn exists(self) -> DocumentFilter {
        DataExpression::from(self).exists()
    }
}

impl<R: TelemetryStream> From<Column<R>> for DataExpression {
    fn from(column: Column<R>) -> Self {
        let mut expression = telemetry(R::ID, column.field);
        expression.label = <R::Row as crate::model::telemetry::TelemetryRow>::columns()
            .into_iter()
            .find(|field| field.metadata.identifier.as_str() == column.field)
            .map(|field| field.metadata.label);
        expression
    }
}
