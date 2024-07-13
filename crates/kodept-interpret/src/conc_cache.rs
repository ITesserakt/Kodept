use chashmap::{CHashMap, ReadGuard, WriteGuard};
use slotmap::{Key, KeyData};
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
struct Slot<T> {
    value: T,
    version: u32,
}

struct ForeignKey {
    index: u32,
    version: NonZeroU32,
}

pub struct GuardWrapper<'a, V> {
    read_guard: ReadGuard<'a, u32, Slot<V>>,
}

impl<V> Deref for GuardWrapper<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.read_guard.value
    }
}

pub struct MutGuardWrapper<'a, V> {
    write_guard: WriteGuard<'a, u32, Slot<V>>,
}

impl<V> Deref for MutGuardWrapper<'_, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.write_guard.value
    }
}

impl<V> DerefMut for MutGuardWrapper<'_, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write_guard.value
    }
}

#[derive(Debug, Clone)]
pub struct ConcSecSlotMap<K: Key, V> {
    slots: CHashMap<u32, Slot<V>>,
    _k: PhantomData<fn(K) -> K>,
}

impl<K: Key, V> Default for ConcSecSlotMap<K, V> {
    fn default() -> Self {
        Self {
            slots: Default::default(),
            _k: Default::default(),
        }
    }
}

impl<K: Key, V> ConcSecSlotMap<K, V> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            slots: CHashMap::with_capacity(cap),
            _k: Default::default(),
        }
    }

    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn contains_key(&self, key: K) -> bool {
        let kd: ForeignKey = key.data().into();
        let slot = self.slots.get(&kd.index);
        slot.map_or(false, |it| it.version == kd.version.get())
    }

    pub fn insert(&self, key: K, value: V) -> Option<V> {
        if key.is_null() {
            return None;
        }

        let kd: ForeignKey = key.data().into();

        let mut last = None;
        self.slots.alter(kd.index, |it| {
            last = it;
            if let Some(slot) = &last {
                if slot.version == kd.version.get() {
                    return Some(Slot {
                        value,
                        version: slot.version,
                    });
                }

                if is_older_version(kd.version.get(), slot.version) {
                    return None;
                }

                last = None;
                return Some(Slot {
                    value,
                    version: kd.version.get(),
                });
            }
            Some(Slot {
                value,
                version: kd.version.get(),
            })
        });
        last.map(|it| it.value)
    }

    pub fn remove(&self, key: K) -> Option<V> {
        let kd: ForeignKey = key.data().into();

        let mut removed = None;
        self.slots.alter(kd.index, |it| {
            if let Some(slot) = it {
                if slot.version == kd.version.get() {
                    removed = Some(slot.value);
                    return None;
                }
                return Some(slot);
            }
            None
        });
        removed
    }

    pub fn clear(&self) {
        self.slots.clear();
    }

    pub fn get(&self, key: K) -> Option<GuardWrapper<V>> {
        let kd: ForeignKey = key.data().into();
        self.slots
            .get(&kd.index)
            .filter(|it| it.version == kd.version.get())
            .map(|it| GuardWrapper { read_guard: it })
    }

    pub fn get_mut(&self, key: K) -> Option<MutGuardWrapper<V>> {
        let kd: ForeignKey = key.data().into();
        self.slots
            .get_mut(&kd.index)
            .filter(|it| it.version == kd.version.get())
            .map(|it| MutGuardWrapper { write_guard: it })
    }

    pub fn into_iter(self) -> impl Iterator<Item = (K, V)> {
        self.slots.into_iter().map(|it| {
            (
                K::from(KeyData::from(ForeignKey {
                    index: it.0,
                    version: it.1.version.try_into().unwrap(),
                })),
                it.1.value,
            )
        })
    }
}

fn is_older_version(a: u32, b: u32) -> bool {
    let diff = a.wrapping_sub(b);
    diff >= (1 << 31)
}

impl From<KeyData> for ForeignKey {
    #[inline(always)]
    fn from(value: KeyData) -> Self {
        let digits = value.as_ffi();
        let index = (digits & 0xffff_ffff) as u32;
        let version = NonZeroU32::new((digits >> 32) as u32).unwrap_or(NonZeroU32::MIN);
        Self { index, version }
    }
}

impl From<ForeignKey> for KeyData {
    #[inline(always)]
    fn from(value: ForeignKey) -> Self {
        let digits = u64::from(value.version.get()) << 32 | u64::from(value.index);
        Self::from_ffi(digits)
    }
}
