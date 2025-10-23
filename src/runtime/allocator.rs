use std::alloc::{alloc, dealloc, realloc, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::collections::HashMap;
use parking_lot::Mutex;
use std::ptr::NonNull;

static TOTAL_ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static TOTAL_DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static GLOBAL_ALLOCATOR: OnceLock<BlazeAllocator> = OnceLock::new();

// Memory pool for small allocations (8, 16, 32, 64, 128, 256 bytes)
const POOL_SIZES: [usize; 6] = [8, 16, 32, 64, 128, 256];
const POOL_CAPACITY: usize = 64; // Number of blocks per pool

struct MemoryPool {
    blocks: Vec<*mut u8>,
    block_size: usize,
}

impl MemoryPool {
    fn new(block_size: usize) -> Self {
        Self {
            blocks: Vec::with_capacity(POOL_CAPACITY),
            block_size,
        }
    }

    fn allocate(&mut self) -> Option<*mut u8> {
        if let Some(ptr) = self.blocks.pop() {
            Some(ptr)
        } else {
            // Pool is empty, allocate new block
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.block_size, self.block_size.min(8));
                let ptr = alloc(layout);
                if ptr.is_null() {
                    None
                } else {
                    Some(ptr)
                }
            }
        }
    }

    fn deallocate(&mut self, ptr: *mut u8) {
        if self.blocks.len() < POOL_CAPACITY {
            self.blocks.push(ptr);
        } else {
            // Pool is full, actually free the memory
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.block_size, self.block_size.min(8));
                dealloc(ptr, layout);
            }
        }
    }
}

pub struct BlazeAllocator {
    allocations: Mutex<HashMap<usize, AllocationInfo>>,
    pools: Mutex<[MemoryPool; 6]>,
}

#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    align: usize,
    file: Option<String>,
    line: Option<u32>,
}

impl BlazeAllocator {
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(HashMap::new()),
            pools: Mutex::new([
                MemoryPool::new(POOL_SIZES[0]),
                MemoryPool::new(POOL_SIZES[1]),
                MemoryPool::new(POOL_SIZES[2]),
                MemoryPool::new(POOL_SIZES[3]),
                MemoryPool::new(POOL_SIZES[4]),
                MemoryPool::new(POOL_SIZES[5]),
            ]),
        }
    }

    fn find_pool_index(size: usize) -> Option<usize> {
        POOL_SIZES.iter().position(|&pool_size| size <= pool_size)
    }

    pub fn allocate_pooled(&self, size: usize, align: usize) -> Option<*mut u8> {
        // Only use pools for small allocations with standard alignment
        if align <= 8 {
            if let Some(pool_idx) = Self::find_pool_index(size) {
                let mut pools = self.pools.lock();
                if let Some(ptr) = pools[pool_idx].allocate() {
                    return Some(ptr);
                }
            }
        }
        None
    }

    pub fn deallocate_pooled(&self, ptr: *mut u8, size: usize, align: usize) -> bool {
        // Only return to pool for small allocations with standard alignment
        if align <= 8 {
            if let Some(pool_idx) = Self::find_pool_index(size) {
                let mut pools = self.pools.lock();
                pools[pool_idx].deallocate(ptr);
                return true;
            }
        }
        false
    }

    pub fn track_allocation(&self, ptr: *mut u8, size: usize, align: usize, file: Option<String>, line: Option<u32>) {
        if !ptr.is_null() {
            let mut allocations = self.allocations.lock();
            allocations.insert(ptr as usize, AllocationInfo { size, align, file, line });
            TOTAL_ALLOCATED.fetch_add(size, Ordering::SeqCst);
        }
    }

    pub fn track_deallocation(&self, ptr: *mut u8) {
        let mut allocations = self.allocations.lock();
        if let Some(info) = allocations.remove(&(ptr as usize)) {
            TOTAL_DEALLOCATED.fetch_add(info.size, Ordering::SeqCst);
        }
    }

    pub fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocated: TOTAL_ALLOCATED.load(Ordering::SeqCst),
            total_deallocated: TOTAL_DEALLOCATED.load(Ordering::SeqCst),
            current_allocations: self.allocations.lock().len(),
        }
    }

    pub fn check_leaks(&self) -> Vec<AllocationInfo> {
        let allocations = self.allocations.lock();
        allocations.values().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_allocations: usize,
}

// Fast path for common small allocations
#[inline]
fn try_fast_alloc(size: usize, align: usize) -> Option<*mut u8> {
    // Fast path: small allocations with standard alignment
    if align <= 8 && size <= 256 {
        get_global_allocator().allocate_pooled(size, align)
    } else {
        None
    }
}

