use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_CAPACITY: usize = 16;
const LOAD_FACTOR_THRESHOLD: f64 = 0.75;

pub struct HashMap<K, V> 
where
    K: Hash + Eq,
{
    buckets: Vec<Vec<Entry<K, V>>>,
    element_count: usize,
}

struct Entry<K, V> {
    key: K,
    value: V,
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self::with_capacity(INITIAL_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }
        Self {
            buckets,
            element_count: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.should_resize() {
            self.resize();
        }

        let bucket_index = self.calculate_bucket_index(&key);
        let bucket = self.buckets.get_mut(bucket_index).unwrap();

        for entry in bucket.iter_mut() {
            if entry.key == key {
                return Some(std::mem::replace(&mut entry.value, value));
            }
        }

        bucket.push(Entry { key, value });
        self.element_count += 1;
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket_index = self.calculate_bucket_index(key);
        let bucket = self.buckets.get(bucket_index)?;

        for entry in bucket.iter() {
            if entry.key == *key {
                return Some(&entry.value);
            }
        }
        None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let bucket_index = self.calculate_bucket_index(key);
        let bucket = self.buckets.get_mut(bucket_index)?;

        for entry in bucket.iter_mut() {
            if entry.key == *key {
                return Some(&mut entry.value);
            }
        }
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let bucket_index = self.calculate_bucket_index(key);
        let bucket = self.buckets.get_mut(bucket_index)?;

        for (index, entry) in bucket.iter().enumerate() {
            if entry.key == *key {
                let removed_entry = bucket.remove(index);
                self.element_count -= 1;
                return Some(removed_entry.value);
            }
        }
        None
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.element_count
    }

    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.element_count = 0;
    }

    fn calculate_bucket_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.buckets.len()
    }

    fn should_resize(&self) -> bool {
        let current_load = self.element_count as f64 / self.buckets.len() as f64;
        current_load >= LOAD_FACTOR_THRESHOLD
    }

    fn resize(&mut self) {
        let old_buckets = std::mem::replace(&mut self.buckets, Vec::new());
        let new_capacity = old_buckets.len() * 2;
        
        self.buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            self.buckets.push(Vec::new());
        }
        self.element_count = 0;

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

