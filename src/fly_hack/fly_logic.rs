use std::{fmt::Display, thread::sleep, time::Duration};

use crate::{
    fly_hack::{
        addresses::Addresses,
        keybinds::{KeyState, KeyStates},
    },
    process_mem::Float,
};

#[derive(Default, Clone, Copy)]
enum SpaceStateMachine {
    #[default]
    Entry,
    Pressed,
    PressedAndReleased,
    DoublePressed,
}

impl SpaceStateMachine {
    pub fn update_state(&mut self, keybind: KeyState) {
        *self = match (&self, keybind) {
            (
                SpaceStateMachine::Entry,
                KeyState::Pressed | KeyState::Holding,
            ) => Self::Pressed,
            (SpaceStateMachine::Pressed, KeyState::Released) => {
                Self::PressedAndReleased
            }
            (
                SpaceStateMachine::PressedAndReleased,
                KeyState::Pressed | KeyState::Holding,
            ) => Self::DoublePressed,
            (SpaceStateMachine::DoublePressed, _) => Self::Entry,
            _ => *self,
        };
    }
}

pub static mut STATE: State = State::Off;

#[derive(Clone, Copy)]
pub enum State {
    Off,
    Idle,
    Ascending,
    Descending,
}

impl State {
    pub fn off(&mut self) {
        *self = Self::Off;
        self.update_static();
    }

    pub fn idle(&mut self) {
        *self = Self::Idle;
        self.update_static();
    }

    pub fn ascending(&mut self) {
        *self = Self::Ascending;
        self.update_static();
    }

    pub fn descending(&mut self) {
        *self = Self::Descending;
        self.update_static();
    }

    pub fn toggle(&mut self) {
        if matches!(self, State::Off) {
            self.idle();
        } else {
            self.off();
        }
        self.update_static();
    }

    fn update_static(self) {
        unsafe { STATE = self };
    }
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

pub struct FlyHack {
    addresses: Addresses,
    state: State,
    speed: Float,
    iteration_number: usize,
    key_states: KeyStates,
    space_state_machine: SpaceStateMachine,
    cooldown: usize,
}

impl FlyHack {
    pub fn new(addresses: [usize; 4]) -> Self {
        Self {
            addresses: Addresses::new(addresses),
            state: State::Off,
            speed: 0.1,
            iteration_number: 0,
            key_states: KeyStates::default(),
            space_state_machine: SpaceStateMachine::default(),
            cooldown: 0,
        }
    }

    pub fn main_loop(&mut self) {
        self.addresses.populate_save();
        let mut space_start = self.iteration_number;
        loop {
            let mul_10_frame = self.iteration_number.is_multiple_of(10);
            self.key_states.update();

            self.double_tap_space_bar_detector(&mut space_start);

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

            self.iteration_number += 1;

            if self.iteration_number >= 10000 {
                self.iteration_number = 0;
            }

            sleep(Duration::from_millis(1));
        }
    }

    fn double_tap_space_bar_detector(&mut self, space_start: &mut usize) {
        self.cooldown = self.cooldown.saturating_sub(1);
        if self.cooldown != 0 {
            return;
        }

        if matches!(self.space_state_machine, SpaceStateMachine::Entry) {
            *space_start = self.iteration_number;
        }

        if !matches!(self.space_state_machine, SpaceStateMachine::Entry)
            && self.iteration_number - *space_start > 200
        {
            self.space_state_machine = SpaceStateMachine::Entry;
        }

        self.space_state_machine.update_state(self.key_states.space);

        if matches!(self.space_state_machine, SpaceStateMachine::DoublePressed)
        {
            self.state.toggle();
            self.cooldown = 200;
        }
    }

    fn ascending(&mut self) {
        self.addresses.sum(self.speed);
        if matches!(self.key_states.space, KeyState::Released) {
            self.state.idle();
        }
    }

    fn descending(&mut self) {
        self.addresses.sum(-self.speed);
        if matches!(self.key_states.shift, KeyState::Released) {
            self.state.idle();
        }
    }

    fn idle(&mut self) {
        self.addresses.keep();
        if matches!(
            self.key_states.shift,
            KeyState::Pressed | KeyState::Holding
        ) {
            self.state.descending();
        }

        if matches!(
            self.key_states.space,
            KeyState::Pressed | KeyState::Holding
        ) {
            self.state.ascending();
        }
    }
}
