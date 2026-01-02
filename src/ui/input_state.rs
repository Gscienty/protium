use std::{collections, sync};

use enum_flags::enum_flags;
use strum::EnumIter;
use tokio::sync::mpsc;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{HtmlCanvasElement, KeyboardEvent, MouseEvent};

#[derive(Hash, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    A,
    AltLeft,
    AltRight,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    B,
    Backquote,
    Backslash,
    Backspace,
    BracketLeft,
    BracketRight,
    C,
    CapsLock,
    Comma,
    ContextMenu,
    ControlLeft,
    ControlRight,
    D,
    Decimal,
    Delete,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    E,
    End,
    Enter,
    Equal,
    Escape,
    F,
    F1,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F2,
    F20,
    F21,
    F22,
    F23,
    F24,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    G,
    H,
    Home,
    I,
    Insert,
    IntiBackslash,
    J,
    K,
    L,
    M,
    MetaLeft,
    MetaRight,
    Minus,
    N,
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadMultiply,
    NumpadSubtract,
    O,
    P,
    PageDown,
    PageUp,
    Pause,
    Period,
    PrintScreen,
    Q,
    Quote,
    R,
    S,
    ScrollLock,
    Semicolon,
    ShiftLeft,
    ShiftRight,
    Slash,
    Space,
    Subtract,
    T,
    Tab,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl TryFrom<&KeyboardEvent> for Key {
    type Error = ();

    fn try_from(value: &KeyboardEvent) -> Result<Self, Self::Error> {
        match value.key_code() {
            8 => Ok(Self::Backspace),
            9 => Ok(Self::Tab),
            13 => Ok(Self::Enter),
            16 => Ok(Self::ShiftLeft),
            17 => Ok(Self::ControlLeft),
            18 => Ok(Self::AltLeft),
            19 => Ok(Self::Pause),
            20 => Ok(Self::CapsLock),
            27 => Ok(Self::Escape),
            32 => Ok(Self::Space),
            33 => Ok(Self::PageUp),
            34 => Ok(Self::PageDown),
            35 => Ok(Self::End),
            36 => Ok(Self::Home),
            37 => Ok(Self::ArrowLeft),
            38 => Ok(Self::ArrowUp),
            39 => Ok(Self::ArrowRight),
            40 => Ok(Self::ArrowDown),
            45 => Ok(Self::Insert),
            46 => Ok(Self::Delete),
            48 => Ok(Self::Digit0),
            49 => Ok(Self::Digit1),
            50 => Ok(Self::Digit2),
            51 => Ok(Self::Digit3),
            52 => Ok(Self::Digit4),
            53 => Ok(Self::Digit5),
            54 => Ok(Self::Digit6),
            55 => Ok(Self::Digit7),
            56 => Ok(Self::Digit8),
            57 => Ok(Self::Digit9),
            65 => Ok(Self::A),
            66 => Ok(Self::B),
            67 => Ok(Self::C),
            68 => Ok(Self::D),
            69 => Ok(Self::E),
            70 => Ok(Self::F),
            71 => Ok(Self::G),
            72 => Ok(Self::H),
            73 => Ok(Self::I),
            74 => Ok(Self::J),
            75 => Ok(Self::K),
            76 => Ok(Self::L),
            77 => Ok(Self::M),
            78 => Ok(Self::N),
            79 => Ok(Self::O),
            80 => Ok(Self::P),
            81 => Ok(Self::Q),
            82 => Ok(Self::R),
            83 => Ok(Self::S),
            84 => Ok(Self::T),
            85 => Ok(Self::U),
            86 => Ok(Self::V),
            87 => Ok(Self::W),
            88 => Ok(Self::X),
            89 => Ok(Self::Y),
            90 => Ok(Self::Z),
            91 => Ok(Self::MetaLeft),
            93 => Ok(Self::ContextMenu),
            96 => Ok(Self::Numpad0),
            97 => Ok(Self::Numpad1),
            98 => Ok(Self::Numpad2),
            99 => Ok(Self::Numpad3),
            100 => Ok(Self::Numpad4),
            101 => Ok(Self::Numpad5),
            102 => Ok(Self::Numpad6),
            103 => Ok(Self::Numpad7),
            104 => Ok(Self::Numpad8),
            105 => Ok(Self::Numpad9),
            106 => Ok(Self::NumpadMultiply),
            107 => Ok(Self::NumpadAdd),
            108 => Ok(Self::NumpadEnter),
            109 => Ok(Self::NumpadSubtract),
            110 => Ok(Self::NumpadDecimal),
            111 => Ok(Self::NumpadDivide),
            112 => Ok(Self::F1),
            113 => Ok(Self::F2),
            114 => Ok(Self::F3),
            115 => Ok(Self::F4),
            116 => Ok(Self::F5),
            117 => Ok(Self::F6),
            118 => Ok(Self::F7),
            119 => Ok(Self::F8),
            120 => Ok(Self::F9),
            121 => Ok(Self::F10),
            122 => Ok(Self::F11),
            123 => Ok(Self::F12),
            144 => Ok(Self::NumLock),
            186 => Ok(Self::Semicolon),
            187 => Ok(Self::Equal),
            188 => Ok(Self::Comma),
            189 => Ok(Self::Subtract),
            190 => Ok(Self::Decimal),
            191 => Ok(Self::Backslash),
            192 => Ok(Self::Backquote),
            219 => Ok(Self::BracketLeft),
            220 => Ok(Self::IntiBackslash),
            221 => Ok(Self::BracketRight),
            222 => Ok(Self::Quote),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[enum_flags]
#[derive(Hash, Clone, Copy, PartialEq, Eq, Default, EnumIter)]
pub enum MouseButton {
    #[default]
    None = 0,
    Main = 1,
    Auxiliary = 2,
    Secondary = 4,
    Fourth = 8,
    Fifth = 16,
}

impl TryFrom<&MouseEvent> for MouseButton {
    type Error = ();

    fn try_from(value: &MouseEvent) -> Result<Self, Self::Error> {
        match value.button() {
            0 => Ok(Self::Main),
            1 => Ok(Self::Auxiliary),
            2 => Ok(Self::Secondary),
            3 => Ok(Self::Fourth),
            4 => Ok(Self::Fifth),
            _ => Err(()),
        }
    }
}

enum InputEvent {
    Key((Key, bool)),
    MouseButton((MouseButton, bool)),
}

#[derive(Debug, Default)]
pub struct Touch {
    id: i32,
    x: i32,
    y: i32,
}

#[derive(Debug)]
pub(crate) struct InputStateReceiver {
    receiver: sync::Mutex<mpsc::UnboundedReceiver<InputEvent>>,

    pub(crate) key_pressed: sync::Mutex<collections::HashSet<Key>>,
    mouse_button_pressed: sync::Mutex<MouseButton>,

    pressed_sender: sync::Arc<PressedEventSender>,

    cursor_state: sync::Arc<CursorState>,
}

#[derive(Debug, Default)]
pub struct InputState {
    cursor_x: i32,
    cursor_y: i32,
    key_pressed: collections::HashSet<Key>,
    mouse_button_pressed: MouseButton,
}

impl InputState {
    pub const fn get_cursor(&self) -> (i32, i32) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn get_key_pressed(&self) -> &collections::HashSet<Key> {
        &self.key_pressed
    }

    pub fn get_mouse_button_pressed(&self) -> MouseButton {
        self.mouse_button_pressed
    }
}

#[derive(Debug)]
struct CursorState {
    cursor_x: sync::atomic::AtomicI32,
    cursor_y: sync::atomic::AtomicI32,
}

#[derive(Debug)]
struct PressedEventSender {
    sender: mpsc::UnboundedSender<InputEvent>,
    key_pressed: sync::Mutex<collections::HashSet<Key>>,
    mouse_button_pressed: sync::Mutex<MouseButton>,
}

impl InputStateReceiver {
    pub(super) fn new(canvas: &HtmlCanvasElement) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let state = Self {
            receiver: sync::Mutex::new(receiver),

            key_pressed: sync::Mutex::new(collections::HashSet::new()),
            mouse_button_pressed: sync::Mutex::new(MouseButton::None),

            pressed_sender: sync::Arc::new(PressedEventSender {
                sender,
                key_pressed: sync::Mutex::new(collections::HashSet::new()),
                mouse_button_pressed: sync::Mutex::new(MouseButton::None),
            }),

            cursor_state: sync::Arc::new(CursorState {
                cursor_x: sync::atomic::AtomicI32::new(0),
                cursor_y: sync::atomic::AtomicI32::new(0),
            }),
        };

        Self::listen(&state, canvas);

        state
    }

    pub(super) fn update_state(&self) -> Option<InputState> {
        let Ok(mut receiver) = self.receiver.lock() else {
            return None;
        };

        while let Ok(event) = receiver.try_recv() {
            match event {
                InputEvent::Key((key, true)) => {
                    if let Ok(mut key_pressed) = self.key_pressed.lock() {
                        key_pressed.insert(key);
                    }
                }
                InputEvent::Key((key, false)) => {
                    if let Ok(mut key_pressed) = self.key_pressed.lock() {
                        key_pressed.remove(&key);
                    }
                }
                InputEvent::MouseButton((key, true)) => {
                    if let Ok(mut mouse_button_pressed) = self.mouse_button_pressed.lock() {
                        mouse_button_pressed.insert(key);
                    }
                }
                InputEvent::MouseButton((key, false)) => {
                    if let Ok(mut mouse_button_pressed) = self.mouse_button_pressed.lock() {
                        mouse_button_pressed.remove(key);
                    }
                }
            };
        }

        Some(InputState {
            cursor_x: self
                .cursor_state
                .cursor_x
                .load(sync::atomic::Ordering::Acquire),
            cursor_y: self
                .cursor_state
                .cursor_y
                .load(sync::atomic::Ordering::Acquire),

            key_pressed: if let Ok(key_pressed) = self.key_pressed.lock() {
                key_pressed.clone()
            } else {
                Default::default()
            },
            mouse_button_pressed: if let Ok(mouse_button_pressed) = self.mouse_button_pressed.lock()
            {
                mouse_button_pressed.clone()
            } else {
                Default::default()
            },
        })
    }

    fn listen(&self, canvas: &HtmlCanvasElement) {
        let Some(window) = web_sys::window() else {
            return;
        };

        let cloned_state = self.pressed_sender.clone();
        let cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if let Ok(mut cache) = cloned_state.key_pressed.lock() {
                cloned_state.handle_keydown(&mut cache, event);
            }
        }) as Box<dyn Fn(KeyboardEvent)>);
        let _ = window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
        cb.forget();

        let cloned_state = self.pressed_sender.clone();
        let cb = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if let Ok(mut cache) = cloned_state.key_pressed.lock() {
                cloned_state.handle_keyup(&mut cache, event);
            }
        }) as Box<dyn Fn(KeyboardEvent)>);
        let _ = window.add_event_listener_with_callback("keyup", cb.as_ref().unchecked_ref());
        cb.forget();

        let cloned_state = self.pressed_sender.clone();
        let cloned_cursor_state = self.cursor_state.clone();
        let cb = Closure::wrap(Box::new(move |event: MouseEvent| {
            if let Ok(mut cache) = cloned_state.mouse_button_pressed.lock() {
                cloned_state.handle_mousedown(&mut cache, &cloned_cursor_state, event);
            }
        }) as Box<dyn Fn(MouseEvent)>);
        let _ = canvas.add_event_listener_with_callback("mousedown", cb.as_ref().unchecked_ref());
        cb.forget();

        let cloned_state = self.pressed_sender.clone();
        let cloned_cursor_state = self.cursor_state.clone();
        let cb = Closure::wrap(Box::new(move |event: MouseEvent| {
            if let Ok(mut cache) = cloned_state.mouse_button_pressed.lock() {
                cloned_state.handle_mouseup(&mut cache, &cloned_cursor_state, event);
            }
        }) as Box<dyn Fn(MouseEvent)>);
        let _ = canvas.add_event_listener_with_callback("mouseup", cb.as_ref().unchecked_ref());
        cb.forget();

        let cloned_state = self.cursor_state.clone();
        let cb = Closure::wrap(Box::new(move |event: MouseEvent| {
            cloned_state.handle_mousemove(event);
        }) as Box<dyn Fn(MouseEvent)>);
        let _ = canvas.add_event_listener_with_callback("mousemove", cb.as_ref().unchecked_ref());
        cb.forget();
    }
}

