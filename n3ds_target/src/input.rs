use std::sync::atomic::Ordering;

use citra_engine::input::{InputManager, InputState};
use ctru::{
    Error,
    prelude::{Apt, Hid},
};

use crate::panicking::{do_panic, panicking};

pub struct N3dsInputManager {
    hid: Hid,
    apt: Apt,
}
impl N3dsInputManager {
    pub fn new(apt: Apt) -> Result<Self, Error> {
        Ok(Self {
            hid: Hid::new()?,
            apt,
        })
    }
}
impl InputManager for N3dsInputManager {
    fn scan(&mut self) -> InputState {
        self.hid.scan_input();
        InputState {
            keys_held: self.hid.keys_held().bits(),
            keys_down: self.hid.keys_down().bits(),
            keys_up: self.hid.keys_up().bits(),
        }
    }
    fn should_continue(&self) -> bool {
        let is_panicing = panicking.load(Ordering::Relaxed);
        if is_panicing {
            do_panic(true);
        }
        self.apt.main_loop()
    }
}
