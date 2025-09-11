use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

pub struct ConcurrentHashMap<K, V> {
    inner: Arc<RwLock<HashMap<K, V>>>,
}

#[allow(dead_code)]
impl<K, V> ConcurrentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
        }
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let mut map = self.inner.write().unwrap();
        map.insert(key, value)
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let map = self.inner.read().unwrap();
        map.get(key).cloned()
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut map = self.inner.write().unwrap();
        map.remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let map = self.inner.read().unwrap();
        map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        let map = self.inner.read().unwrap();
        map.len()
    }

    pub fn is_empty(&self) -> bool {
        let map = self.inner.read().unwrap();
        map.is_empty()
    }

    pub fn clear(&self) {
        let mut map = self.inner.write().unwrap();
        map.clear();
    }

    pub fn keys(&self) -> Vec<K> {
        let map = self.inner.read().unwrap();
        map.keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<V> {
        let map = self.inner.read().unwrap();
        map.values().cloned().collect()
    }

    pub fn insert_or_update<F>(&self, key: K, value: V, update_fn: F) -> V
    where
        F: FnOnce(&V) -> V,
    {
        let mut map = self.inner.write().unwrap();
        match map.get(&key) {
            Some(existing_value) => {
                let new_value = update_fn(existing_value);
                map.insert(key, new_value.clone());
                new_value
            }
            None => {
                map.insert(key, value.clone());
                value
            }
        }
    }

    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<K, V>) -> R,
    {
        let map = self.inner.read().unwrap();
        f(&*map)
    }

    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<K, V>) -> R,
    {
        let mut map = self.inner.write().unwrap();
        f(&mut *map)
    }
}

impl<K, V> Clone for ConcurrentHashMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K, V> Default for ConcurrentHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_operations() {
        let map = ConcurrentHashMap::new();

        // Test insert e get
        assert_eq!(map.insert("key1".to_string(), 42), None);
        assert_eq!(map.get(&"key1".to_string()), Some(42));

        // Test update
        assert_eq!(map.insert("key1".to_string(), 84), Some(42));
        assert_eq!(map.get(&"key1".to_string()), Some(84));

        // Test contains_key
        assert!(map.contains_key(&"key1".to_string()));
        assert!(!map.contains_key(&"key2".to_string()));

        // Test remove
        assert_eq!(map.remove(&"key1".to_string()), Some(84));
        assert_eq!(map.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_concurrent_access() {
        let map = ConcurrentHashMap::new();
        let map_clone = map.clone();

        // Thread writer
        let writer = thread::spawn(move || {
            for i in 0..100 {
                map_clone.insert(i, i * 2);
                thread::sleep(Duration::from_millis(1));
            }
        });

        // Thread readers
        let mut readers = vec![];
        for _ in 0..3 {
            let map_clone = map.clone();
            let reader = thread::spawn(move || {
                for i in 0..100 {
                    let _ = map_clone.get(&i);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            readers.push(reader);
        }

        writer.join().unwrap();
        for reader in readers {
            reader.join().unwrap();
        }

        assert_eq!(map.len(), 100);
    }
}
