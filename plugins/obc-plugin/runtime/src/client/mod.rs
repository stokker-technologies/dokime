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
use crate::{
    Error,
    app::{Handle, State},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct Client {
    state: State,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub fn new(handle: Handle) -> Self {
        Self {
            state: handle.state.state(),
        }
    }

    pub async fn ensure_ready(&self) -> Result<(), Error> {
        if self.stopped().await? {
            Err(Error::Stopped)
        } else {
            Ok(())
        }
    }

    pub async fn stopped(&self) -> Result<bool, Error> {
        Ok(self.state.read(|state| state.stopped).await?.value)
    }

    pub fn state(&self) -> &State {
        &self.state
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
