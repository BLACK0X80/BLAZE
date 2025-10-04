pub fn size_of<T>() -> usize {
    std::mem::size_of::<T>()
}

pub fn align_of<T>() -> usize {
    std::mem::align_of::<T>()
}

pub fn type_name<T>() -> &'static str {
    std::any::type_name::<T>()
}

pub fn copy_memory(dst: *mut u8, src: *const u8, count: usize) {
    unsafe {
        std::ptr::copy(src, dst, count);
    }
}

pub fn copy_nonoverlapping_memory(dst: *mut u8, src: *const u8, count: usize) {
    unsafe {
        std::ptr::copy_nonoverlapping(src, dst, count);
    }
}

pub fn set_memory(dst: *mut u8, val: u8, count: usize) {
    unsafe {
        std::ptr::write_bytes(dst, val, count);
    }
}

pub fn atomic_load_i32(ptr: *const i32) -> i32 {
    unsafe { std::ptr::read_volatile(ptr) }
}

pub fn atomic_store_i32(ptr: *mut i32, val: i32) {
    unsafe { std::ptr::write_volatile(ptr, val) }
}

pub fn atomic_add_i32(ptr: *mut i32, val: i32) -> i32 {
    unsafe {
        let old = std::ptr::read_volatile(ptr);
        std::ptr::write_volatile(ptr, old + val);
        old
    }
}

pub fn atomic_sub_i32(ptr: *mut i32, val: i32) -> i32 {
    unsafe {
        let old = std::ptr::read_volatile(ptr);
        std::ptr::write_volatile(ptr, old - val);
        old
    }
}

pub fn atomic_compare_exchange_i32(ptr: *mut i32, expected: i32, desired: i32) -> i32 {
    unsafe {
        let current = std::ptr::read_volatile(ptr);
        if current == expected {
            std::ptr::write_volatile(ptr, desired);
        }
        current
    }
}

