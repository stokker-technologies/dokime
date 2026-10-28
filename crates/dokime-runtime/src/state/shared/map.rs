/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persistent Ordered State Entries; Only The Changed Search Path Is Copied.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};
use std::ops::Index;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

type Link<K, V> = Option<Arc<Node<K, V>>>;

#[derive(Clone, Debug)]
pub struct Map<K, V> {
    root: Link<K, V>,
}

#[derive(Clone, Debug)]
struct Node<K, V> {
    key: Arc<K>,
    value: Arc<V>,
    left: Link<K, V>,
    right: Link<K, V>,
    height: usize,
    len: usize,
}

pub struct Entry<'a, K, V> {
    map: &'a mut Map<K, V>,
    key: K,
}

#[derive(Clone)]
pub struct Iter<'a, K, V> {
    front: Vec<&'a Node<K, V>>,
    back: Vec<&'a Node<K, V>>,
    remaining: usize,
}

pub struct IterMut<'a, K, V> {
    pending: VecDeque<Part<'a, K, V>>,
    remaining: usize,
}

enum Part<'a, K, V> {
    Branch(&'a mut Link<K, V>),
    Value(&'a K, &'a mut Arc<V>),
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self { root: None }
    }
}

impl<K, V> Map<K, V> {
    pub fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.len)
    }
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    pub fn clear(&mut self) {
        self.root = None;
    }
    pub fn iter(&self) -> Iter<'_, K, V> {
        let mut iter = Iter {
            front: Vec::new(),
            back: Vec::new(),
            remaining: self.len(),
        };
        descend(&mut iter.front, self.root.as_deref(), false);
        descend(&mut iter.back, self.root.as_deref(), true);
        iter
    }
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &V> + ExactSizeIterator + Clone
    where
        K: Clone,
        V: Clone,
    {
        self.iter().map(|(_, value)| value)
    }
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = &K> + ExactSizeIterator + Clone
    where
        K: Clone,
        V: Clone,
    {
        self.iter().map(|(key, _)| key)
    }
    pub fn get<Q: Ord + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            current = match key.cmp((*node.key).borrow()) {
                Ordering::Less => node.left.as_deref(),
                Ordering::Greater => node.right.as_deref(),
                Ordering::Equal => return Some(&node.value),
            };
        }
        None
    }
    pub fn contains_key<Q: Ord + ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }
}

impl<K: Ord + Clone, V: Clone> Map<K, V> {
    pub fn get_mut<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        if !self.contains_key(key) {
            return None;
        }
        let mut current = &mut self.root;
        loop {
            let node = Arc::make_mut(current.as_mut()?);
            current = match key.cmp((*node.key).borrow()) {
                Ordering::Less => &mut node.left,
                Ordering::Greater => &mut node.right,
                Ordering::Equal => return Some(Arc::make_mut(&mut node.value)),
            };
        }
    }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        insert(&mut self.root, Arc::new(key), Arc::new(value)).map(Arc::unwrap_or_clone)
    }
    pub fn remove<Q: Ord + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        if !self.contains_key(key) {
            return None;
        }
        remove(&mut self.root, key).map(Arc::unwrap_or_clone)
    }
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        Entry { map: self, key }
    }
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> + ExactSizeIterator {
        self.into_iter().map(|(_, value)| value)
    }
    pub fn retain(&mut self, mut keep: impl FnMut(&K, &mut V) -> bool) {
        let removed: Vec<_> = self.into_iter().filter_map(|(key, value)| (!keep(key, value)).then(|| key.clone())).collect();
        for key in removed {
            self.remove(&key);
        }
    }
}

impl<'a, K: Ord + Clone, V: Clone> Entry<'a, K, V> {
    pub fn or_insert_with(self, value: impl FnOnce() -> V) -> &'a mut V {
        if !self.map.contains_key(&self.key) {
            self.map.insert(self.key.clone(), value());
        }
        self.map.get_mut(&self.key).expect("entry was inserted")
    }
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(V::default)
    }
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.or_insert_with(|| value)
    }
}

