use std::collections::HashMap;
use xcb_util::{ewmh, icccm, keysyms::KeySymbols};
use crate::keys::{self, Command, KeyCombo};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowType {
    Desktop,
    Dock,
    Toolbar,
    Menu,
    Utility,
    Splash,
    Dialog,
    DropdownMenu,
    PopupMenu,
    Tooltip,
    Notification,
    Combo,
    Dnd,
    Normal,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WindowState {
    Modal,
    Sticky,
    MaximizedVert,
    MaximizedHorz,
    Shaded,
    SkipTaskbar,
    SkipPager,
    Hidden,
    Fullscreen,
    Above,
    Below,
    DemandsAttention,
}

macro_rules! atoms {
    ( $( $name:ident ),+ ) => {
        #[allow(non_snake_case)]
        pub struct Atoms {
            $(
                pub $name: xcb::Atom,
            )*
        }

        impl Atoms {
            pub fn intern(c: &xcb::Connection) -> Self {
                $(
                    let request = xcb::intern_atom(c, false, stringify!($name));
                    #[allow(non_snake_case)]
                    let $name = request.get_reply().expect("Could not intern atom").atom();
                )*

                Self {$(
                    $name,
                )*}
            }
        }
    };
}

pub type AtomTable<T> = HashMap<xcb::Atom, T>;
atoms![WM_DELETE_WINDOW, WM_PROTOCOLS];

pub struct Client {
    pub connection: ewmh::Connection,

    pub root_window: xcb::Window,
    pub screen: Screen,
    pub screen_idx: i32,

    pub atoms: Atoms,
    pub window_types: AtomTable<WindowType>,
    pub window_states: AtomTable<WindowState>,
    pub keymap: HashMap<keys::KeyCombo, keys::Command>,

    pub focused: Option<xcb::Window>,
    pub screens: Vec<u32>,
}

pub struct Screen {
    idx: i32,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Client {
    pub fn open_connection() -> Self {
        let (connection, screen_idx) = xcb::Connection::connect(None).unwrap();
        let connection = ewmh::Connection::connect(connection)
            .map_err(|(e, _)| e).unwrap();

        let screen = {
            let root = connection.get_setup()
                .roots().nth(screen_idx as usize)
                .ok_or("Invalid screen").unwrap();

            Screen {
                idx: screen_idx.into(),
                x: 0, y: 0,
                width: root.width_in_pixels(),
                height: root.height_in_pixels(),
            }
        };

        let root_window = connection.get_setup()
            .roots().nth(screen_idx as usize)
            .ok_or("Invalid screen").unwrap().root();

        let atoms = Atoms::intern(&connection);

        let mut types = HashMap::new();
        let mut states = HashMap::new();

        types.insert(connection.WM_WINDOW_TYPE_DESKTOP(), WindowType::Desktop);
        types.insert(connection.WM_WINDOW_TYPE_DOCK(), WindowType::Dock);
        types.insert(connection.WM_WINDOW_TYPE_TOOLBAR(), WindowType::Toolbar);
        types.insert(connection.WM_WINDOW_TYPE_MENU(), WindowType::Menu);
        types.insert(connection.WM_WINDOW_TYPE_UTILITY(), WindowType::Utility);
        types.insert(connection.WM_WINDOW_TYPE_SPLASH(), WindowType::Splash);
        types.insert(connection.WM_WINDOW_TYPE_DIALOG(), WindowType::Dialog);
        types.insert(connection.WM_WINDOW_TYPE_DROPDOWN_MENU(), WindowType::DropdownMenu);
        types.insert(connection.WM_WINDOW_TYPE_POPUP_MENU(), WindowType::PopupMenu);
        types.insert(connection.WM_WINDOW_TYPE_TOOLTIP(), WindowType::Tooltip);
        types.insert(connection.WM_WINDOW_TYPE_NOTIFICATION(), WindowType::Notification);
        types.insert(connection.WM_WINDOW_TYPE_COMBO(), WindowType::Combo);
        types.insert(connection.WM_WINDOW_TYPE_DND(), WindowType::Dnd);
        types.insert(connection.WM_WINDOW_TYPE_NORMAL(), WindowType::Normal);

        states.insert(connection.WM_STATE_MODAL(), WindowState::Modal);
        states.insert(connection.WM_STATE_STICKY(), WindowState::Sticky);
        states.insert(connection.WM_STATE_MAXIMIZED_VERT(), WindowState::MaximizedVert);
        states.insert(connection.WM_STATE_MAXIMIZED_HORZ(), WindowState::MaximizedHorz);
        states.insert(connection.WM_STATE_SHADED(), WindowState::Shaded);
        states.insert(connection.WM_STATE_SKIP_TASKBAR(), WindowState::SkipTaskbar);
        states.insert(connection.WM_STATE_SKIP_PAGER(), WindowState::SkipPager);
        states.insert(connection.WM_STATE_HIDDEN(), WindowState::Hidden);
        states.insert(connection.WM_STATE_FULLSCREEN(), WindowState::Fullscreen);
        states.insert(connection.WM_STATE_ABOVE(), WindowState::Above);
        states.insert(connection.WM_STATE_BELOW(), WindowState::Below);
        states.insert(connection.WM_STATE_DEMANDS_ATTENTION(), WindowState::DemandsAttention);

        Self {
            connection,
            screen,
            screen_idx,
            root_window,
            atoms,
            window_types: types,
            window_states: states,
            keymap: HashMap::new(),
            focused: None,
            screens: vec![],
        }
    }

    pub fn register_redirect(&self) {
        let values = [(
            xcb::CW_EVENT_MASK,
            (xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
                xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT |
                xcb::EVENT_MASK_PROPERTY_CHANGE |
                xcb::EVENT_MASK_BUTTON_PRESS),
        )];

        xcb::change_window_attributes_checked(
            &self.connection,
            self.root_window,
            &values,
        )
            .request_check()
            .expect("Could not register redirect");
    }

    pub fn register_keybinds(
        &mut self,
        keybinds: Vec<(KeyCombo, Command)>
    ) {
        let symbols = KeySymbols::new(&self.connection);
        for (combo, command) in keybinds {
            self.keymap.insert(combo, command);

            if let Some(code) = symbols.get_keycode(combo.key).next() {
                xcb::grab_key(
                    &self.connection,
                    false,
                    self.root_window,
                    combo.mods as u16, code,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                );
            }
        }
    }

    pub fn get_window_types(&self, window: xcb::Window) -> Vec<WindowType> {
        ewmh::get_wm_window_type(&self.connection, window)
            .get_reply()
            .map(|it| {
                it.atoms()
                    .iter()
                    .filter_map(|a| {
                        self.window_types.get(a).cloned()
                    })
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    pub fn get_window_states(&self, window: xcb::Window) -> Vec<WindowState> {
        ewmh::get_wm_state(&self.connection, window)
            .get_reply()
            .map(|it| {
                it.atoms()
                    .iter()
                    .filter_map(|a| {
                        self.window_states.get(a).cloned()
                    })
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    pub fn poll(&mut self) -> Event {
        loop {
            self.connection.flush();
            let event = self.connection.wait_for_event()
                .expect("Wait for event returned none");

            match event.response_type() {
                xcb::CONFIGURE_REQUEST => {
                    // We can skip this request, just have to pass it on unchanged
                    let event = unsafe { xcb::cast_event::<xcb::ConfigureRequestEvent>(&event) };

                    let values = vec![
                        (xcb::CONFIG_WINDOW_X as u16, event.x() as u32),
                        (xcb::CONFIG_WINDOW_Y as u16, event.y() as u32),
                        (xcb::CONFIG_WINDOW_WIDTH as u16, u32::from(event.width())),
                        (xcb::CONFIG_WINDOW_HEIGHT as u16, u32::from(event.height())),
                        (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, u32::from(event.border_width())),
                        (xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32),
                        (xcb::CONFIG_WINDOW_STACK_MODE as u16, u32::from(event.stack_mode())),
                    ];

                    let filtered_values: Vec<_> = values
                        .into_iter()
                        .filter(|&(mask, _)| mask & event.value_mask() != 0)
                        .collect();

                    xcb::configure_window(&self.connection, event.window(), &filtered_values);
                },
                xcb::MAP_REQUEST => {
                    let event = unsafe { xcb::cast_event::<xcb::MapRequestEvent>(&event) };
                    return Event::MapRequest(event.window());
                },
                xcb::UNMAP_NOTIFY => {
                    let event = unsafe { xcb::cast_event::<xcb::UnmapNotifyEvent>(&event) };

                    if event.event() == self.root_window {
                        // If it's an event on the root window, ignore it
                        continue;
                    }

                    return Event::UnmapNotify(event.window());
                },
                xcb::DESTROY_NOTIFY => {
                    let event = unsafe { xcb::cast_event::<xcb::DestroyNotifyEvent>(&event) };
                    return Event::DestroyNotify(event.event());
                },
                xcb::ENTER_NOTIFY => {
                    let event = unsafe { xcb::cast_event::<xcb::EnterNotifyEvent>(&event) };
                    return Event::EnterNotify(event.event());
                },

                xcb::KEY_PRESS => {
                    let event = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                    let syms = KeySymbols::new(&self.connection);
                    let key = syms.press_lookup_keysym(event, 0);
                    let mods = u32::from(event.state());

                    let combo = KeyCombo { mods, key, event: keys::Event::KeyDown };
                    if let Some(command) = self.keymap.get(&combo) {
                        return Event::Command(command.clone());
                    }
                },
                xcb::KEY_RELEASE => {
                    let event = unsafe { xcb::cast_event::<xcb::KeyPressEvent>(&event) };
                    let syms = KeySymbols::new(&self.connection);
                    let key = syms.press_lookup_keysym(event, 0);
                    let mods = u32::from(event.state());

                    let combo = KeyCombo { mods, key, event: keys::Event::KeyUp };
                    if let Some(command) = self.keymap.get(&combo) {
                        return Event::Command(command.clone());
                    }
                },
                _ => {},
            }
        }
    }
}

pub enum Event {
    MapRequest(xcb::Window),
    UnmapNotify(xcb::Window),
    DestroyNotify(xcb::Window),
    EnterNotify(xcb::Window),
    Command(keys::Command),
}

