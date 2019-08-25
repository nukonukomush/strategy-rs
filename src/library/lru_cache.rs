use linked_hash_map::LinkedHashMap;
use std::borrow::Borrow;
use std::hash::Hash;

pub struct LRUCache<K, V> {
    capacity: usize,
    linked_hash_map: LinkedHashMap<K, V>,
}

impl<K: Hash + Eq, V> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity,
            linked_hash_map: LinkedHashMap::with_capacity(capacity),
        }
    }

    pub fn get<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.linked_hash_map.get_refresh(k)
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.linked_hash_map.insert(k, v);
        if self.linked_hash_map.len() > self.capacity {
            self.linked_hash_map.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru() {
        let mut cache: LRUCache<usize, usize> = LRUCache::new(3);
        cache.insert(0, 0);
        cache.insert(1, 1);
        cache.insert(2, 2);
        cache.insert(3, 3);

        assert_eq!(cache.get(&0), None);
        assert_eq!(cache.get(&1), Some(&mut 1));
        assert_eq!(cache.get(&2), Some(&mut 2));
        assert_eq!(cache.get(&3), Some(&mut 3));

        cache.insert(4, 4);

        assert_eq!(cache.get(&0), None);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&mut 2));
        assert_eq!(cache.get(&3), Some(&mut 3));
        assert_eq!(cache.get(&4), Some(&mut 4));

        cache.get(&2);
        cache.insert(5, 5);

        assert_eq!(cache.get(&0), None);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some(&mut 2));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&4), Some(&mut 4));
        assert_eq!(cache.get(&5), Some(&mut 5));
    }
}

