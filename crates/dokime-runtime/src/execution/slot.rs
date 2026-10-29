/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::{Mutex, PoisonError};

pub struct Slot<T> {
    value: Mutex<Option<T>>,
}

impl<T> Slot<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { value: Mutex::new(None) }
    }

    pub fn set(&self, value: T) -> Option<T> {
        self.locked().replace(value)
    }

    pub fn take(&self) -> Option<T> {
        self.locked().take()
    }

    pub fn is_set(&self) -> bool {
        self.locked().is_some()
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, Option<T>> {
        self.value.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slot").field("set", &self.is_set()).finish_non_exhaustive()
    }
}

impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Self { value: Mutex::new(Some(value)) }
    }
}