impl<K: Ord + Clone, V: Clone> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = Self::default();
        for (key, value) in iter {
            map.insert(key, value);
        }
        map
    }
}

impl<K, V, Q: Ord + ?Sized> Index<&Q> for Map<K, V>
where
    K: Borrow<Q>,
{
    type Output = V;
    fn index(&self, key: &Q) -> &V {
        self.get(key).expect("state key exists")
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for Map<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if let (Some(left), Some(right)) = (&self.root, &other.root)
            && Arc::ptr_eq(left, right)
        {
            return true;
        }
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.front.pop()?;
        descend(&mut self.front, node.right.as_deref(), false);
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let node = self.back.pop()?;
        descend(&mut self.back, node.left.as_deref(), true);
        self.remaining -= 1;
        Some((&node.key, &node.value))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: Clone + Ord, V: Clone> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        let remaining = self.len();
        IterMut {
            pending: VecDeque::from([Part::Branch(&mut self.root)]),
            remaining,
        }
    }
}

impl<'a, K: Clone, V: Clone> IterMut<'a, K, V> {
    fn take(&mut self, reverse: bool) -> Option<(&'a K, &'a mut V)> {
        loop {
            let part = if reverse { self.pending.pop_back()? } else { self.pending.pop_front()? };
            match part {
                Part::Value(key, value) => {
                    self.remaining -= 1;
                    return Some((key, Arc::make_mut(value)));
                }
                Part::Branch(root) => {
                    let Some(root) = root else {
                        continue;
                    };
                    let node = Arc::make_mut(root);
                    let left = Part::Branch(&mut node.left);
                    let value = Part::Value(node.key.as_ref(), &mut node.value);
                    let right = Part::Branch(&mut node.right);
                    if reverse {
                        self.pending.extend([left, value, right]);
                    } else {
                        self.pending.push_front(right);
                        self.pending.push_front(value);
                        self.pending.push_front(left);
                    }
                }
            }
        }
    }
}

impl<'a, K: Clone, V: Clone> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.take(false)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<K: Clone, V: Clone> DoubleEndedIterator for IterMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.take(true)
    }
}

impl<K: Clone, V: Clone> ExactSizeIterator for IterMut<'_, K, V> {}

impl<K: Serialize, V: Serialize> Serialize for Map<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}

impl<'de, K: Ord + Clone + Deserialize<'de>, V: Clone + Deserialize<'de>> Deserialize<'de> for Map<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        BTreeMap::<K, V>::deserialize(deserializer).map(|values| values.into_iter().collect())
    }
}

impl<K, V> Node<K, V> {
    fn update(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
        self.len = 1 + self.left.as_ref().map_or(0, |node| node.len) + self.right.as_ref().map_or(0, |node| node.len);
    }
}

fn height<K, V>(root: &Link<K, V>) -> usize {
    root.as_ref().map_or(0, |node| node.height)
}

fn descend<'a, K, V>(stack: &mut Vec<&'a Node<K, V>>, mut node: Option<&'a Node<K, V>>, reverse: bool) {
    while let Some(current) = node {
        stack.push(current);
        node = if reverse { current.right.as_deref() } else { current.left.as_deref() };
    }
}

fn rotate<K: Clone, V: Clone>(root: &mut Link<K, V>, left: bool) {
    let mut previous = root.take().expect("rotation root exists");
    let node = Arc::make_mut(&mut previous);
    let mut next = if left { node.right.take() } else { node.left.take() }.expect("rotation child exists");
    let child = Arc::make_mut(&mut next);
    if left {
        node.right = child.left.take();
    } else {
        node.left = child.right.take();
    }
    node.update();
    if left {
        child.left = Some(previous);
    } else {
        child.right = Some(previous);
    }
    child.update();
    *root = Some(next);
}

