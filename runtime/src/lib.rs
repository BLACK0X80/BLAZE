use std::ffi::CStr;
use std::io::{self, Write, Read};
use std::slice;
use std::fs::File;
use std::os::raw::{c_char, c_int};
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr};
use tokio::runtime::{Runtime, Handle};
use std::sync::Once;
use parking_lot::Mutex;
use std::collections::HashMap;

static INIT: Once = Once::new();
static mut RUNTIME: Option<Runtime> = None;

fn get_runtime() -> &'static Runtime {
    unsafe {
        INIT.call_once(|| {
            RUNTIME = Some(Runtime::new().expect("Failed to create Tokio runtime"));
        });
        RUNTIME.as_ref().unwrap()
    }
}

#[no_mangle]
pub extern "C" fn blaze_print(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(s) = std::str::from_utf8(slice) {
            print!("{}", s);
            let _ = io::stdout().flush();
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_eprint(ptr: *const u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(s) = std::str::from_utf8(slice) {
            eprint!("{}", s);
            let _ = io::stderr().flush();
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_read_char() -> u32 {
    let mut buffer = [0u8; 1];
    match io::stdin().read_exact(&mut buffer) {
        Ok(_) => buffer[0] as u32,
        Err(_) => 0xFFFFFFFF,
    }
}

#[no_mangle]
pub extern "C" fn blaze_file_open(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(path) = std::str::from_utf8(slice) {
            match File::open(path) {
                Ok(file) => {
                    let fd = Box::into_raw(Box::new(file)) as c_int;
                    fd
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_file_create(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(path) = std::str::from_utf8(slice) {
            match File::create(path) {
                Ok(file) => {
                    let fd = Box::into_raw(Box::new(file)) as c_int;
                    fd
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_file_read(fd: c_int, ptr: *mut u8, len: usize) -> isize {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let file = &mut *(fd as *mut File);
        let buffer = slice::from_raw_parts_mut(ptr, len);
        
        match file.read(buffer) {
            Ok(n) => n as isize,
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_file_write(fd: c_int, ptr: *const u8, len: usize) -> isize {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let file = &mut *(fd as *mut File);
        let buffer = slice::from_raw_parts(ptr, len);
        
        match file.write(buffer) {
            Ok(n) => n as isize,
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_file_close(fd: c_int) -> c_int {
    unsafe {
        let _ = Box::from_raw(fd as *mut File);
        0
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_bind(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(addr_str) = std::str::from_utf8(slice) {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                match TcpListener::bind(addr) {
                    Ok(listener) => {
                        Box::into_raw(Box::new(listener)) as c_int
                    }
                    Err(_) => -1,
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_accept(
    fd: c_int,
    addr_buf: *mut u8,
    addr_len: *mut usize,
) -> c_int {
    if addr_buf.is_null() || addr_len.is_null() {
        return -1;
    }
    
    unsafe {
        let listener = &*(fd as *const TcpListener);
        
        match listener.accept() {
            Ok((stream, addr)) => {
                let addr_string = addr.to_string();
                let bytes = addr_string.as_bytes();
                let copy_len = bytes.len().min(*addr_len);
                
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    addr_buf,
                    copy_len,
                );
                *addr_len = copy_len;
                
                Box::into_raw(Box::new(stream)) as c_int
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_connect(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(addr_str) = std::str::from_utf8(slice) {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                match TcpStream::connect(addr) {
                    Ok(stream) => {
                        Box::into_raw(Box::new(stream)) as c_int
                    }
                    Err(_) => -1,
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_read(fd: c_int, ptr: *mut u8, len: usize) -> isize {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let stream = &mut *(fd as *mut TcpStream);
        let buffer = slice::from_raw_parts_mut(ptr, len);
        
        match stream.read(buffer) {
            Ok(n) => n as isize,
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_write(fd: c_int, ptr: *const u8, len: usize) -> isize {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let stream = &mut *(fd as *mut TcpStream);
        let buffer = slice::from_raw_parts(ptr, len);
        
        match stream.write(buffer) {
            Ok(n) => n as isize,
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_tcp_shutdown(fd: c_int) -> c_int {
    unsafe {
        let stream = &*(fd as *const TcpStream);
        match stream.shutdown(std::net::Shutdown::Both) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_close(fd: c_int) {
    unsafe {
        let _ = Box::from_raw(fd as *mut TcpStream);
    }
}

#[no_mangle]
pub extern "C" fn blaze_udp_bind(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return -1;
    }
    
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(addr_str) = std::str::from_utf8(slice) {
            if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                match UdpSocket::bind(addr) {
                    Ok(socket) => {
                        Box::into_raw(Box::new(socket)) as c_int
                    }
                    Err(_) => -1,
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_spawn(task_ptr: *mut u8) {
    let runtime = get_runtime();
    
    runtime.spawn(async move {
        unsafe {
            let _task = Box::from_raw(task_ptr);
        }
    });
}

#[no_mangle]
pub extern "C" fn blaze_sleep(duration_ms: u64) -> *mut u8 {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn blaze_yield() -> *mut u8 {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn blaze_await(ptr: *mut u8) {
}

#[no_mangle]
pub extern "C" fn blaze_join(task_id: u64) -> *mut u8 {
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn blaze_panic(ptr: *const u8, len: usize) -> ! {
    unsafe {
        let slice = slice::from_raw_parts(ptr, len);
        if let Ok(s) = std::str::from_utf8(slice) {
            panic!("{}", s);
        } else {
            panic!("BLAZE panic (invalid UTF-8)");
        }
    }
}

#[no_mangle]
pub extern "C" fn blaze_alloc(size: usize, align: usize) -> *mut u8 {
    unsafe {
        std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(size, align))
    }
}

#[no_mangle]
pub extern "C" fn blaze_dealloc(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        std::alloc::dealloc(ptr, std::alloc::Layout::from_size_align_unchecked(size, align));
    }
}

#[no_mangle]
pub extern "C" fn blaze_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8 {
    unsafe {
        std::alloc::realloc(
            ptr,
            std::alloc::Layout::from_size_align_unchecked(old_size, align),
            new_size,
        )
    }
}
