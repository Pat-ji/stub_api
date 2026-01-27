use std::cell::UnsafeCell;
use std::sync::Once;
use serde::{Deserialize, Serialize};

pub static OFFSETS: OffsetsStorage = OffsetsStorage::new();

#[inline(always)]
pub fn get_offsets() -> &'static Offsets {
    unsafe { &*OFFSETS.data.get() }
}

pub struct OffsetsStorage {
    data: UnsafeCell<Offsets>,
}
unsafe impl Send for OffsetsStorage {}
unsafe impl Sync for OffsetsStorage {}
impl OffsetsStorage {
    const fn new() -> Self {
        Self {
            data: UnsafeCell::new(unsafe { std::mem::zeroed::<Offsets>() }),
        }
    }

    pub fn init(&self, value: Offsets) {
        unsafe { *self.data.get() = value; }
    }
}

#[repr(C)]
#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct Offsets {
}