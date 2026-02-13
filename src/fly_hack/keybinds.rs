use std::process;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LSHIFT, VK_SPACE},
    WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

macro_rules! key_state {
    ($x:expr) => {
        unsafe { GetAsyncKeyState(i32::from($x.0)) as u32 & 0x8000 != 0 }
    };
}

macro_rules! update_multiple {
    ($($key:expr => $new:expr),*) => {
        $(
            if !$new {
                $key = KeyState::Released;
            }

            match $key {
                KeyState::Pressed => if $new {
                    $key = KeyState::Holding;
                },
                KeyState::Released => if $new {
                    $key = KeyState::Pressed;
                }
                KeyState::Holding => (),
            }
        )*
    };
}

#[derive(Default, Clone, Copy)]
pub enum KeyState {
    Pressed,
    Holding,
    #[default]
    Released,
}

#[derive(Default, Clone, Copy)]
pub struct KeyStates {
    pub space: KeyState,
    pub shift: KeyState,
}

impl KeyStates {
    pub fn update(&mut self) {
        if !is_window_focused() {
            self.shift = KeyState::Released;
            self.space = KeyState::Released;
            return;
        }

        let shift = key_state!(VK_LSHIFT);
        let space = key_state!(VK_SPACE);

        update_multiple!(
            self.shift => shift,
            self.space => space
        );
    }
}

fn is_window_focused() -> bool {
    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.0.is_null() {
            return false;
        }

        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(foreground_window, Some(&raw mut window_pid));

        let current_pid = process::id();

        window_pid == current_pid
    }
}
