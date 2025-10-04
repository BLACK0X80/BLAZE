use std::alloc::{self, Layout};
use std::ptr;

pub struct BlazeAllocator;

impl BlazeAllocator {
    pub fn allocate(&self, size: usize, align: usize) -> *mut u8 {
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { alloc::alloc(layout) }
    }

    pub fn deallocate(&self, ptr: *mut u8, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { alloc::dealloc(ptr, layout) }
    }

    pub fn reallocate(&self, ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8 {
        let old_layout = Layout::from_size_align(old_size, align).unwrap();
        unsafe { alloc::realloc(ptr, old_layout, new_size) }
    }

    pub fn allocate_zeroed(&self, size: usize, align: usize) -> *mut u8 {
        let layout = Layout::from_size_align(size, align).unwrap();
        unsafe { alloc::alloc_zeroed(layout) }
    }
}

static GLOBAL_ALLOCATOR: BlazeAllocator = BlazeAllocator;

pub fn blaze_alloc(size: usize) -> *mut u8 {
    GLOBAL_ALLOCATOR.allocate(size, std::mem::align_of::<u8>())
}

pub fn blaze_dealloc(ptr: *mut u8, size: usize) {
    GLOBAL_ALLOCATOR.deallocate(ptr, size, std::mem::align_of::<u8>())
}

pub fn blaze_realloc(ptr: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
    GLOBAL_ALLOCATOR.reallocate(ptr, old_size, std::mem::align_of::<u8>(), new_size)
}

