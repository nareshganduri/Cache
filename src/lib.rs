//! Thread-safe and non-thread-safe variants of lazily evaluated caches.

#![deny(missing_docs)]

use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::sync::{RwLock, RwLockReadGuard};

/// a non-thread-safe implementation of a lazily evaluated expression. For a
/// thread-safe variant, use [`AtomicCache`].
/// 
/// [`AtomicCache`]: ./struct.AtomicCache.html
pub struct Cache<T> {
    calc: Box<Fn() -> T>,
    data: RefCell<Option<T>>,
}

impl<T> Cache<T> {
    /// Constructs a new Cache using a boxed closure that lazily evaluates
    /// to the value that will be cached.
    /// ```
    /// # use cache::Cache;
    /// let cache = Cache::new(Box::new(|| 55));
    ///
    /// assert_eq!(*cache.get(), 55);
    /// ```
    pub fn new(calc: Box<Fn() -> T>) -> Self {
        Cache {
            calc,
            data: RefCell::new(None),
        }
    }

    /// gets a reference to the cached value, computing it first if it
    /// does not exist
    /// ```
    /// # use cache::Cache;
    /// let cache = Cache::new(Box::new(|| 55));
    ///
    /// assert_eq!(*cache.get(), 55);
    /// ```
    pub fn get(&self) -> CacheRef<T> {
        if self.data.borrow().is_none() {
            let calc = &self.calc;
            let data = calc();

            self.data.replace(Some(data));
        }

        CacheRef::new(self.data.borrow())
    }
}

/// A non-thread-safe reference to the cached value stored in a [`Cache`].
/// Constructed using the [`get`] method on a [`Cache`].
/// Consists of a thin wrapper around a [`RefCell`] reference.
///
/// [`Cache`]: ./struct.Cache.html
/// [`get`]: ./struct.Cache.html#method.get
/// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
pub struct CacheRef<'a, T>(Ref<'a, Option<T>>);

impl<'a, T> CacheRef<'a, T> {
    fn new(r: Ref<'a, Option<T>>) -> Self {
        CacheRef(r)
    }
}

impl<'a, T> Deref for CacheRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

/// a thread-safe variant of [`Cache`]
///
/// [`Cache`]: ./struct.Cache.html
pub struct AtomicCache<T> {
    calc: Box<Fn() -> T + Send + Sync>,
    data: RwLock<Option<T>>,
}

impl<T> AtomicCache<T> {
    /// Constructs a new AtomicCache using a boxed closure that lazily evaluates
    /// to the value that will be cached.
    /// ```
    /// # use cache::AtomicCache;
    /// let cache = AtomicCache::new(Box::new(|| 55));
    ///
    /// assert_eq!(*cache.get(), 55);
    /// ```
    pub fn new(calc: Box<Fn() -> T + Send + Sync>) -> Self {
        AtomicCache {
            calc,
            data: RwLock::new(None),
        }
    }

    /// gets a reference to the cached value, computing it first if it
    /// does not exist
    /// ```
    /// # use cache::AtomicCache;
    /// let cache = AtomicCache::new(Box::new(|| 55));
    ///
    /// assert_eq!(*cache.get(), 55);
    /// ```
    pub fn get(&self) -> AtomicCacheRef<T> {
        if self.data.read().unwrap().is_none() {
            let calc = &self.calc;
            let data = calc();

            let mut write = self.data.write().unwrap();
            *write = Some(data);
        }

        AtomicCacheRef::new(self.data.read().unwrap())
    }
}

/// A thread-safe reference to the cached value stored in an [`AtomicCache`].
/// Constructed using the [`get`] method on an [`AtomicCache`].
/// Consists of a thin wrapper around a [`RwLockReadGuard`].
///
/// [`AtomicCache`]: ./struct.AtomicCache.html
/// [`get`]: ./struct.AtomicCache.html#method.get
/// [`RwLockReadGuard`]: https://doc.rust-lang.org/std/sync/struct.RwLockReadGuard.html
pub struct AtomicCacheRef<'a, T>(RwLockReadGuard<'a, Option<T>>);

impl<'a, T> AtomicCacheRef<'a, T> {
    fn new(r: RwLockReadGuard<'a, Option<T>>) -> Self {
        AtomicCacheRef(r)
    }
}

impl<'a, T> Deref for AtomicCacheRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[derive(Debug, PartialEq)]
    struct A(usize);

    impl A {
        fn new(x: usize) -> Self {
            A(x)
        }

        fn inner(&self) -> usize {
            self.0
        }
    }

    #[test]
    fn test_atomic() {
        let cache = Arc::new(AtomicCache::new(Box::new(|| A::new(0))));

        for _ in 0..10 {
            let cache = Arc::clone(&cache);

            thread::spawn(move || {
                let value = cache.get();

                assert_eq!(value.inner(), 0);
            });
        }
    }
}
