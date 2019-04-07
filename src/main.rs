#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use x11;
use xcb;

mod keys;
mod client;

use client::Client;

fn main() {
    let mut client = Client::open_connection();
    client.register_redirect();
    client.register_keybinds(keybinds());

    loop {
        use client::Event::*;

        match client.poll() {
            MapRequest(w) => {},
            UnmapNotify(w) => {},
            DestroyNotify(w) => {},
            EnterNotify(w) => {},
            Command(cmd) => {},
        }
    }
}

fn keybinds() -> Vec<(keys::KeyCombo, keys::Command)> {
    use keys::{
        Command::*,
        Key::*,
        Event::*,
    };

    keybinds! {
        (C | Spawn("alacritty"))
        (Q | CloseWindow)
    }
}

