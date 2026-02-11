#![warn(clippy::pedantic)]
#![allow(non_snake_case, clippy::unreadable_literal, clippy::cast_sign_loss)]

use std::thread::spawn;

use windows::Win32::{
    Foundation::HMODULE,
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
};

use crate::{
    fly_hack::fly_logic::FlyHack,
    process_mem::{f_read, scan_for_f32},
};

mod fly_hack;
mod process_mem;

fn find_y_coord_addresses() -> [usize; 4] {
    let addr_list = match scan_for_f32(0xECBD1000, 0xECBD2000, 43.0, 45.0) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("{e}");
            panic!();
        }
    };

    for addr in &addr_list {
        println!("@hit! <{addr:X}> : <{}>", f_read(*addr));
    }

    assert!(addr_list.len() == 4, "Somehow we failed to find the addresses.");

    addr_list.try_into().unwrap()
}

fn print_teto() {
    println!(
        "I used the teto to destroy the teto!\n{}",
        include_str!("../teto.txt")
    );
}

fn main_thread() {
    let address_list = find_y_coord_addresses();

    print_teto();

    FlyHack::new(address_list).main_loop();
}

/// # Safety
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
        spawn(main_thread);
    }
    true
}
