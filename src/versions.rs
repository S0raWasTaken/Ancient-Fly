use std::{
    num::NonZeroUsize,
    ops::{Add, Range},
    thread::sleep,
    time::Duration,
};

use crate::process_mem::{check_address, is_memory_range_readable};

/// To find the second and fourth address,
/// we sum the last address + 0xC, always.
/// to find the third, we need to sum
/// one of these numbers, depending on
/// the version of Minecraft.
#[repr(usize)]
#[derive(Debug)]
pub enum Version {
    RD132211 = 0xF8,
    RD132328 = 0x1070,
    RD160052 = 0x100,
}

#[allow(clippy::enum_glob_use)]
use Version::*;

impl Add<Version> for usize {
    type Output = usize;

    fn add(self, rhs: Version) -> Self::Output {
        self + rhs as usize
    }
}

fn retry<F, T, E>(mut fun: F, times: NonZeroUsize) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let times = times.get();
    let mut last = None;

    for _ in 0..times {
        match fun() {
            Ok(v) => return Ok(v),
            Err(e) => last = Some(e),
        }
    }

    Err(last.unwrap())
}

const NOTFOUND: &str = "Version and base address not found.\n\
Are you running a supported minecraft version?";

const UNREADABLE_MEMORY: &str = "Memory range is unreadable";

fn try_rd13x(
    range: Range<usize>,
    version: Version,
) -> Result<(usize, Version), String> {
    if !is_memory_range_readable(range.clone()) {
        return Err(UNREADABLE_MEMORY.to_string());
    }

    for addr in range {
        if unsafe { check_address(addr, 44.61, 44.63) } {
            return Ok((addr, version));
        }
    }
    Err(NOTFOUND.to_string())
}

fn try_rd160052() -> Result<(usize, Version), String> {
    if !is_memory_range_readable(0x701500000..0x701850000) {
        return Err(UNREADABLE_MEMORY.to_string());
    }

    for addr in [0x70203C5E0, 0x7018328E0, 0x7015A0A10, 0x701832960] {
        if unsafe { check_address(addr, 40.0, 54.0) } {
            return Ok((addr, RD160052));
        }
    }
    Err(NOTFOUND.to_string())
}

const MAX_RETRIES: NonZeroUsize = NonZeroUsize::new(5).unwrap();
pub fn find_version_and_base_addr() -> Result<(usize, Version), String> {
    retry(
        || {
            try_rd13x(0xECBD1000..0xECBD2000, RD132211)
                .or_else(|_| try_rd13x(0x701500000..0x701850000, RD132328))
                .or_else(|_| try_rd160052())
                .inspect_err(|_| sleep(Duration::from_secs(1)))
        },
        MAX_RETRIES,
    )
}
