use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
// Helpers
fn calculate_id<T: Hash>(t: &T) -> u64 {
    let salt: u64 = rand::random();
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.write_u64(salt);
    s.finish()
} 