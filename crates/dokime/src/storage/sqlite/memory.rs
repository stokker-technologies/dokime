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
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

// Third Party Imports

// Local Imports
use super::Error;

/* === Constants === */
/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Budget {
    state: Mutex<Usage>,
}

struct Usage {
    used: usize,
    ceiling: usize,
}

pub(crate) struct Reservation {
    owner: Arc<Budget>,
    bytes: usize,
}

pub(super) struct Ceiling {
    owner: Arc<Budget>,
    previous: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Budget {
    pub(super) fn new(ceiling: NonZeroUsize) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(Usage {
                used: 0,
                ceiling: ceiling.get(),
            }),
        })
    }

    pub(super) fn restrict(self: &Arc<Self>, ceiling: NonZeroUsize) -> Result<Ceiling, Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ceiling = ceiling.get().min(state.ceiling);
        if state.used > ceiling {
            return Err(Error::Data("retained SQLite memory exceeds query bound"));
        }
        let previous = state.ceiling;
        state.ceiling = ceiling;
        Ok(Ceiling {
            owner: self.clone(),
            previous,
        })
    }

    pub(super) fn reserve_owned(
        self: &Arc<Self>,
        bytes: usize,
        ceiling: usize,
    ) -> Result<Reservation, Error> {
        if !self.reserve_up_to(bytes, ceiling) {
            return Err(Error::Data("owned query state exceeds memory bound"));
        }
        Ok(Reservation {
            owner: self.clone(),
            bytes,
        })
    }

    fn reserve_up_to(&self, bytes: usize, ceiling: usize) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = state
            .used
            .checked_add(bytes)
            .filter(|next| *next <= ceiling.min(state.ceiling));
        let Some(next) = next else {
            return false;
        };
        state.used = next;
        true
    }

    fn release(&self, bytes: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Each reservation owns exactly one release.
        state.used = state
            .used
            .checked_sub(bytes)
            .unwrap_or_else(|| std::process::abort());
    }
}

impl Reservation {
    pub(super) fn shrink(&mut self, bytes: usize) -> Result<(), Error> {
        let released = self.bytes.checked_sub(bytes).ok_or(Error::Data(
            "retained allocation exceeds its construction reservation",
        ))?;
        self.owner.release(released);
        self.bytes = bytes;
        Ok(())
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.owner.release(self.bytes);
    }
}

impl Drop for Ceiling {
    fn drop(&mut self) {
        self.owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ceiling = self.previous;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
