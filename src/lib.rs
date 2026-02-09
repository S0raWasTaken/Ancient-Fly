#![warn(clippy::pedantic)]
#![allow(non_snake_case, clippy::unreadable_literal, clippy::cast_sign_loss)]

use std::{
    fmt::Display,
    thread::{sleep, spawn},
    time::Duration,
};

use windows::Win32::{
    Foundation::HMODULE,
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
};

use crate::keybinds::{KeyState, KeyStates};

enum State {
    Off,
    Idle,
    Ascending,
    Descending,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            State::Off => "Off",
            State::Idle => "Idle",
            State::Ascending => "Ascending",
            State::Descending => "Descending",
        };
        write!(f, "{state_str}")
    }
}

struct Addresses {
    addresses: [usize; 4],
    saved_values: [f32; 4],
}

impl Addresses {
    pub fn new(addresses: [usize; 4]) -> Self {
        Self { addresses, saved_values: Default::default() }
    }

    pub fn populate_save(&mut self) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            *save = f_read(*addr);
        }
    }

    pub fn sum(&mut self, value: f32) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            *save += value;
            f_write(*addr, *save);
        }
    }

    pub fn keep(&mut self) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            f_write(*addr, *save);
        }
    }
}

mod keybinds;

struct FlyHack {
    addresses: Addresses,
    state: State,
    speed: f32,
    iteration_number: usize,
    key_states: KeyStates,
}

impl FlyHack {
    pub fn new(addresses: Vec<usize>) -> Self {
        Self {
            addresses: Addresses::new(addresses.try_into().unwrap()),
            state: State::Off,
            speed: 0.1,
            iteration_number: 0,
            key_states: KeyStates::default(),
        }
    }

    pub fn main_loop(&mut self) {
        self.addresses.populate_save();
        loop {
            let mul_10_frame = self.iteration_number.is_multiple_of(10);
            self.key_states.update();

            if matches!(self.key_states.f1, KeyState::Pressed) {
                if let State::Off = self.state {
                    self.state = State::Idle;
                } else {
                    self.state = State::Off;
                }
            }

            match self.state {
                State::Idle => {
                    self.idle();
                    if self.iteration_number.is_multiple_of(100) {
                        self.addresses.populate_save();
                    }
                }
                State::Ascending if mul_10_frame => self.ascending(),
                State::Descending if mul_10_frame => self.descending(),
                State::Off if self.iteration_number.is_multiple_of(100) => {
                    self.addresses.populate_save();
                }
                _ => (),
            }

            if self.iteration_number.is_multiple_of(2000) {
                println!("State: {}", self.state);
            }

            self.iteration_number += 1;

            if self.iteration_number >= 10000 {
                self.iteration_number = 0;
            }

            sleep(Duration::from_millis(1));
        }
    }

    fn ascending(&mut self) {
        self.addresses.sum(self.speed);
        if matches!(self.key_states.space, KeyState::Released) {
            self.state = State::Idle;
        }
    }

    fn descending(&mut self) {
        self.addresses.sum(-self.speed);
        if matches!(self.key_states.shift, KeyState::Released) {
            self.state = State::Idle;
        }
    }

    fn idle(&mut self) {
        self.addresses.keep();
        if matches!(
            self.key_states.shift,
            KeyState::Pressed | KeyState::Holding
        ) {
            self.state = State::Descending;
        }

        if matches!(
            self.key_states.space,
            KeyState::Pressed | KeyState::Holding
        ) {
            self.state = State::Ascending;
        }
    }
}

fn main_thread() {
    let addr_list = scan_for_f32(0xECBD1000, 0xECBD2000, 43.0, 45.0);
    for addr in &addr_list {
        println!("@hit! <{addr:X}> : <{}>", f_read(*addr));
    }

    assert!(
        addr_list.len() == 4,
        "Something went terribly wrong with the memory scan. Crashing!"
    );

    println!(
        "I used the teto to destroy the teto!\n{}",
        include_str!("../teto.txt")
    );

    FlyHack::new(addr_list).main_loop();
}

/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    module: HMODULE,
    fdw_reason: u32,
    _: *mut (),
) -> bool {
    if fdw_reason == DLL_PROCESS_ATTACH {
        let res = unsafe { DisableThreadLibraryCalls(module) };
        if res.is_err() {
            return false;
        }
        spawn(main_thread);
    }
    true
}

/// Read address into a float
fn f_read(addr: usize) -> f32 {
    unsafe { *(addr as *const f32) }
}

/// Write float into address
fn f_write(addr: usize, value: f32) {
    unsafe { *(addr as *mut f32) = value }
}

fn scan_for_f32(start: usize, end: usize, min: f32, max: f32) -> Vec<usize> {
    let mut results = Vec::with_capacity(4);
    let mut addr = start;
    while addr + std::mem::size_of::<f32>() <= end {
        unsafe {
            let ptr = addr as *const f32;
            let value = std::ptr::read_unaligned(ptr);
            if value >= min && value <= max {
                results.push(addr);
            }
        }
        addr += std::mem::size_of::<f32>();
    }
    results
}
