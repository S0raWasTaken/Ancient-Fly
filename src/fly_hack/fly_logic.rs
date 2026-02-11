use std::{fmt::Display, thread::sleep, time::Duration};

use crate::fly_hack::{
    addresses::Addresses,
    keybinds::{KeyState, KeyStates},
};

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

pub struct FlyHack {
    addresses: Addresses,
    state: State,
    speed: f32,
    iteration_number: usize,
    key_states: KeyStates,
}

impl FlyHack {
    pub fn new(addresses: [usize; 4]) -> Self {
        Self {
            addresses: Addresses::new(addresses),
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
