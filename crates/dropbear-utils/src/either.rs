#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct Dirty<T> {
    value: T,
    dirty: bool,
}

impl<T> Dirty<T> {
    /// Creates a new [`Dirty`] of type [`T`]. Marks clean on initial creation.
    pub fn new(value: T) -> Self {
        Self { value, dirty: false }
    }

    /// Creates a new [`Dirty`] of type [`T`]. Marks dirty on initial creation. 
    pub fn new_dirty(value: T) -> Self {
        Self { value, dirty: true }
    }

    /// Fetches a reference to the value.
    /// 
    /// Does not change the state of the cleanliness
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Sets this to a new value, marking dirty in the process. 
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
    }

    /// Mutates the inner value and marks dirty. 
    pub fn mutate(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.value);
        self.dirty = true;
    }

    /// Returns the dirtiness of the value.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Marks the value as clean.
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Marks the value as dirty. 
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Takes the value and clears the dirty flag if dirty, returning None if clean.
    pub fn take_if_dirty(&mut self) -> Option<&T> {
        if self.dirty {
            self.dirty = false;
            Some(&self.value)
        } else {
            None
        }
    }
}

impl<T: Clone> Dirty<T> {
    pub fn get_clean(&mut self) -> T {
        self.dirty = false;
        self.value.clone()
    }
}

impl<T> std::ops::Deref for Dirty<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Dirty<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.dirty = true;
        &mut self.value
    }
}