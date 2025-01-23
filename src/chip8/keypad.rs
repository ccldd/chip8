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

    /// A key is released when it transitions from Down to Up.
    keys_released: Vec<Key>,
}

impl Keypad {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        for key in Key::iter() {
            keys.insert(key, KeyState::Up);
        }
        Self {
            keys,
            keys_released: Vec::new(),
        }
    }

    pub fn set_key_state(&mut self, key: Key, key_state: KeyState) {
        let old_key_state = self.keys.insert(key, key_state);
        if let Some(old_key_state) = old_key_state {
            if old_key_state == KeyState::Down && key_state == KeyState::Up {
                self.keys_released.push(key);
            }
        }
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

    pub fn get_first_key_released(&self) -> Option<Key> {
        self.keys_released.first().copied()
    }

    pub fn clear_keys_released(&mut self) {
        self.keys_released.clear();
    }
}
