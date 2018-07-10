use failure::Error;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::AsRef;
use std::hash::Hash;
use std::rc::Rc;
use std::result::Result;
use std::time::{Duration, Instant};
// Super basic in-memory key-value cache

#[derive(Clone)]
pub struct CachedObject<T>
where
    T: Clone,
{
    data: T,
    ttl: Option<Duration>,
    created: Instant,
}

impl<T> CachedObject<T>
where
    T: Clone,
{
    pub fn new(data: T, ttl: Option<Duration>) -> Self {
        Self {
            data,
            ttl,
            created: Instant::now(),
        }
    }

    pub fn expired(&self) -> bool {
        match self.ttl {
            Some(ttl) => Instant::now().duration_since(self.created) < ttl,
            None => false,
        }
    }
}

#[derive(Clone)]
pub struct Cache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    cache: Rc<RefCell<HashMap<K, CachedObject<V>>>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn insert(&mut self, key: K, value: V, ttl: Option<Duration>) {
        let value = CachedObject::new(value, ttl);
        self.cache.borrow_mut().insert(key, value);
    }

    pub fn remove(&mut self, key: K) {
        self.cache.borrow_mut().remove(&key);
    }

    pub fn get_or_fetch<T>(&mut self, key: K, ttl: Option<Duration>, f: T) -> Result<V, Error>
    where
        T: FnOnce() -> Result<V, Error>,
    {
        let cache = self.cache.clone();
        if let Some(entry) = cache.borrow_mut().get(&key) {
            if entry.expired() {
                return Ok(entry.data.to_owned());
            }
        }
        let value = f()?;
        self.insert(key, value.clone(), ttl);
        Ok(value)
    }
}
