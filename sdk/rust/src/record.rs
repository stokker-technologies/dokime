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
use crate::Payload;
use crate::{Metadata, RecordContract};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A Record Contract Names a Message, Independent of Stream Publication.
pub struct Record<T: Payload> {
    pub(crate) definition: RecordContract,
    marker: PhantomData<fn() -> T>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Payload> Record<T> {
    pub fn of<H: Fn() -> Self>(_: H, label: impl Into<String>) -> Self {
        Self::new(crate::authoring::code_name::<H>(), label)
    }

    pub fn new(identifier: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            definition: RecordContract {
                metadata: Some(Metadata {
                    identifier: identifier.into(),
                    label: label.into(),
                    description: String::new(),
                    ..Default::default()
                }),
                message: <T::Message as harmos::Message>::name(),
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

impl<T: Payload> Clone for Record<T> {
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
