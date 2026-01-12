//! Custom optional implementations

use serde::{Deserialize, Serialize};

/// An optional value that when replaced can store the old value. 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct HistoricalOption<T> {
    _inner: Option<T>,
    _old: Option<T>,
}

impl<T> HistoricalOption<T> {
    /// Creates a new HistoricalOption with Some value
    pub fn new(value: T) -> Self {
        Self {
            _inner: Some(value),
            _old: None,
        }
    }

    /// Creates a new HistoricalOption with None
    pub fn none() -> Self {
        Self {
            _inner: None,
            _old: None,
        }
    }

    /// Replaces the current value, storing the old one in history
    pub fn replace(&mut self, new_value: Option<T>) {
        self._old = self._inner.take();
        self._inner = new_value;
    }

    /// Swaps the old value with the new one. 
    pub fn swap(&mut self) {
        let old = self._old.take();
        let inner = self._inner.take();
        self._inner = old;
        self._old = inner;
    }

    /// Sets to None, storing the old value in history
    pub fn clear(&mut self) {
        self.replace(None);
    }

    /// Gets a reference to the current value
    pub fn get(&self) -> Option<&T> {
        self._inner.as_ref()
    }

    /// Gets a mutable reference to the current value
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self._inner.as_mut()
    }

    /// Gets a reference to the old/previous value
    pub fn old(&self) -> Option<&T> {
        self._old.as_ref()
    }

    /// Takes the old value, leaving None in its place
    pub fn take_old(&mut self) -> Option<T> {
        self._old.take()
    }

    /// Returns true if the current value is Some
    pub fn is_some(&self) -> bool {
        self._inner.is_some()
    }

    /// Returns true if the current value is None
    pub fn is_none(&self) -> bool {
        self._inner.is_none()
    }

    /// Converts to the inner Option
    pub fn into_inner(self) -> Option<T> {
        self._inner
    }

    /// Turn the option ON. 
    /// If there is a saved history value, restore it.
    /// Otherwise, use the provided default.
    pub fn enable_or(&mut self, default_val: T) {
        if self._inner.is_none() {
            self._inner = Some(self._old.take().unwrap_or(default_val));
        }
    }

    /// Turn the option OFF.
    /// Save the current value into history so it can be restored later.
    pub fn disable(&mut self) {
        if self._inner.is_some() {
            self._old = self._inner.take();
        }
    }
}

impl<T> From<Option<T>> for HistoricalOption<T> {
    fn from(opt: Option<T>) -> Self {
        Self {
            _inner: opt,
            _old: None,
        }
    }
}

impl<T> From<T> for HistoricalOption<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for HistoricalOption<T> {
    fn default() -> Self {
        Self::none()
    }
}