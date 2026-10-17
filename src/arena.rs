use std::alloc::{Layout, alloc};
use std::mem::{MaybeUninit, transmute};
use std::ptr::copy_nonoverlapping;
use std::slice::{from_raw_parts, from_raw_parts_mut};

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
    pub fn new() -> Arena {
        Arena {  }
    }
    pub fn vec_capacity<T>(&self, capacity: usize) -> VecArena<'_, T> {
        unsafe {
            VecArena { arena: &self, buffer: 
                alloc(Layout::from_size_align(size_of::<T>() * capacity, align_of::<T>()).unwrap()) as *mut T, 
                capacity, 
                len: 0 
            }
        }
    }
}
pub struct VecArena<'a, T> {
    arena: &'a Arena,
    buffer: *mut T,
    capacity: usize,
    len: usize,
}
impl<'a, T> VecArena<'a, T> {
    pub fn push(&mut self, element: T) {
        unsafe {

            if self.capacity == self.len {
                let ptr = alloc(Layout::from_size_align(size_of::<T>() * self.capacity * 2, align_of::<T>()).unwrap()) as *mut T;
                copy_nonoverlapping(self.buffer, ptr, self.len);
                self.buffer = ptr;
                self.capacity *= 2; 
            }

            *self.buffer.add(self.len) = element;
            self.len += 1;
        }
        
    }
    pub fn array(&self) -> &'a [T] {
        unsafe {
            from_raw_parts(self.buffer, self.len)
        }
    }
}
