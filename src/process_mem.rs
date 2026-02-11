use std::fmt::Display;
use std::mem;
use windows::Win32::System::Memory::{MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS};
use windows::Win32::System::Memory::{MEMORY_BASIC_INFORMATION, VirtualQuery};
use windows::Win32::System::Memory::{
    PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
};
use windows::Win32::System::Memory::{
    PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY,
};

#[derive(Debug)]
pub enum ScanError {
    InvalidRange,
    NoReadableMemory,
    AccessDenied,
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ScanError::InvalidRange => {
                write!(f, "Invalid memory range (start >= end)")
            }
            ScanError::NoReadableMemory => {
                write!(f, "No readable memory found in range")
            }
            ScanError::AccessDenied => {
                write!(f, "Cannot query memory in this range")
            }
        }
    }
}

impl std::error::Error for ScanError {}

pub type Float = f32;

pub fn scan_memory<T: Copy + PartialOrd + Display>(
    start: usize,
    end: usize,
    min: T,
    max: T,
) -> Result<Vec<usize>, ScanError> {
    if start >= end {
        return Err(ScanError::InvalidRange);
    }

    let mut results = Vec::new();
    let mut addr = start;
    let mut found_any_readable = false;

    while addr < end {
        let mut mbi = MEMORY_BASIC_INFORMATION::default();

        let result = unsafe {
            VirtualQuery(
                Some(addr as *const _),
                &raw mut mbi,
                mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            )
        };

        if result == 0 {
            if !found_any_readable {
                return Err(ScanError::AccessDenied);
            }
            break;
        }

        let region_start = mbi.BaseAddress as usize;
        let region_end = region_start + mbi.RegionSize;

        let is_readable = mbi.State == MEM_COMMIT
            && (mbi.Protect & PAGE_GUARD).0 == 0
            && (mbi.Protect & PAGE_NOACCESS).0 == 0
            && is_page_readable(mbi.Protect);

        if is_readable {
            found_any_readable = true;

            let scan_start = addr.max(region_start);
            let scan_end = end.min(region_end);

            let mut scan_addr = scan_start;
            while scan_addr + mem::size_of::<T>() <= scan_end {
                let ptr = scan_addr as *const T;
                let value = unsafe { std::ptr::read_unaligned(ptr) };
                if value >= min && value <= max {
                    results.push(scan_addr);
                }
                scan_addr += 1;
            }
        }

        addr = region_end;
    }

    if !found_any_readable {
        return Err(ScanError::NoReadableMemory);
    }

    Ok(results)
}

fn is_page_readable(
    protect: windows::Win32::System::Memory::PAGE_PROTECTION_FLAGS,
) -> bool {
    matches!(
        protect,
        PAGE_READONLY
            | PAGE_READWRITE
            | PAGE_WRITECOPY
            | PAGE_EXECUTE_READ
            | PAGE_EXECUTE_READWRITE
            | PAGE_EXECUTE_WRITECOPY
    )
}

/// Read address into a float
pub fn f_read(addr: usize) -> Float {
    unsafe { *(addr as *const Float) }
}

/// Write float into address
pub fn f_write(addr: usize, value: Float) {
    unsafe { *(addr as *mut Float) = value }
}
