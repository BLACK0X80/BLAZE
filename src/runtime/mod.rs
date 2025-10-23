pub mod allocator;
pub mod intrinsics;
pub mod panic;

pub use allocator::{BlazeAllocator, blaze_alloc, blaze_dealloc, blaze_realloc, blaze_alloc_tracked, blaze_dealloc_tracked};
pub use intrinsics::*;
pub use panic::*;

use std::sync::atomic::{AtomicUsize, Ordering};

static RUNTIME_INITIALIZED: AtomicUsize = AtomicUsize::new(0);

pub fn initialize_runtime() -> anyhow::Result<()> {
    if RUNTIME_INITIALIZED.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
        setup_panic_handler();
        initialize_allocator();
        register_intrinsics();
        setup_exit_handler();
        Ok(())
    } else {
        Err(anyhow::anyhow!("Runtime already initialized"))
    }
}

fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info.location().unwrap();
        let message = panic_info.payload().downcast_ref::<&str>().unwrap_or(&"unknown panic");
        blaze_panic(message, location.file(), location.line(), location.column());
    }));
}

fn initialize_allocator() {
    // Initialize the global allocator
    let _ = allocator::get_global_allocator();
}

fn register_intrinsics() {
}

fn setup_exit_handler() {
    // Register exit handler for leak detection
    extern "C" fn exit_handler() {
        allocator::report_memory_leaks();
    }
    
    unsafe {
        libc::atexit(exit_handler);
    }
}