impl CursorState {
    fn set_mouse_cursor(&self, e: &MouseEvent) {
        self.cursor_x
            .store(e.client_x(), sync::atomic::Ordering::Release);
        self.cursor_y
            .store(e.client_y(), sync::atomic::Ordering::Release);
    }

    fn handle_mousemove(&self, e: MouseEvent) {
        e.prevent_default();

        self.set_mouse_cursor(&e);
    }
}

impl PressedEventSender {
    fn handle_keydown(&self, state: &mut collections::HashSet<Key>, e: KeyboardEvent) {
        e.prevent_default();

        let Ok(key) = Key::try_from(&e) else {
            return;
        };
        if state.insert(key) {
            let _ = self.sender.send(InputEvent::Key((key, true)));
        }
    }

    fn handle_keyup(&self, state: &mut collections::HashSet<Key>, e: KeyboardEvent) {
        e.prevent_default();

        let Ok(key) = Key::try_from(&e) else {
            return;
        };
        if state.remove(&key) {
            let _ = self.sender.send(InputEvent::Key((key, false)));
        }
    }

    fn handle_mousedown(&self, state: &mut MouseButton, cursor_state: &CursorState, e: MouseEvent) {
        e.prevent_default();

        let Ok(button) = MouseButton::try_from(&e) else {
            return;
        };
        let changed = !state.has_flag(button);
        state.insert(button);
        if changed {
            let _ = self.sender.send(InputEvent::MouseButton((button, true)));
        }
        cursor_state.set_mouse_cursor(&e);
    }

    fn handle_mouseup(&self, state: &mut MouseButton, cursor_state: &CursorState, e: MouseEvent) {
        e.prevent_default();

        let Ok(button) = MouseButton::try_from(&e) else {
            return;
        };
        let changed = state.has_flag(button);
        state.remove(button);
        if changed {
            let _ = self.sender.send(InputEvent::MouseButton((button, false)));
        }
        cursor_state.set_mouse_cursor(&e);
    }
}

static INPUT_STATE: sync::OnceLock<InputStateReceiver> = sync::OnceLock::new();

pub(crate) fn init_input_state(canvas: &web_sys::HtmlCanvasElement) {
    INPUT_STATE.get_or_init(|| InputStateReceiver::new(canvas));
}

pub(crate) fn process_input_state<F, T>(cb: F) -> Option<T>
where
    F: FnOnce(&InputStateReceiver) -> Option<T>,
{
    let material = INPUT_STATE.get()?;

    cb(material)
}
