use crate::stdlib::prelude::*;
use crate::stdlib::collections::hashmap::HashMap;

pub struct HashSet<T> {
    map: HashMap<T, ()>,
}

impl<T: Hash + Eq> HashSet<T> {
    pub fn new() -> Self {
        HashSet {
            map: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        HashSet {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.map.remove(value).is_some()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            inner: self.map.keys(),
        }
    }
}

pub struct Iter<T> {
    inner: crate::stdlib::collections::hashmap::Keys<T, ()>,
}

impl<T> Iterator for Iter<T> {
    type Item = &'static T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

