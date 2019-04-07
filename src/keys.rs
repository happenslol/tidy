use xcb;
use x11::keysym::*;
use std::{os::raw::c_uint, collections::HashMap};

pub type ModMaskCode = c_uint;
pub type KeyCode = c_uint;

#[derive(Debug, Clone)]
pub enum Command {
    Spawn(&'static str),
    CloseWindow,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub mods: ModMaskCode,
    pub key: KeyCode,
    pub event: Event,
}

impl KeyCombo {
    pub fn new(event: Event, mods: &[Mod], key: Key) -> Self {
        let mods = mods.iter().fold(0, |acc, it| acc | **it);
        let key = key as KeyCode;
        Self { mods, key, event }
    }
}

impl std::convert::From<(u32, u32)> for KeyCombo {
    fn from(item: (u32, u32)) -> Self {
        KeyCombo {
            mods: item.0,
            key: item.1,
            event: Event::KeyDown,
        }
    }
}

pub type KeyBinds = HashMap<KeyCombo, Command>;

#[macro_export]
macro_rules! keybinds {
    ( $( $bind:tt )+ ) => {
        {
            use crate::keys::{KeyCombo, Command};

            let binds: Vec<(KeyCombo, Command)> = vec![
                $(
                    keybind!($bind)
                ),*
            ];

            binds
        }
    };
}

#[macro_export]
macro_rules! keybind {
    ( (on $ev:expr => $key:ident | $cmd:expr) ) => {
        (KeyCombo::new($ev, &[], $key), $cmd)
    };

    ( (on $ev:expr => [$( $mod:ident ),*] + $key:ident | $cmd:expr) ) => {
        (KeyCombo::new($ev, &[$($mod),*], $key), $cmd)
    };

    ( ($key:ident | $cmd:expr) ) => {
        (KeyCombo::new($crate::keys::Event::KeyDown, &[], $key), $cmd)
    };

    ( ([$( $mod:ident ),*] + $key:ident | $cmd:expr) ) => {
        (KeyCombo::new($crate::keys::Event::KeyDown, &[$($mod),*], $key), $cmd)
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mod {
    Shift,
    Caps,
    Control,
    Alt,
    Super,
}

impl std::ops::Deref for Mod {
    type Target = ModMaskCode;
    fn deref(&self) -> &ModMaskCode {
        use Mod::*;
        match *self {
            Shift => &xcb::MOD_MASK_SHIFT,
            Caps => &xcb::MOD_MASK_LOCK,
            Control => &xcb::MOD_MASK_CONTROL,
            Alt => &xcb::MOD_MASK_1,
            Super => &xcb::MOD_MASK_4,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    KeyDown,
    KeyUp,
}

pub enum Key {
    Q = XK_q as isize,
    X = XK_x as isize,
    C = XK_c as isize,
    H = XK_h as isize,
    L = XK_l as isize,
}