fn balance<K: Clone, V: Clone>(root: &mut Link<K, V>) {
    let Some(node) = root else {
        return;
    };
    let node = Arc::make_mut(node);
    node.update();
    if height(&node.left) > height(&node.right) + 1 {
        let child = node.left.as_ref().expect("heavy left child");
        if height(&child.right) > height(&child.left) {
            rotate(&mut node.left, true);
        }
        rotate(root, false);
    } else if height(&node.right) > height(&node.left) + 1 {
        let child = node.right.as_ref().expect("heavy right child");
        if height(&child.left) > height(&child.right) {
            rotate(&mut node.right, false);
        }
        rotate(root, true);
    }
}

fn insert<K: Ord + Clone, V: Clone>(root: &mut Link<K, V>, key: Arc<K>, value: Arc<V>) -> Option<Arc<V>> {
    let Some(current) = root else {
        *root = Some(Arc::new(Node {
            key,
            value,
            left: None,
            right: None,
            height: 1,
            len: 1,
        }));
        return None;
    };
    let node = Arc::make_mut(current);
    let previous = match key.cmp(&node.key) {
        Ordering::Less => insert(&mut node.left, key, value),
        Ordering::Greater => insert(&mut node.right, key, value),
        Ordering::Equal => return Some(std::mem::replace(&mut node.value, value)),
    };
    balance(root);
    previous
}

fn take_first<K: Clone, V: Clone>(root: &mut Link<K, V>) -> (Arc<K>, Arc<V>) {
    let node = Arc::make_mut(root.as_mut().expect("successor exists"));
    if node.left.is_some() {
        let entry = take_first(&mut node.left);
        balance(root);
        return entry;
    }
    let entry = (Arc::clone(&node.key), Arc::clone(&node.value));
    *root = node.right.take();
    entry
}

