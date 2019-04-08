#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use x11;
use xcb;
use xcb_util::icccm;

mod keys;
mod client;

use client::{Client, WindowType, Geometry};

pub struct Window {
    id: xcb::Window,
    geom: Geometry,
    base_size: (u16, u16),
    min_size: (u16, u16),
    max_size: (u16, u16),
    monitor: u32, //TODO
}

fn into_u16(val: (i32, i32)) -> (u16, u16) {
    (val.0 as u16, val.1 as u16)
}

impl Window {
    pub fn from_id(client: &Client, id: xcb::Window) -> Self {
        let hints = icccm::get_wm_normal_hints(&client.connection, id)
            .get_reply()
            .expect("could not get window hints");

        let geom = client.get_window_geometry(id);
        let base_size = hints.base().map(into_u16).unwrap_or((0, 0));
        let min_size = hints.min_size().map(into_u16).unwrap_or((0, 0));
        let max_size = hints.max_size().map(into_u16).unwrap_or((0, 0));

        Self {
            id,
            geom,
            base_size,
            min_size,
            max_size,
            monitor: 0, //TODO
        }
    }
}

fn main() {
    let mut client = Client::open_connection();
    client.register_redirect();
    client.register_keybinds(keybinds());

    let mut windows = Vec::<Window>::new();

    loop {
        use client::Event::*;
        use keys::Command::*;

        match client.poll() {
            MapRequest(w) => {
                if windows.iter().any(|it| it.id == w) {
                    continue;
                }

                let types = client.get_window_types(w);
                if types.iter().any(|it| *it == WindowType::Dock ||
                    *it == WindowType::Toolbar ||
                    *it == WindowType::Desktop) {
                    xcb::map_window(&client.connection, w);
                    continue;
                }

                let window = Window::from_id(&client, w);
                windows.push(window);
                xcb::map_window(&client.connection, w);
            },
            UnmapNotify(w) => {},
            DestroyNotify(w) => {},
            EnterNotify(w) => {},
            Command(cmd) => match cmd {
                Spawn(to_spawn) => { std::process::Command::new(to_spawn)
                    .spawn().expect("failed to spawn cmd"); },
                CloseWindow => {},
            },
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

