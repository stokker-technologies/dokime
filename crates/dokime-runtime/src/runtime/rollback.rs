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
use futures_util::future::BoxFuture;
use tokio::sync::Mutex;

// Local Imports
use super::Runtime;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Receipt {
    owner: Arc<Mutex<Option<Box<dyn Cleanup>>>>,
}

#[derive(Default)]
pub(crate) struct Boot {
    pub(crate) cleanup: Option<harmos::runtime::BootstrapCleanup>,
    pub(crate) storage: Option<crate::storage::Manager>,
    pub(crate) local: Option<crate::storage::sqlite::State>,
}

pub(crate) struct Preparing(pub(crate) Option<Receipt>);

/* === Enums === */

enum Owner {
    Runtime(Box<Runtime>),
    Boot(Arc<std::sync::Mutex<Boot>>),
}

/* === Traits === */

// Errors Retain Cleanup Ownership Without Exposing The Runtime's Auto-Trait Graph.
trait Cleanup: Send {
    fn retry(&mut self) -> BoxFuture<'_, crate::runtime::Result<()>>;
}

/* === Implementations === */

impl Drop for Preparing {
    fn drop(&mut self) {
        if let Some(receipt) = self.0.take() {
            tokio::spawn(async move {
                if let Err(error) = receipt.retry().await {
                    tracing::error!(%error, "cancelled Dokime startup cleanup failed");
                }
            });
        }
    }
}

impl Receipt {
    pub(super) fn new(runtime: Runtime) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(Box::new(Owner::Runtime(Box::new(runtime)))))),
        }
    }

    pub(crate) fn boot(boot: Arc<std::sync::Mutex<Boot>>) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(Box::new(Owner::Boot(boot))))),
        }
    }

    pub async fn retry(&self) -> crate::runtime::Result<()> {
        let mut owner = self.owner.lock().await;
        if let Some(owned) = owner.as_mut() {
            owned.retry().await?;
            *owner = None;
        }
        Ok(())
    }
}

impl Cleanup for Owner {
    fn retry(&mut self) -> BoxFuture<'_, crate::runtime::Result<()>> {
        Box::pin(async move {
            match self {
                Owner::Runtime(runtime) => {
                    runtime.shutdown().await?;
                }
                Owner::Boot(boot) => {
                    let (cleanup, storage, local) = {
                        let boot = boot.lock().expect("startup owners are healthy");
                        (boot.cleanup.clone(), boot.storage.clone(), boot.local.clone())
                    };
                    if let Some(cleanup) = cleanup {
                        cleanup.retry().await?;
                    }
                    if let Some(storage) = storage {
                        storage.close().await?;
                    }
                    if let Some(local) = local {
                        local.close().await?;
                    }
                }
            }
            Ok(())
        })
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
