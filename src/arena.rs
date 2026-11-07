use std::alloc::{Layout, alloc};
use std::fmt::{Debug, Formatter, Result};
use std::mem::{MaybeUninit, transmute};
use std::ptr::{copy_nonoverlapping, read};
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
    pub fn alloc_slice<'a, T: Copy>(&'a self, slice: &[T]) -> &'a mut [T] {
        unsafe {
            let ptr = alloc(Layout::from_size_align(size_of::<T>() * slice.len(), align_of::<T>()).unwrap()) as *mut T;
            copy_nonoverlapping(slice.as_ptr(), ptr, slice.len());

            from_raw_parts_mut(ptr, slice.len())
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
    pub fn last_mut(&mut self) -> Option<&'a mut T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(&mut *self.buffer.add(self.len-1))
            }   
        }
    }
    pub fn last(&self) -> Option<&'a T> {
        if self.len == 0 {
            None
        }
        else {
            unsafe {
                Some(&mut *self.buffer.add(self.len-1))
            }   
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            None
        }
        else {
            unsafe {
                Some(&mut *self.buffer.add(index))
            }
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        unsafe {
            self.len -= 1;
            Some(read(self.buffer.add(self.len)))
        }
    }
}
impl<'a, T: Debug> Debug for VecArena<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("VecArena")
            .field("buffer", &self.array())
            .finish()
    }
}