use std::alloc::{self, Layout};
use std::ptr;
use std::marker::PhantomData;

pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.add(self.len)))
            }
        }
    }

    pub fn iter(&self) -> VecIter<T> {
        VecIter {
            ptr: self.ptr as *const T,
            len: self.len,
            index: 0,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> VecIterMut<T> {
        VecIterMut {
            ptr: self.ptr,
            len: self.len,
            index: 0,
            _marker: PhantomData,
        }
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity.checked_mul(2).expect("capacity overflow")
        };

        let new_layout = Layout::array::<T>(new_capacity)
            .expect("invalid layout");

        let new_ptr = unsafe { alloc::alloc(new_layout) };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        let new_ptr = new_ptr as *mut T;

        if !self.ptr.is_null() && self.len > 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
                let old_layout = Layout::array::<T>(self.capacity)
                    .expect("invalid old layout");
                alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() && self.capacity > 0 {
            unsafe {
                for i in 0..self.len {
                    ptr::drop_in_place(self.ptr.add(i));
                }

                let layout = Layout::array::<T>(self.capacity)
                    .expect("invalid layout");
                alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

pub struct VecIter<'a, T> {
    ptr: *const T,
    len: usize,
    index: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iterator for VecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let result = unsafe { &*self.ptr.add(self.index) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

pub struct VecIterMut<'a, T> {
    ptr: *mut T,
    len: usize,
    index: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for VecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let result = unsafe { &mut *self.ptr.add(self.index) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<T> Vec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let layout = Layout::array::<T>(capacity)
            .expect("invalid layout");
        
        let ptr = unsafe { alloc::alloc(layout) };
        
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        Self {
            ptr: ptr as *mut T,
            len: 0,
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len, "index out of bounds");

        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            let ptr = self.ptr.add(index);
            ptr::copy(ptr, ptr.add(1), self.len - index);
            ptr::write(ptr, value);
        }

        self.len = self.len.checked_add(1).expect("length overflow");
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        unsafe {
            let ptr = self.ptr.add(index);
            let value = ptr::read(ptr);
            ptr::copy(ptr.add(1), ptr, self.len - index - 1);
            self.len -= 1;
            value
        }
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}
