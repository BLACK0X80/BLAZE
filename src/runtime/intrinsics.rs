// Memory intrinsics
#[no_mangle]
pub unsafe extern "C" fn blaze_memcpy(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    std::ptr::copy_nonoverlapping(src, dest, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn blaze_memset(dest: *mut u8, value: u8, count: usize) -> *mut u8 {
    std::ptr::write_bytes(dest, value, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn blaze_memmove(dest: *mut u8, src: *const u8, count: usize) -> *mut u8 {
    std::ptr::copy(src, dest, count);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn blaze_memcmp(s1: *const u8, s2: *const u8, count: usize) -> i32 {
    for i in 0..count {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as i32) - (b as i32);
        }
    }
    0
}

// String intrinsics
#[no_mangle]
pub unsafe extern "C" fn blaze_strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn blaze_strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn blaze_strncmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let c1 = *s1.add(i);
        let c2 = *s2.add(i);
        if c1 != c2 {
            return (c1 as i32) - (c2 as i32);
        }
        if c1 == 0 {
            return 0;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn blaze_strcpy(dest: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn blaze_strncpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        let c = *src.add(i);
        *dest.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    // Pad with zeros if needed
    while i < n {
        *dest.add(i) = 0;
        i += 1;
    }
    dest
}

// Math intrinsics - basic operations
#[no_mangle]
pub extern "C" fn blaze_sqrt_f32(x: f32) -> f32 {
    x.sqrt()
}

#[no_mangle]
pub extern "C" fn blaze_sqrt_f64(x: f64) -> f64 {
    x.sqrt()
}

#[no_mangle]
pub extern "C" fn blaze_sin_f32(x: f32) -> f32 {
    x.sin()
}

#[no_mangle]
pub extern "C" fn blaze_sin_f64(x: f64) -> f64 {
    x.sin()
}

#[no_mangle]
pub extern "C" fn blaze_cos_f32(x: f32) -> f32 {
    x.cos()
}

#[no_mangle]
pub extern "C" fn blaze_cos_f64(x: f64) -> f64 {
    x.cos()
}

#[no_mangle]
pub extern "C" fn blaze_tan_f32(x: f32) -> f32 {
    x.tan()
}

#[no_mangle]
pub extern "C" fn blaze_tan_f64(x: f64) -> f64 {
    x.tan()
}

#[no_mangle]
pub extern "C" fn blaze_pow_f32(x: f32, y: f32) -> f32 {
    x.powf(y)
}

#[no_mangle]
pub extern "C" fn blaze_pow_f64(x: f64, y: f64) -> f64 {
    x.powf(y)
}

#[no_mangle]
pub extern "C" fn blaze_exp_f32(x: f32) -> f32 {
    x.exp()
}

#[no_mangle]
pub extern "C" fn blaze_exp_f64(x: f64) -> f64 {
    x.exp()
}

#[no_mangle]
pub extern "C" fn blaze_log_f32(x: f32) -> f32 {
    x.ln()
}

#[no_mangle]
pub extern "C" fn blaze_log_f64(x: f64) -> f64 {
    x.ln()
}

#[no_mangle]
pub extern "C" fn blaze_log10_f32(x: f32) -> f32 {
    x.log10()
}

#[no_mangle]
pub extern "C" fn blaze_log10_f64(x: f64) -> f64 {
    x.log10()
}

#[no_mangle]
pub extern "C" fn blaze_abs_f32(x: f32) -> f32 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn blaze_abs_f64(x: f64) -> f64 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn blaze_floor_f32(x: f32) -> f32 {
    x.floor()
}

#[no_mangle]
pub extern "C" fn blaze_floor_f64(x: f64) -> f64 {
    x.floor()
}

#[no_mangle]
pub extern "C" fn blaze_ceil_f32(x: f32) -> f32 {
    x.ceil()
}

#[no_mangle]
pub extern "C" fn blaze_ceil_f64(x: f64) -> f64 {
    x.ceil()
}

#[no_mangle]
pub extern "C" fn blaze_round_f32(x: f32) -> f32 {
    x.round()
}

#[no_mangle]
pub extern "C" fn blaze_round_f64(x: f64) -> f64 {
    x.round()
}

#[no_mangle]
pub extern "C" fn blaze_min_f32(x: f32, y: f32) -> f32 {
    x.min(y)
}

#[no_mangle]
pub extern "C" fn blaze_min_f64(x: f64, y: f64) -> f64 {
    x.min(y)
}

#[no_mangle]
pub extern "C" fn blaze_max_f32(x: f32, y: f32) -> f32 {
    x.max(y)
}

#[no_mangle]
pub extern "C" fn blaze_max_f64(x: f64, y: f64) -> f64 {
    x.max(y)
}

// Integer math intrinsics
#[no_mangle]
pub extern "C" fn blaze_abs_i32(x: i32) -> i32 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn blaze_abs_i64(x: i64) -> i64 {
    x.abs()
}

#[no_mangle]
pub extern "C" fn blaze_min_i32(x: i32, y: i32) -> i32 {
    x.min(y)
}

#[no_mangle]
pub extern "C" fn blaze_min_i64(x: i64, y: i64) -> i64 {
    x.min(y)
}

#[no_mangle]
pub extern "C" fn blaze_max_i32(x: i32, y: i32) -> i32 {
    x.max(y)
}

#[no_mangle]
pub extern "C" fn blaze_max_i64(x: i64, y: i64) -> i64 {
    x.max(y)
}

// I/O intrinsics
#[no_mangle]
pub unsafe extern "C" fn blaze_println_str(s: *const u8) {
    if s.is_null() {
        println!();
        return;
    }
    
    let len = blaze_strlen(s);
    let slice = std::slice::from_raw_parts(s, len);
    if let Ok(string) = std::str::from_utf8(slice) {
        println!("{}", string);
    }
}

#[no_mangle]
pub extern "C" fn blaze_println_i32(x: i32) {
    println!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_println_i64(x: i64) {
    println!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_println_f32(x: f32) {
    println!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_println_f64(x: f64) {
    println!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_println_bool(x: bool) {
    println!("{}", x);
}

#[no_mangle]
pub unsafe extern "C" fn blaze_print_str(s: *const u8) {
    if s.is_null() {
        return;
    }
    
    let len = blaze_strlen(s);
    let slice = std::slice::from_raw_parts(s, len);
    if let Ok(string) = std::str::from_utf8(slice) {
        print!("{}", string);
    }
}

#[no_mangle]
pub extern "C" fn blaze_print_i32(x: i32) {
    print!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_print_i64(x: i64) {
    print!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_print_f32(x: f32) {
    print!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_print_f64(x: f64) {
    print!("{}", x);
}

#[no_mangle]
pub extern "C" fn blaze_print_bool(x: bool) {
    print!("{}", x);
}
