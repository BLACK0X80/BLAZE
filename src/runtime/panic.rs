use std::backtrace::Backtrace;

#[no_mangle]
pub extern "C" fn blaze_panic_c(message: *const u8, file: *const u8, line: u32, column: u32) -> ! {
    unsafe {
        let msg = if !message.is_null() {
            std::ffi::CStr::from_ptr(message as *const i8)
                .to_str()
                .unwrap_or("unknown panic")
        } else {
            "unknown panic"
        };
        
        let file_str = if !file.is_null() {
            std::ffi::CStr::from_ptr(file as *const i8)
                .to_str()
                .unwrap_or("<unknown>")
        } else {
            "<unknown>"
        };
        
        blaze_panic(msg, file_str, line, column);
    }
}

pub fn blaze_panic(message: &str, file: &str, line: u32, column: u32) -> ! {
    eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
    eprintln!("║                      BLAZE PANIC                              ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Thread panicked at {}:{}:{}", file, line, column);
    eprintln!("Message: {}", message);
    eprintln!();
    
    // Print backtrace if enabled
    if std::env::var("RUST_BACKTRACE").is_ok() || std::env::var("BLAZE_BACKTRACE").is_ok() {
        eprintln!("Stack backtrace:");
        let backtrace = Backtrace::force_capture();
        eprintln!("{}", backtrace);
    } else {
        eprintln!("note: run with `BLAZE_BACKTRACE=1` environment variable to display a backtrace");
    }
    
    eprintln!();
    std::process::abort();
}

// Panic with just a message (no location info)
#[no_mangle]
pub extern "C" fn blaze_panic_msg(message: *const u8) -> ! {
    unsafe {
        let msg = if !message.is_null() {
            std::ffi::CStr::from_ptr(message as *const i8)
                .to_str()
                .unwrap_or("unknown panic")
        } else {
            "unknown panic"
        };
        
        blaze_panic(msg, "<unknown>", 0, 0);
    }
}

// Assert function for runtime checks
#[no_mangle]
pub extern "C" fn blaze_assert(condition: bool, message: *const u8, file: *const u8, line: u32) {
    if !condition {
        blaze_panic_c(message, file, line, 0);
    }
}

// Bounds check helper
#[no_mangle]
pub extern "C" fn blaze_bounds_check(index: usize, length: usize, file: *const u8, line: u32) {
    if index >= length {
        let msg = format!("index out of bounds: the len is {} but the index is {}", length, index);
        let msg_cstr = std::ffi::CString::new(msg).unwrap();
        blaze_panic_c(msg_cstr.as_ptr() as *const u8, file, line, 0);
    }
}
