use crate::stdlib::prelude::*;
use std::ptr;
use std::alloc::{self, Layout};

pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            ptr: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            Vec::new()
        } else {
            let layout = Layout::array::<T>(capacity).unwrap();
            let ptr = unsafe { alloc::alloc(layout) as *mut T };
            Vec {
                ptr,
                len: 0,
                capacity,
            }
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
            unsafe { Some(ptr::read(self.ptr.add(self.len))) }
        }
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn insert(&mut self, index: usize, element: T) {
        if index > self.len {
            panic!("insertion index out of bounds");
        }

        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::copy(
                self.ptr.add(index),
                self.ptr.add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.add(index), element);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("removal index out of bounds");
        }

        unsafe {
            let result = ptr::read(self.ptr.add(index));
            ptr::copy(
                self.ptr.add(index + 1),
                self.ptr.add(index),
                self.len - index - 1,
            );
            self.len -= 1;
            result
        }
    }

    pub fn swap_remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("swap_remove index out of bounds");
        }

        unsafe {
            let last = ptr::read(self.ptr.add(self.len - 1));
            let hole = self.ptr.add(index);
            self.len -= 1;
            ptr::replace(hole, last)
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }

        while self.len > len {
            self.pop();
        }
    }

    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        if new_len > self.len {
            while self.len < new_len {
                self.push(value.clone());
            }
        } else {
            self.truncate(new_len);
        }
    }

    pub fn extend_from_slice(&mut self, other: &[T])
    where
        T: Clone,
    {
        for item in other {
            self.push(item.clone());
        }
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
        if self.len <= 1 {
            return;
        }
        self.quicksort(0, self.len - 1);
    }

    pub fn binary_search(&self, x: &T) -> Result<usize, usize>
    where
        T: Ord,
    {
        let mut left = 0;
        let mut right = self.len;

        while left < right {
            let mid = left + (right - left) / 2;
            match self.get(mid).unwrap().cmp(x) {
                Ordering::Less => left = mid + 1,
                Ordering::Equal => return Ok(mid),
                Ordering::Greater => right = mid,
            }
        }
        Err(left)
    }

    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|item| item == x)
    }

    pub fn iter(&self) -> VecIter<T> {
        VecIter {
            ptr: self.ptr,
            len: self.len,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> VecIterMut<T> {
        VecIterMut {
            ptr: self.ptr,
            len: self.len,
            index: 0,
        }
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 { 1 } else { self.capacity * 2 };

        let new_layout = Layout::array::<T>(new_capacity).unwrap();
        let new_ptr = unsafe { alloc::alloc(new_layout) as *mut T };

        if !self.ptr.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
                let old_layout = Layout::array::<T>(self.capacity).unwrap();
                alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }

    fn quicksort(&mut self, low: usize, high: usize)
    where
        T: Ord,
    {
        if low < high {
            let pi = self.partition(low, high);
            if pi > 0 {
                self.quicksort(low, pi - 1);
            }
            self.quicksort(pi + 1, high);
        }
    }

    fn partition(&mut self, low: usize, high: usize) -> usize
    where
        T: Ord,
    {
        let mut i = low;
        for j in low..high {
            unsafe {
                if (*self.ptr.add(j)).cmp(&*self.ptr.add(high)) == Ordering::Less {
                    ptr::swap(self.ptr.add(i), self.ptr.add(j));
                    i += 1;
                }
            }
        }
        unsafe {
            ptr::swap(self.ptr.add(i), self.ptr.add(high));
        }
        i
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        self.clear();
        if !self.ptr.is_null() {
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = VecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        VecIntoIter {
            vec: self,
            index: 0,
        }
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut vec = Vec::new();
        for item in iter {
            vec.push(item);
        }
        vec
    }
}

pub struct VecIter<T> {
    ptr: *mut T,
    len: usize,
    index: usize,
}

impl<T> Iterator for VecIter<T> {
    type Item = &'static T;

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

pub struct VecIterMut<T> {
    ptr: *mut T,
    len: usize,
    index: usize,
}

impl<T> Iterator for VecIterMut<T> {
    type Item = &'static mut T;

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

pub struct VecIntoIter<T> {
    vec: Vec<T>,
    index: usize,
}

impl<T> Iterator for VecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.len {
            let result = unsafe { ptr::read(self.vec.ptr.add(self.index)) };
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

