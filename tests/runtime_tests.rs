use blaze_compiler::runtime::*;

#[test]
fn test_memory_pool_allocation() {
    unsafe {
        // Test small allocations that should use pools
        let ptr1 = blaze_alloc(8, 8);
        assert!(!ptr1.is_null(), "8-byte allocation should succeed");
        
        let ptr2 = blaze_alloc(16, 8);
        assert!(!ptr2.is_null(), "16-byte allocation should succeed");
        
        let ptr3 = blaze_alloc(64, 8);
        assert!(!ptr3.is_null(), "64-byte allocation should succeed");
        
        // Deallocate and verify they go back to pool
        blaze_dealloc(ptr1, 8, 8);
        blaze_dealloc(ptr2, 16, 8);
        blaze_dealloc(ptr3, 64, 8);
        
        // Allocate again - should reuse from pool
        let ptr4 = blaze_alloc(8, 8);
        assert!(!ptr4.is_null(), "Reallocation should succeed");
        blaze_dealloc(ptr4, 8, 8);
    }
}

#[test]
fn test_large_allocation() {
    unsafe {
        // Test large allocation that bypasses pools
        let ptr = blaze_alloc(1024, 8);
        assert!(!ptr.is_null(), "Large allocation should succeed");
        blaze_dealloc(ptr, 1024, 8);
    }
}

#[test]
fn test_allocation_stats() {
    let stats_before = get_allocation_stats();
    
    unsafe {
        let ptr = blaze_alloc(100, 8);
        let stats_after = get_allocation_stats();
        
        assert!(stats_after.total_allocated >= stats_before.total_allocated + 100,
                "Total allocated should increase");
        
        blaze_dealloc(ptr, 100, 8);
        let stats_final = get_allocation_stats();
        
        assert!(stats_final.total_deallocated >= stats_after.total_deallocated + 100,
                "Total deallocated should increase");
    }
}

#[test]
fn test_realloc() {
    unsafe {
        let ptr = blaze_alloc(50, 8);
        assert!(!ptr.is_null(), "Initial allocation should succeed");
        
        // Write some data
        *ptr = 42;
        
        // Reallocate to larger size
        let new_ptr = blaze_realloc(ptr, 50, 100, 8);
        assert!(!new_ptr.is_null(), "Reallocation should succeed");
        assert_eq!(*new_ptr, 42, "Data should be preserved");
        
        blaze_dealloc(new_ptr, 100, 8);
    }
}

#[test]
fn test_realloc_to_zero() {
    unsafe {
        let ptr = blaze_alloc(50, 8);
        assert!(!ptr.is_null(), "Initial allocation should succeed");
        
        // Reallocate to zero size (should free)
        let new_ptr = blaze_realloc(ptr, 50, 0, 8);
        assert!(new_ptr.is_null(), "Realloc to zero should return null");
    }
}

#[test]
fn test_realloc_from_null() {
    unsafe {
        // Realloc from null should act like alloc
        let ptr = blaze_realloc(std::ptr::null_mut(), 0, 50, 8);
        assert!(!ptr.is_null(), "Realloc from null should allocate");
        blaze_dealloc(ptr, 50, 8);
    }
}

// Intrinsics tests
#[test]
fn test_memcpy() {
    unsafe {
        let src = [1u8, 2, 3, 4, 5];
        let mut dest = [0u8; 5];
        
        blaze_memcpy(dest.as_mut_ptr(), src.as_ptr(), 5);
        
        assert_eq!(dest, src, "memcpy should copy data correctly");
    }
}

#[test]
fn test_memset() {
    unsafe {
        let mut buffer = [0u8; 10];
        
        blaze_memset(buffer.as_mut_ptr(), 42, 10);
        
        assert!(buffer.iter().all(|&x| x == 42), "memset should set all bytes");
    }
}

#[test]
fn test_memmove_overlapping() {
    unsafe {
        let mut buffer = [1u8, 2, 3, 4, 5, 6, 7, 8];
        
        // Move overlapping region
        blaze_memmove(buffer.as_mut_ptr().add(2), buffer.as_ptr(), 5);
        
        assert_eq!(buffer[2], 1, "memmove should handle overlapping regions");
        assert_eq!(buffer[3], 2, "memmove should handle overlapping regions");
    }
}

#[test]
fn test_memcmp() {
    unsafe {
        let a = [1u8, 2, 3, 4, 5];
        let b = [1u8, 2, 3, 4, 5];
        let c = [1u8, 2, 3, 4, 6];
        
        assert_eq!(blaze_memcmp(a.as_ptr(), b.as_ptr(), 5), 0, "Equal arrays should return 0");
        assert!(blaze_memcmp(a.as_ptr(), c.as_ptr(), 5) < 0, "Different arrays should return non-zero");
    }
}

#[test]
fn test_strlen() {
    unsafe {
        let s = b"hello\0";
        assert_eq!(blaze_strlen(s.as_ptr()), 5, "strlen should return correct length");
        
        let empty = b"\0";
        assert_eq!(blaze_strlen(empty.as_ptr()), 0, "strlen of empty string should be 0");
    }
}

