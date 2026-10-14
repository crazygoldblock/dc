use std::{alloc::{Layout, alloc}, mem::{MaybeUninit, transmute}, slice::from_raw_parts_mut};

pub struct Arena {

}
impl Arena {
    pub fn alloc<T>(&self, value: T) -> &mut T {
        unsafe {
            let ptr = alloc(Layout::from_size_align(size_of::<T>(), align_of::<T>()).unwrap()) as *mut T;
            *ptr = value;
            &mut *ptr
        }
    }
    pub fn alloc_array<T: Clone>(&self, value: T, len: usize) -> &mut [T] {
        unsafe {
            let ptr = alloc(Layout::from_size_align(size_of::<T>() * len, align_of::<T>()).unwrap()) as *mut MaybeUninit<T>;
            let array = from_raw_parts_mut(ptr, len);

            for e in array.iter_mut() {
                e.write(value.clone());
            }

            transmute(array)
        }
    }
    pub fn clear(&mut self) {

    }
}