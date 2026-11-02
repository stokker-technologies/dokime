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
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};

// Third Party Imports

// Local Imports
use crate::RuntimeHandle;
use crate::domains::ActionDispatcher;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Resources {
    pub(crate) dispatchers: Arc<RwLock<BTreeMap<Identifier, Arc<dyn ActionDispatcher>>>>,
    pub(crate) unavailable: Arc<RwLock<BTreeSet<String>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Resources {
    pub(crate) fn new() -> Self {
        Self {
            dispatchers: Arc::new(RwLock::new(BTreeMap::new())),
            unavailable: Arc::new(RwLock::new(BTreeSet::new())),
        }
    }

    pub(crate) fn from_runtime(runtime: &RuntimeHandle) -> Self {
        Self {
            dispatchers: runtime.action_dispatchers.clone(),
            unavailable: runtime.unavailable_plugins.clone(),
        }
    }
    pub(crate) fn available(&self, source: &str) -> bool {
        !self.unavailable.read().expect("plugin availability lock poisoned").contains(source)
    }
    pub(crate) fn dispatcher(&self, source: &Identifier) -> Option<Arc<dyn ActionDispatcher>> {
        self.dispatchers.read().expect("analysis dispatcher lock poisoned").get(source).cloned()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