fn remove<K: Clone + Borrow<Q>, V: Clone, Q: Ord + ?Sized>(root: &mut Link<K, V>, key: &Q) -> Option<Arc<V>> {
    let node = Arc::make_mut(root.as_mut()?);
    let previous = match key.cmp((*node.key).borrow()) {
        Ordering::Less => remove(&mut node.left, key),
        Ordering::Greater => remove(&mut node.right, key),
        Ordering::Equal => {
            let previous = Arc::clone(&node.value);
            if node.left.is_none() {
                *root = node.right.take();
            } else if node.right.is_none() {
                *root = node.left.take();
            } else {
                (node.key, node.value) = take_first(&mut node.right);
            }
            Some(previous)
        }
    };
    balance(root);
    previous
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    #[derive(Debug)]
    struct Counted {
        value: usize,
        copies: Arc<AtomicUsize>,
    }

    impl Clone for Counted {
        fn clone(&self) -> Self {
            self.copies.fetch_add(1, AtomicOrdering::Relaxed);
            Self {
                value: self.value,
                copies: Arc::clone(&self.copies),
            }
        }
    }

    #[test]
    fn snapshot_copies_only_changed_entries_across_commits_and_queries() {
        let baseline_copies = Arc::new(AtomicUsize::new(0));
        let baseline: BTreeMap<usize, Counted> = (0..4096)
            .map(|key| {
                (
                    key,
                    Counted {
                        value: key,
                        copies: Arc::clone(&baseline_copies),
                    },
                )
            })
            .collect();
        for _ in 0..32 {
            std::hint::black_box(baseline.clone());
        }
        assert_eq!(baseline_copies.load(AtomicOrdering::Relaxed), 131_072);
        let copies = Arc::new(AtomicUsize::new(0));
        let mut state: Map<usize, Counted> = (0..4096).map(|key| (key, Counted { value: key, copies: Arc::clone(&copies) })).collect();
        let original = state.clone();
        let mut revisions = Vec::new();
        for revision in 0..32 {
            revisions.push(Arc::new(state.clone()));
            state.get_mut(&revision).unwrap().value += 1;
            for _ in 0..64 {
                let query = Arc::clone(revisions.last().unwrap());
                assert!(Arc::ptr_eq(&query, revisions.last().unwrap()));
                assert_eq!(query[&revision].value, revision);
            }
        }
        assert_eq!(copies.load(AtomicOrdering::Relaxed), 32);
        assert_eq!(original[&0].value, 0);
        assert_eq!(state[&0].value, 1);
        assert!(std::ptr::eq(&original[&4095], &state[&4095]));
        // An Entry Update Copies Its Search Path, Not The Whole Index.
        fn shared<K, V>(left: &Link<K, V>, right: &Link<K, V>) -> usize {
            match (left, right) {
                (Some(left), Some(right)) if Arc::ptr_eq(left, right) => left.len,
                (Some(left), Some(right)) => shared(&left.left, &right.left) + shared(&left.right, &right.right),
                _ => 0,
            }
        }
        let before = state.clone();
        state.get_mut(&2048).unwrap().value += 1;
        assert!(shared(&before.root, &state.root) >= state.len() - state.root.as_ref().unwrap().height);
        let before = state.clone();
        let count = copies.load(AtomicOrdering::Relaxed);
        state.values_mut().next().unwrap().value += 1;
        assert_eq!(copies.load(AtomicOrdering::Relaxed), count + 1);
        assert_eq!(before[&0].value + 1, state[&0].value);
    }

    #[test]
    fn snapshot_map_matches_ordered_mutations_and_encoding() {
        let mut actual = Map::default();
        let mut expected = BTreeMap::new();
        let mut number = 17_u32;
        for step in 0..4096 {
            number = number.wrapping_mul(1664525).wrapping_add(1013904223);
            let key = (number >> 8) % 257;
            if step % 3 == 0 {
                assert_eq!(actual.remove(&key), expected.remove(&key));
            } else {
                assert_eq!(actual.insert(key, step), expected.insert(key, step));
            }
            assert!(actual.iter().eq(expected.iter()));
            assert_eq!(actual.len(), expected.len());
            fn validate<K, V>(node: &Link<K, V>) -> (usize, usize) {
                let Some(node) = node else {
                    return (0, 0);
                };
                let (left_height, left_len) = validate(&node.left);
                let (right_height, right_len) = validate(&node.right);
                assert!(left_height.abs_diff(right_height) <= 1);
                assert_eq!(node.height, left_height.max(right_height) + 1);
                assert_eq!(node.len, left_len + right_len + 1);
                (node.height, node.len)
            }
            validate(&actual.root);
        }
        let encoded = serde_json::to_vec(&actual).unwrap();
        assert_eq!(encoded, serde_json::to_vec(&expected).unwrap());
        assert_eq!(serde_json::from_slice::<Map<u32, i32>>(&encoded).unwrap(), actual);
        let mut front = actual.iter();
        let mut reference = expected.iter();
        for index in 0..actual.len() {
            if index % 2 == 0 {
                assert_eq!(front.next(), reference.next());
            } else {
                assert_eq!(front.next_back(), reference.next_back());
            }
        }
        assert_eq!(front.next(), None);
        assert_eq!(front.next_back(), None);
        let mut actual_mut = (&mut actual).into_iter();
        let mut expected_mut = expected.iter_mut();
        for index in 0..actual_mut.len() {
            let actual_value = if index % 2 == 0 { actual_mut.next() } else { actual_mut.next_back() };
            let expected_value = if index % 2 == 0 { expected_mut.next() } else { expected_mut.next_back() };
            let (actual_key, actual_value) = actual_value.unwrap();
            let (expected_key, expected_value) = expected_value.unwrap();
            assert_eq!(actual_key, expected_key);
            *actual_value += 1;
            *expected_value += 1;
        }
        assert!(actual.iter().eq(expected.iter()));
    }

    #[test]
    fn snapshot_map_bulk_mutation_preserves_retained_revision() {
        let mut state: Map<usize, usize> = (0..128).map(|key| (key, key)).collect();
        let original = state.clone();
        for value in state.values_mut() {
            *value += 1;
        }
        state.retain(|key, _| key % 2 == 0);
        *state.entry(129).or_default() = 130;
        assert_eq!(state.remove(&0), Some(1));
        assert_eq!(state.len(), 64);
        assert_eq!(original.len(), 128);
        assert!(original.iter().all(|(key, value)| key == value));
        state.clear();
        assert!(state.is_empty());
        assert_eq!(original.len(), 128);
    }
}
