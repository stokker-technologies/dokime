/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Immutable State Sharing With Copy-On-Write Mutation.

/* === Modules === */

mod map;

/* === Re-Exports === */

pub use map::Map;

/* === Imports === */

// Standard Library Imports
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

// Third Party Imports
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq)]
pub struct Shared<T>(Arc<T>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl<T: Serialize> Serialize for Shared<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (**self).serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Shared<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::from)
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Self(Arc::new(T::default()))
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Clone> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.0)
    }
}

impl<'a, T> IntoIterator for &'a Shared<T>
where
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&**self).into_iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut Shared<T>
where
    &'a mut T: IntoIterator,
{
    type Item = <&'a mut T as IntoIterator>::Item;
    type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&mut **self).into_iter()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_shares_all_unchanged_runtime_fields() {
        let state = crate::state::RuntimeState::new(crate::model::identity::new_id());
        let copy = state.clone();
        macro_rules! shared {
            ($($field:ident),+ $(,)?) => { $(assert!(Arc::ptr_eq(&state.$field.0, &copy.$field.0));)+ };
        }
        shared!(
            run_parameters,
            installed_plugins,
            plugin_owners,
            suites,
            record_definitions,
            record_history,
            telemetry_definitions,
            retired_executions,
            run_queue,
            records,
            resources,
            installed_plugin_errors,
            prompt_definitions,
            archived_cycles,
            archived_executions
        );
        let published = crate::queries::RuntimeStateSnapshot::new(Arc::new(copy), 17);
        for _ in 0..128 {
            let reader = published.clone();
            assert!(std::ptr::eq(reader.state(), published.state()));
            assert_eq!(reader.revision(), 17);
        }
        let mut changed = state.clone();
        changed.records.through = 42;
        assert_eq!(state.records.through, 0);
        assert_eq!(changed.records.through, 42);
        assert!(!Arc::ptr_eq(&state.records.0, &changed.records.0));
        assert!(Arc::ptr_eq(&state.suites.0, &changed.suites.0));
        let encoded = serde_json::to_vec(&state).unwrap();
        let restored: crate::state::RuntimeState = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(restored, state);
    }
}
