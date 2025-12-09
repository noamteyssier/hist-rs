use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::ops::{AddAssign, Deref};

use bumpalo::Bump;
use hashbrown::HashMap;

/// The hashmap key.
struct Key {
    ptr: usize, // Store pointer as usize to make it Send.
    len: usize,
}

impl Deref for Key {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // SAFETY: not sure
        unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance(self.ptr), self.len) }
    }
}

impl From<&mut [u8]> for Key {
    fn from(value: &mut [u8]) -> Self {
        Self {
            // SAFETY: not sure
            ptr: value.as_ptr().expose_provenance(),
            len: value.len(),
        }
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        *self == *other
    }
}

impl Eq for Key {}

impl Borrow<[u8]> for Key {
    fn borrow(&self) -> &[u8] {
        self.deref()
    }
}

pub trait Unity: Sized {
    fn unity() -> Self;
}

macro_rules! impl_unity_for_int {
    ($int:ty) => {
        impl Unity for $int {
            fn unity() -> Self {
                1
            }
        }
    };
}

impl_unity_for_int!(u8);
impl_unity_for_int!(u16);
impl_unity_for_int!(u32);
impl_unity_for_int!(u64);
impl_unity_for_int!(usize);

/// Bump-allocated HashMap from `[u8]` to `T`.
pub struct BumpBytesMap<T = usize> {
    bump: Bump,
    map: HashMap<Key, T>,
}

impl<T> BumpBytesMap<T> {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            map: HashMap::default(),
        }
    }

    #[cfg(test)]
    pub fn get<'a>(&'a self, key: &[u8]) -> Option<&'a T> {
        self.map.get(key)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &T)> {
        self.map.iter().map(|(key, value)| (key.deref(), value))
    }
}

impl<T: AddAssign + Unity> BumpBytesMap<T> {
    pub fn insert_or_inc(&mut self, key: &[u8]) {
        use hashbrown::hash_map::RawEntryMut;

        match self.map.raw_entry_mut().from_key(key) {
            RawEntryMut::Occupied(mut entry) => *entry.get_mut() += T::unity(),
            RawEntryMut::Vacant(entry) => {
                let owned = self.bump.alloc_slice_copy(key);
                entry.insert(owned.into(), T::unity());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BumpBytesMap;

    #[test]
    fn test_spawn() {
        let handle = std::thread::spawn(move || {
            let mut hm = BumpBytesMap::<u32>::new();
            hm.insert_or_inc(b"foo");
            hm.insert_or_inc(b"bar");
            hm.insert_or_inc(b"foo");
            hm
        });
        let hm = handle.join().unwrap();
        assert_eq!(hm.get(b"foo"), Some(&2));
        assert_eq!(hm.get(b"bar"), Some(&1));
        assert_eq!(hm.len(), 2);
    }
}