#[test]
fn test_strcmp() {
    unsafe {
        let s1 = b"hello\0";
        let s2 = b"hello\0";
        let s3 = b"world\0";
        
        assert_eq!(blaze_strcmp(s1.as_ptr(), s2.as_ptr()), 0, "Equal strings should return 0");
        assert!(blaze_strcmp(s1.as_ptr(), s3.as_ptr()) != 0, "Different strings should return non-zero");
    }
}

#[test]
fn test_strncmp() {
    unsafe {
        let s1 = b"hello\0";
        let s2 = b"help\0";
        
        assert_eq!(blaze_strncmp(s1.as_ptr(), s2.as_ptr(), 3), 0, "First 3 chars are equal");
        assert!(blaze_strncmp(s1.as_ptr(), s2.as_ptr(), 4) != 0, "4th char differs");
    }
}

#[test]
fn test_strcpy() {
    unsafe {
        let src = b"hello\0";
        let mut dest = [0u8; 10];
        
        blaze_strcpy(dest.as_mut_ptr(), src.as_ptr());
        
        assert_eq!(&dest[..6], src, "strcpy should copy string with null terminator");
    }
}

#[test]
fn test_strncpy() {
    unsafe {
        let src = b"hello\0";
        let mut dest = [0u8; 10];
        
        blaze_strncpy(dest.as_mut_ptr(), src.as_ptr(), 10);
        
        assert_eq!(&dest[..6], src, "strncpy should copy string");
        assert_eq!(dest[6], 0, "strncpy should pad with zeros");
    }
}

// Math intrinsics tests
#[test]
fn test_sqrt() {
    let result_f32 = blaze_sqrt_f32(16.0);
    assert!((result_f32 - 4.0).abs() < 0.001, "sqrt(16) should be 4");
    
    let result_f64 = blaze_sqrt_f64(25.0);
    assert!((result_f64 - 5.0).abs() < 0.001, "sqrt(25) should be 5");
}

#[test]
fn test_trig_functions() {
    use std::f64::consts::PI;
    
    let sin_result = blaze_sin_f64(PI / 2.0);
    assert!((sin_result - 1.0).abs() < 0.001, "sin(π/2) should be 1");
    
    let cos_result = blaze_cos_f64(0.0);
    assert!((cos_result - 1.0).abs() < 0.001, "cos(0) should be 1");
    
    let tan_result = blaze_tan_f64(0.0);
    assert!(tan_result.abs() < 0.001, "tan(0) should be 0");
}

#[test]
fn test_pow() {
    let result_f32 = blaze_pow_f32(2.0, 3.0);
    assert!((result_f32 - 8.0).abs() < 0.001, "2^3 should be 8");
    
    let result_f64 = blaze_pow_f64(3.0, 2.0);
    assert!((result_f64 - 9.0).abs() < 0.001, "3^2 should be 9");
}

#[test]
fn test_exp_log() {
    let exp_result = blaze_exp_f64(1.0);
    assert!((exp_result - std::f64::consts::E).abs() < 0.001, "exp(1) should be e");
    
    let log_result = blaze_log_f64(std::f64::consts::E);
    assert!((log_result - 1.0).abs() < 0.001, "log(e) should be 1");
    
    let log10_result = blaze_log10_f64(100.0);
    assert!((log10_result - 2.0).abs() < 0.001, "log10(100) should be 2");
}

#[test]
fn test_abs() {
    assert_eq!(blaze_abs_f32(-5.5), 5.5, "abs(-5.5) should be 5.5");
    assert_eq!(blaze_abs_f64(-10.0), 10.0, "abs(-10.0) should be 10.0");
    assert_eq!(blaze_abs_i32(-42), 42, "abs(-42) should be 42");
    assert_eq!(blaze_abs_i64(-100), 100, "abs(-100) should be 100");
}

#[test]
fn test_floor_ceil_round() {
    assert_eq!(blaze_floor_f32(3.7), 3.0, "floor(3.7) should be 3");
    assert_eq!(blaze_ceil_f32(3.2), 4.0, "ceil(3.2) should be 4");
    assert_eq!(blaze_round_f32(3.5), 4.0, "round(3.5) should be 4");
    
    assert_eq!(blaze_floor_f64(3.7), 3.0, "floor(3.7) should be 3");
    assert_eq!(blaze_ceil_f64(3.2), 4.0, "ceil(3.2) should be 4");
    assert_eq!(blaze_round_f64(3.5), 4.0, "round(3.5) should be 4");
}

#[test]
fn test_min_max() {
    assert_eq!(blaze_min_f32(3.0, 5.0), 3.0, "min(3, 5) should be 3");
    assert_eq!(blaze_max_f32(3.0, 5.0), 5.0, "max(3, 5) should be 5");
    
    assert_eq!(blaze_min_f64(3.0, 5.0), 3.0, "min(3, 5) should be 3");
    assert_eq!(blaze_max_f64(3.0, 5.0), 5.0, "max(3, 5) should be 5");
    
    assert_eq!(blaze_min_i32(3, 5), 3, "min(3, 5) should be 3");
    assert_eq!(blaze_max_i32(3, 5), 5, "max(3, 5) should be 5");
    
    assert_eq!(blaze_min_i64(3, 5), 3, "min(3, 5) should be 3");
    assert_eq!(blaze_max_i64(3, 5), 5, "max(3, 5) should be 5");
}
