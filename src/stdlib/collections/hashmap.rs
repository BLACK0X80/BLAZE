use crate::stdlib::prelude::*;
use crate::stdlib::collections::vec::Vec;

const INITIAL_CAPACITY: usize = 16;
const LOAD_FACTOR: f64 = 0.75;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<Entry<K, V>>>,
    len: usize,
}

struct Entry<K, V> {
    key: K,
    value: V,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap::with_capacity(INITIAL_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }
        HashMap { buckets, len: 0 }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.len >= (self.buckets.len() as f64 * LOAD_FACTOR) as usize {
            self.resize();
        }

        let hash = self.hash(&key);
        let bucket_index = hash % self.buckets.len();
        let bucket = &mut self.buckets.get_mut(bucket_index).unwrap();

        for entry in bucket.iter_mut() {
            if entry.key == key {
                let old_value = std::mem::replace(&mut entry.value, value);
                return Some(old_value);
            }
        }

        bucket.push(Entry { key, value });
        self.len += 1;
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        let bucket_index = hash % self.buckets.len();
        let bucket = self.buckets.get(bucket_index)?;

        for entry in bucket.iter() {
            if entry.key == *key {
                return Some(&entry.value);
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(key);
        let bucket_index = hash % self.buckets.len();
        let bucket = self.buckets.get_mut(bucket_index)?;

        for entry in bucket.iter_mut() {
            if entry.key == *key {
                return Some(&mut entry.value);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);
        let bucket_index = hash % self.buckets.len();
        let bucket = self.buckets.get_mut(bucket_index)?;

        for (i, entry) in bucket.iter().enumerate() {
            if entry.key == *key {
                let removed_entry = bucket.swap_remove(i);
                self.len -= 1;
                return Some(removed_entry.value);
            }
        }
        None
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.len = 0;
    }

    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            iter: self.iter(),
        }
    }

    pub fn values(&self) -> Values<K, V> {
        Values {
            iter: self.iter(),
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            buckets: &self.buckets,
            bucket_index: 0,
            entry_index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            buckets: &mut self.buckets,
            bucket_index: 0,
            entry_index: 0,
        }
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn resize(&mut self) {
        let old_buckets = std::mem::replace(&mut self.buckets, Vec::new());
        let new_capacity = old_buckets.len() * 2;
        
        self.buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            self.buckets.push(Vec::new());
        }
        self.len = 0;

        for bucket in old_buckets {
            for entry in bucket {
                self.insert(entry.key, entry.value);
            }
        }
    }
}

impl<K, V> Default for HashMap<K, V>
where
    K: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, K, V> {
    buckets: &'a Vec<Vec<Entry<K, V>>>,
    bucket_index: usize,
    entry_index: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_index >= self.buckets.len() {
                return None;
            }

            let bucket = &self.buckets[self.bucket_index];
            if self.entry_index < bucket.len() {
                let entry = &bucket[self.entry_index];
                self.entry_index += 1;
                return Some((&entry.key, &entry.value));
            }

            self.bucket_index += 1;
            self.entry_index = 0;
        }
    }
}

pub struct IterMut<'a, K, V> {
    buckets: &'a mut Vec<Vec<Entry<K, V>>>,
    bucket_index: usize,
    entry_index: usize,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket_index >= self.buckets.len() {
                return None;
            }

            if self.entry_index < self.buckets[self.bucket_index].len() {
                let entry = &mut self.buckets[self.bucket_index][self.entry_index];
                self.entry_index += 1;
                return Some((&entry.key, &mut entry.value));
            }

            self.bucket_index += 1;
            self.entry_index = 0;
        }
    }
}

pub struct Keys<K, V> {
    iter: Iter<'static, K, V>,
}

impl<K, V> Iterator for Keys<K, V> {
    type Item = &'static K;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, _)| k)
    }
}

pub struct Values<K, V> {
    iter: Iter<'static, K, V>,
}

impl<K, V> Iterator for Values<K, V> {
    type Item = &'static V;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_, v)| v)
    }
}

pub struct DefaultHasher {
    state: u64,
}

impl DefaultHasher {
    pub fn new() -> Self {
        DefaultHasher { state: 0 }
    }
}

impl Hasher for DefaultHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state = self.state.wrapping_mul(31).wrapping_add(byte as u64);
        }
    }
}

impl Hash for i32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(*self);
    }
}

impl Hash for u32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(*self);
    }
}

impl Hash for i64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i64(*self);
    }
}

impl Hash for u64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(*self);
    }
}

impl Hash for String {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
    }
}

impl Hash for str {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
    }
}

