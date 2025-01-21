use std::collections::HashMap;

use num_enum::{FromPrimitive, IntoPrimitive};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive, IntoPrimitive, EnumIter, Hash)]
#[repr(u8)]
pub enum Key {
    #[num_enum(default)]
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyState {
    Down,
    Up,
}

pub struct Keypad {
    keys: HashMap<Key, KeyState>,
}

impl Keypad {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        for key in Key::iter() {
            keys.insert(key, KeyState::Up);
        }
        Self { keys }
    }

    pub fn set_key_state(&mut self, key: Key, key_state: KeyState) {
        self.keys.insert(key, key_state);
    }

    pub fn get_key_state(&self, key: Key) -> KeyState {
        *self.keys.get(&key).unwrap()
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.get_key_state(key) == KeyState::Down
    }

    pub fn is_key_up(&self, key: Key) -> bool {
        self.get_key_state(key) == KeyState::Up
    }

    pub fn get_key_pressed(&self) -> Option<Key> {
        let key = self
            .keys
            .iter()
            .find(|(_, &key_state)| key_state == KeyState::Down);

        if let Some((&key, _)) = key {
            Some(key)
        } else {
            None
        }
    }
}
