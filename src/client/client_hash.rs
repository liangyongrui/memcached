use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub(crate) fn default_hash_function(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}
