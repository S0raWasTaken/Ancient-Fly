#![warn(clippy::pedantic)]
#![allow(non_snake_case, clippy::unreadable_literal, clippy::cast_sign_loss)]

use windows::Win32::{
    Foundation::HMODULE,
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
};

use crate::{
    fly_hack::{fly_logic::FlyHack, gui::install_hook},
    process_mem::f_read,
    versions::find_version_and_base_addr,
};

mod fly_hack;
mod process_mem;
mod versions;

macro_rules! dbg_read {
    ($addr:expr) => {{
        println!("@hit! <{:X}> {}", $addr, unsafe { f_read($addr) });
        $addr
    }};
}

const INNER_DIFF: usize = 0xC;

fn find_y_coord_addresses() -> [usize; 4] {
    let (first_addr, version) = find_version_and_base_addr().unwrap();

    let second_addr = first_addr + INNER_DIFF;
    let third_addr = second_addr + version;
    let fourth_addr = third_addr + INNER_DIFF;

    [
        dbg_read!(first_addr),
        dbg_read!(second_addr),
        dbg_read!(third_addr),
        dbg_read!(fourth_addr),
    ]
}

fn print_teto() {
    println!(
        "I used the teto to destroy the teto!\n{}",
        include_str!("../resources/teto.txt")
    );
}

fn main_thread() {
    let address_list = find_y_coord_addresses();

    print_teto();

    let _ = unsafe { install_hook() }; // Render the GUI

    FlyHack::new(address_list).main_loop();
}

/// # Safety
/// It's the entry point for the DLL.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    module: HMODULE,
    fdw_reason: u32,
    _: *mut (),
) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        if unsafe { DisableThreadLibraryCalls(module) }.is_err() {
            return false;
        }
        let thread = std::thread::Builder::new().name("Main".to_string());
        if let Err(e) = thread.spawn(main_thread) {
            eprintln!("Failed to spawn main thread: {e}");
            return false;
        }
    }
    true
}