pub unsafe fn blaze_alloc(size: usize, align: usize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }

    if !align.is_power_of_two() {
        return std::ptr::null_mut();
    }

    // Try fast path first
    if let Some(ptr) = try_fast_alloc(size, align) {
        TOTAL_ALLOCATED.fetch_add(size, Ordering::Relaxed);
        return ptr;
    }

    // Slow path: use system allocator
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return std::ptr::null_mut(),
    };

    let ptr = alloc(layout);
    
    if !ptr.is_null() {
        TOTAL_ALLOCATED.fetch_add(size, Ordering::Relaxed);
    }
    
    ptr
}

pub unsafe fn blaze_dealloc(ptr: *mut u8, size: usize, align: usize) {
    if ptr.is_null() || size == 0 {
        return;
    }

    if !align.is_power_of_two() {
        return;
    }

    TOTAL_DEALLOCATED.fetch_add(size, Ordering::Relaxed);

    // Try fast path: return to pool
    if get_global_allocator().deallocate_pooled(ptr, size, align) {
        return;
    }

    // Slow path: use system deallocator
    let layout = match Layout::from_size_align(size, align) {
        Ok(layout) => layout,
        Err(_) => return,
    };

    dealloc(ptr, layout);
}

pub unsafe fn blaze_realloc(ptr: *mut u8, old_size: usize, new_size: usize, align: usize) -> *mut u8 {
    if new_size == 0 {
        blaze_dealloc(ptr, old_size, align);
        return std::ptr::null_mut();
    }

    if ptr.is_null() {
        return blaze_alloc(new_size, align);
    }

    if !align.is_power_of_two() {
        return std::ptr::null_mut();
    }

    // If both sizes fit in pools, allocate new and copy
    if align <= 8 && old_size <= 256 && new_size <= 256 {
        let new_ptr = blaze_alloc(new_size, align);
        if !new_ptr.is_null() {
            std::ptr::copy_nonoverlapping(ptr, new_ptr, old_size.min(new_size));
            blaze_dealloc(ptr, old_size, align);
        }
        return new_ptr;
    }

    let old_layout = match Layout::from_size_align(old_size, align) {
        Ok(layout) => layout,
        Err(_) => return std::ptr::null_mut(),
    };

    let new_ptr = realloc(ptr, old_layout, new_size);
    
    if !new_ptr.is_null() {
        TOTAL_DEALLOCATED.fetch_add(old_size, Ordering::Relaxed);
        TOTAL_ALLOCATED.fetch_add(new_size, Ordering::Relaxed);
    }
    
    new_ptr
}

pub fn get_global_allocator() -> &'static BlazeAllocator {
    GLOBAL_ALLOCATOR.get_or_init(|| BlazeAllocator::new())
}

pub fn get_allocation_stats() -> AllocationStats {
    let allocator = get_global_allocator();
    allocator.get_stats()
}

pub fn check_memory_leaks() -> Vec<AllocationInfo> {
    let allocator = get_global_allocator();
    allocator.check_leaks()
}

// Check if leak detection is enabled via environment variable
pub fn is_leak_detection_enabled() -> bool {
    std::env::var("BLAZE_LEAK_CHECK")
        .map(|v| v == "1" || v.to_lowercase() == "true" || v.to_lowercase() == "on")
        .unwrap_or(false)
}

// Report memory leaks at program exit
pub fn report_memory_leaks() {
    if !is_leak_detection_enabled() {
        return;
    }

    let leaks = check_memory_leaks();
    if leaks.is_empty() {
        eprintln!("✓ No memory leaks detected");
        return;
    }

    eprintln!("\n⚠️  Memory leak detected!");
    eprintln!("Found {} leaked allocation(s):\n", leaks.len());
    
    for (idx, leak) in leaks.iter().enumerate() {
        eprintln!("Leak #{}: {} bytes (alignment: {})", idx + 1, leak.size, leak.align);
        if let (Some(file), Some(line)) = (&leak.file, leak.line) {
            eprintln!("  Allocated at: {}:{}", file, line);
        } else {
            eprintln!("  Allocated at: <unknown location>");
        }
        eprintln!();
    }
    
    let total_leaked: usize = leaks.iter().map(|l| l.size).sum();
    eprintln!("Total leaked memory: {} bytes", total_leaked);
    eprintln!("\nTo disable leak checking, unset BLAZE_LEAK_CHECK environment variable");
}

// Allocation functions with source location tracking
pub unsafe fn blaze_alloc_tracked(size: usize, align: usize, file: *const u8, line: u32) -> *mut u8 {
    let ptr = blaze_alloc(size, align);
    
    if !ptr.is_null() && is_leak_detection_enabled() {
        let file_str = if !file.is_null() {
            std::ffi::CStr::from_ptr(file as *const i8)
                .to_str()
                .ok()
                .map(|s| s.to_string())
        } else {
            None
        };
        
        get_global_allocator().track_allocation(ptr, size, align, file_str, Some(line));
    }
    
    ptr
}

pub unsafe fn blaze_dealloc_tracked(ptr: *mut u8, size: usize, align: usize) {
    if is_leak_detection_enabled() {
        get_global_allocator().track_deallocation(ptr);
    }
    blaze_dealloc(ptr, size, align);
}
