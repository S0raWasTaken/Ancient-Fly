use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LSHIFT, VK_SPACE,
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

#[derive(Default)]
pub enum KeyState {
    Pressed,
    Holding,
    #[default]
    Released,
}

#[derive(Default)]
pub struct KeyStates {
    pub f1: KeyState,
    pub space: KeyState,
    pub shift: KeyState,
}

impl KeyStates {
    pub fn update(&mut self) {
        let f1 = key_state!(windows::Win32::UI::Input::KeyboardAndMouse::VK_F1);
        let shift = key_state!(VK_LSHIFT);
        let space = key_state!(VK_SPACE);

        update_multiple!(
            self.f1 => f1,
            self.shift => shift,
            self.space => space
        );
    }
}
