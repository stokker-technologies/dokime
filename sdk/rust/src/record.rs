/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Record

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::marker::PhantomData;

// Third Party Imports

// Local Imports
use crate::{Message, v1};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Record Contract Names a Message, Independent of Stream Publication.
pub struct Record<T: Message> {
    pub(crate) definition: v1::RecordContract,
    marker: PhantomData<fn() -> T>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Message> Record<T> {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            definition: v1::RecordContract {
                metadata: Some(v1::Metadata {
                    identifier: identifier.into(),
                    label: label.into(),
                    description: String::new(),
                    ..Default::default()
                }),
                message: T::name(),
            },
            marker: PhantomData,
        }
    }
    pub fn identifier(&self) -> &str {
        &self.definition.metadata.as_ref().expect("Record Metadata").identifier
    }
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.definition.metadata.as_mut().expect("Record Metadata").description = description.into();
        self
    }
}

impl<T: Message> Clone for Record<T> {
    fn clone(&self) -> Self {
        Self {
            definition: self.definition.clone(),
            marker: PhantomData,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
