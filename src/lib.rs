#[cfg(feature = "parking_lot")]
use parking_lot::{Mutex, MutexGuard};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LockResult, PoisonError};
#[cfg(not(feature = "parking_lot"))]
use std::sync::{Mutex, MutexGuard};

pub type OnTheFlySwapInner<T> = Box<T>;

pub struct OnTheFlySwap<T: ?Sized> {
    inner: Arc<Mutex<Option<OnTheFlySwapInner<T>>>>,
}

impl<T: ?Sized> Clone for OnTheFlySwap<T> {
    fn clone(&self) -> Self {
        OnTheFlySwap {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct MutexGuardRef<'a, T: ?Sized> {
    pub mutex_guard: MutexGuard<'a, Option<Box<T>>>,
}

impl<'a, T: ?Sized> MutexGuardRef<'a, T> {
    pub fn inner(&self) -> Option<&T> {
        match self.mutex_guard.deref() {
            Some(b) => Some(&*b),
            None => None,
        }
    }

    pub fn inner_mut(&mut self) -> Option<&mut T> {
        match self.mutex_guard.deref_mut() {
            Some(b) => Some(&mut *b),
            None => None,
        }
    }

    pub fn inner_box(&mut self) -> Option<&Box<T>> {
        match self.mutex_guard.deref_mut() {
            Some(b) => Some(b),
            None => None,
        }
    }
}

impl<T> OnTheFlySwap<T>
where
    T: ?Sized + Send,
{
    pub fn new(b: Box<T>) -> OnTheFlySwap<T> {
        OnTheFlySwap {
            inner: Arc::new(Mutex::new(Some(b))),
        }
    }

    pub fn new_empty() -> OnTheFlySwap<T> {
        OnTheFlySwap {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn replace_inner(&mut self, replace_by: Option<OnTheFlySwapInner<T>>) {
        if let Some(on_the_fly_inner) = replace_by {
            #[cfg(not(feature = "parking_lot"))]
            self.inner.lock().unwrap().replace(on_the_fly_inner);
            #[cfg(feature = "parking_lot")]
            self.inner.lock().replace(on_the_fly_inner);
        } else {
            #[cfg(not(feature = "parking_lot"))]
            self.inner.lock().unwrap().take();
            #[cfg(feature = "parking_lot")]
            self.inner.lock().take();
        }
    }

    pub fn replace_take(&mut self, _replace_by: Option<OnTheFlySwapInner<T>>) {
        unimplemented!("replace_take not implemented yet")
    }

    #[cfg(not(feature = "parking_lot"))]
    pub fn lock_w(&self) -> LockResult<MutexGuardRef<'_, T>> {
        match self.inner.lock() {
            Ok(mut m) => match m.as_deref_mut() {
                _ => Ok(MutexGuardRef { mutex_guard: m }),
            },
            Err(e) => Err(PoisonError::new(MutexGuardRef {
                mutex_guard: e.into_inner(),
            })),
        }
    }

    pub fn lock(&self) -> MutexGuardRef<'_, T> {
        MutexGuardRef {
            #[cfg(not(feature = "parking_lot"))]
            mutex_guard: self.inner.lock().unwrap(),
            #[cfg(feature = "parking_lot")]
            mutex_guard: self.inner.lock(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace() {
        let mut number_source = OnTheFlySwap::new(Box::new(0));
        let mut numbers = Vec::<u8>::new();
        if let Some(number_source) = number_source.lock().inner() {
            numbers.push(*number_source);
        }
        number_source.replace_inner(Some(Box::new(1)));
        if let Some(number_source) = number_source.lock().inner() {
            numbers.push(*number_source);
        }
        assert_eq!(numbers[0], 0);
        assert_eq!(numbers[1], 1);
    }
}
