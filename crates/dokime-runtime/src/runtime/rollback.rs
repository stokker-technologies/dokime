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
use std::{fmt, sync::Arc};

// Third Party Imports
use tokio::sync::Mutex;

// Local Imports
use super::Runtime;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Receipt {
    owner: Arc<Mutex<Option<Owner>>>,
}

struct Boot {
    runtime: Option<crate::app::HarmosRuntime>,
    storage: crate::storage::Manager,
    local: crate::storage::sqlite::State,
}

/* === Enums === */

enum Owner {
    Runtime(Box<Runtime>),
    Boot(Box<Boot>),
}

/* === Traits === */

/* === Implementations === */

impl Receipt {
    pub(super) fn new(runtime: Runtime) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(Owner::Runtime(Box::new(runtime))))),
        }
    }

    pub(super) fn boot(
        runtime: Option<crate::app::HarmosRuntime>, storage: crate::storage::Manager, local: crate::storage::sqlite::State,
    ) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(Owner::Boot(Box::new(Boot { runtime, storage, local }))))),
        }
    }

    pub async fn retry(&self) -> crate::runtime::Result<()> {
        let mut owner = self.owner.lock().await;
        if let Some(owned) = owner.as_mut() {
            match owned {
                Owner::Runtime(runtime) => {
                    runtime.shutdown().await?;
                }
                Owner::Boot(boot) => {
                    let Boot { runtime, storage, local } = boot.as_mut();
                    if let Some(active) = runtime.as_ref() {
                        active.stop().await?;
                        *runtime = None;
                    }
                    storage.close().await?;
                    local.close().await?;
                }
            }
            *owner = None;
        }
        Ok(())
    }
}

impl fmt::Debug for Receipt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Receipt").finish_non_exhaustive()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
