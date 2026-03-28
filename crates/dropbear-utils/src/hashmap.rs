use std::collections::HashMap;

/// A wrapper for a [HashMap] that iterates its generation for each access. It is able to check
/// for any stale values, and remove them in [StaleTracker::remove_stale].
///
/// # Types
/// * `K` - The key type, must implement `Eq` and `Hash`
/// * `V` - The value type
///
/// # Examples
///
/// ```
/// let mut tracker = dropbear_utils::StaleTracker::new();
///
/// // insert some values
/// tracker.insert("session_1", "user_data");
/// tracker.insert("session_2", "other_data");
///
/// // tick/update
/// // current generation is 0
/// tracker.tick();
///
/// // access session_1 to keep it fresh
/// // session_1 is now at 0
/// tracker.get(&"session_1");
///
/// // tick once more
/// // current generation is now 1
/// tracker.tick();
///
/// // remove entries older than 0 generations
/// // session_2 hasn't been accessed, therefore it will be removed
/// let removed = tracker.remove_stale(1);
/// assert_eq!(removed, vec!["session_2"]);
/// assert_eq!(tracker.len(), 1);
/// ```
pub struct StaleTracker<K, V> {
    map: HashMap<K, (V, usize)>, // (value, last_access_generation)
    current_generation: usize,
}

impl<K: Eq + std::hash::Hash, V> StaleTracker<K, V> {
    /// Creates a new empty `StaleTracker`.
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let tracker: StaleTracker<String, i32> = StaleTracker::new();
    /// ```
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            current_generation: 0,
        }
    }

    /// Inserts a key-value pair into the tracker.
    ///
    /// The entry is marked as accessed at the current generation. If the key already
    /// exists, the old value is replaced and its access time is reset to the current generation.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to associate with the key
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// tracker.insert("key", 42);
    /// ```
    pub fn insert(&mut self, key: K, value: V) {
        self.map.insert(key, (value, self.current_generation));
    }

    /// Gets a reference to the value associated with the key.
    ///
    /// This method marks the entry as accessed at the current generation, preventing it
    /// from being considered stale. Returns `None` if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(&V)` if the key exists
    /// * `None` if the key doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// tracker.insert("key", 42);
    ///
    /// assert_eq!(tracker.get(&"key"), Some(&42));
    /// assert_eq!(tracker.get(&"missing"), None);
    /// ```
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.map.get_mut(key).map(|(value, generation)| {
            *generation = self.current_generation;
            &*value
        })
    }

    /// Gets a mutable reference to the value associated with the key.
    ///
    /// This method marks the entry as accessed at the current generation. Returns `None`
    /// if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(&mut V)` if the key exists
    /// * `None` if the key doesn't exist
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// tracker.insert("counter", 0);
    ///
    /// if let Some(value) = tracker.get_mut(&"counter") {
    ///     *value += 1;
    /// }
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.map.get_mut(key).map(|(value, generation)| {
            *generation = self.current_generation;
            value
        })
    }

    /// Advances the generation counter by one.
    ///
    /// Call this method periodically (e.g., once per frame, once per request, etc.) to
    /// mark a new time period. Entries that aren't accessed after calling `tick()` will
    /// age and eventually become stale.
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// tracker.insert("key", 42);
    ///
    /// // iterate
    /// tracker.tick();
    /// tracker.tick();
    ///
    /// // "key" is now 2 generations old (hasn't been accessed since insertion)
    /// ```
    pub fn tick(&mut self) {
        self.current_generation += 1;
    }

    /// Removes and returns all entries that haven't been accessed within `max_age` generations.
    ///
    /// An entry is considered stale if `(current_generation - last_access_generation) > max_age`.
    ///
    /// # Arguments
    ///
    /// * `max_age` - Maximum number of generations an entry can go without access before removal
    ///
    /// # Returns
    ///
    /// A vector of keys that were removed
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    ///
    /// tracker.insert("fresh", 1);
    /// tracker.insert("stale", 2);
    ///
    /// tracker.get(&"fresh"); // access this one
    /// tracker.tick();        // advance generation
    /// tracker.tick();        // advance again
    ///
    /// // "stale" hasn't been accessed in 2 generations
    /// let removed = tracker.remove_stale(1);
    /// assert!(removed.contains(&"stale"));
    /// assert!(!removed.contains(&"fresh"));
    /// ```
    pub fn remove_stale(&mut self, max_age: usize) -> Vec<K>
    where
        K: Clone,
    {
        let current = self.current_generation;
        let stale_keys: Vec<K> = self
            .map
            .iter()
            .filter(|(_, (_, generation))| current - generation > max_age)
            .map(|(k, _)| k.clone())
            .collect();

        for key in &stale_keys {
            self.map.remove(key);
        }

        stale_keys
    }

    /// Returns the number of entries currently in the tracker.
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// assert_eq!(tracker.len(), 0);
    ///
    /// tracker.insert("key", 42);
    /// assert_eq!(tracker.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the tracker contains no entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let tracker: StaleTracker<String, i32> = StaleTracker::new();
    /// assert!(tracker.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the current generation number.
    ///
    /// This can be useful for debugging or understanding how much time has passed.
    ///
    /// # Examples
    ///
    /// ```
    /// use dropbear_utils::StaleTracker;
    /// let mut tracker = StaleTracker::new();
    /// assert_eq!(tracker.current_generation(), 0);
    ///
    /// tracker.tick();
    /// assert_eq!(tracker.current_generation(), 1);
    /// ```
    pub fn current_generation(&self) -> usize {
        self.current_generation
    }

    /// Iterates over key/value pairs without updating access generation.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map
            .iter()
            .map(|(key, (value, _generation))| (key, value))
    }
}

impl<K: Eq + std::hash::Hash, V> Default for StaleTracker<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
