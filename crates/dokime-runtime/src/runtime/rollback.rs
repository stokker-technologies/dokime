/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{fmt, sync::Arc};

use tokio::sync::Mutex;

use super::Runtime;

#[derive(Clone)]
pub struct Receipt {
    owner: Arc<Mutex<Option<Owner>>>,
}

#[derive(Default)]
pub(crate) struct Boot {
    pub(crate) cleanup: Option<harmos::runtime::BootstrapCleanup>,
    pub(crate) storage: Option<crate::storage::Manager>,
    pub(crate) local: Option<crate::storage::sqlite::State>,
}

pub(crate) struct Preparing(pub(crate) Option<Receipt>);

enum Owner {
    Runtime(Box<Runtime>),
    Boot(Arc<std::sync::Mutex<Boot>>),
}

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
            owner: Arc::new(Mutex::new(Some(Owner::Runtime(Box::new(runtime))))),
        }
    }

    pub(crate) fn boot(boot: Arc<std::sync::Mutex<Boot>>) -> Self {
        Self {
            owner: Arc::new(Mutex::new(Some(Owner::Boot(boot)))),
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
