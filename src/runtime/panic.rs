use std::process;

pub fn blaze_panic(message: &str, file: &str, line: u32, column: u32) -> ! {
    eprintln!("thread 'main' panicked at '{}', {}:{}:{}", message, file, line, column);
    process::abort();
}

pub fn blaze_panic_bounds_check(index: usize, len: usize) -> ! {
    blaze_panic(
        &format!("index out of bounds: the len is {} but the index is {}", len, index),
        "unknown",
        0,
        0,
    );
}

pub fn blaze_panic_divide_by_zero() -> ! {
    blaze_panic("attempt to divide by zero", "unknown", 0, 0);
}

pub fn blaze_panic_overflow(operation: &str) -> ! {
    blaze_panic(
        &format!("attempt to {} with overflow", operation),
        "unknown",
        0,
        0,
    );
}

