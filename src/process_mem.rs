use std::ops::Range;
use std::ptr::{read_unaligned, write_unaligned};

use windows::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READ,
    PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_GUARD, PAGE_READONLY,
    PAGE_READWRITE, PAGE_WRITECOPY, VirtualQuery,
};

/// # Safety
/// The caller must ensure:
/// - `addr` points to valid, readable memory for at least 4 bytes.
/// - The memory at `addr` is initialized.
pub unsafe fn check_address(addr: usize, min: f32, max: f32) -> bool {
    let value = unsafe { f_read(addr) };
    value >= min && value <= max
}

pub fn is_memory_range_readable(range: Range<usize>) -> bool {
    if range.is_empty() {
        return false;
    }

    let start_addr = range.start;
    let end_addr = range.end;
    let mut current_addr = start_addr;

    let readable_flags = PAGE_READONLY
        | PAGE_READWRITE
        | PAGE_WRITECOPY
        | PAGE_EXECUTE_READ
        | PAGE_EXECUTE_READWRITE
        | PAGE_EXECUTE_WRITECOPY;

    while current_addr < end_addr {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();

        unsafe {
            let result = VirtualQuery(
                Some(current_addr as *const std::ffi::c_void),
                &raw mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );

            if result == 0 {
                return false;
            }
        }

        // Check if this region is committed and readable
        if mbi.State != MEM_COMMIT
            || (mbi.Protect.0 & readable_flags.0) == 0
            || (mbi.Protect.0 & PAGE_GUARD.0) != 0
        {
            return false;
        }

        // Move to the next region
        current_addr =
            (mbi.BaseAddress as usize).saturating_add(mbi.RegionSize);

        // Prevent infinite loops
        if current_addr <= (mbi.BaseAddress as usize) {
            return false;
        }
    }

    true
}

/// Read address into a float
/// # Safety
/// - `addr` must be valid for reading 4 bytes.
/// - The memory at `addr` must be initialized.
pub unsafe fn f_read(addr: usize) -> f32 {
    unsafe { read_unaligned(addr as *const f32) }
}

/// Write float into address
/// # Safety
/// - `addr` must be valid for writing 4 bytes.
/// - The memory at `addr` must be writable.
pub unsafe fn f_write(addr: usize, value: f32) {
    unsafe { write_unaligned(addr as *mut f32, value) }
}
