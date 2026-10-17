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
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
// Third Party Imports

// Local Imports
use crate::diagnostics::Diagnostic;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Default)]
pub struct Callbacks {
    tasks: Arc<Mutex<Vec<Arc<dyn CallbackTask>>>>,
    settling: Arc<tokio::sync::Mutex<()>>,
}

/* === Enums === */

/* === Traits === */

pub trait CallbackTask: Send + Sync {
    fn is_settled(&self) -> bool;

    fn cancel(&self);

    fn join(&self) -> Pin<Box<dyn Future<Output = Result<(), Diagnostic>> + Send + '_>>;
}

/* === Implementations === */
impl Callbacks {
    pub fn register(&self, task: Arc<dyn CallbackTask>) {
        let mut tasks = self.tasks.lock().unwrap_or_else(|error| error.into_inner());
        tasks.retain(|task| !task.is_settled());
        tasks.push(task);
    }

    pub(crate) fn cancel(&self) {
        let tasks = self
            .tasks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        for task in tasks {
            if !task.is_settled() {
                task.cancel();
            }
        }
    }

    pub async fn settle(&self) -> Vec<Diagnostic> {
        let _settling = self.settling.lock().await;
        let mut failures = Vec::new();

        loop {
            let tasks = {
                let mut tasks = self.tasks.lock().unwrap_or_else(|error| error.into_inner());
                tasks.retain(|task| !task.is_settled());
                tasks.clone()
            };
            if tasks.is_empty() {
                return failures;
            }

            for task in &tasks {
                task.cancel();
            }
            for task in tasks {
                if let Err(failure) = task.join().await {
                    failures.push(failure);
                }

                self.tasks
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .retain(|owned| !Arc::ptr_eq(owned, &task));
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
