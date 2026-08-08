use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

/// Represents keyboard modifier keys
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ModifierKeys {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,     // Cmd/Win key
    pub fn_lock: bool,
    pub caps_lock: bool,
    pub scroll_lock: bool,
}

impl ModifierKeys {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_win(mut self) -> Self {
        self.win = true;
        self
    }
}

/// Keyboard event for the high-speed in-memory message bus
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub struct KeyboardEvent {
    pub code: KeyCode,
    pub modifiers: ModifierKeys,
    pub pressed: bool,  // true = key just pressed, false = just released
}

/// Plugin for keyboard input handling
pub struct KeyboardPlugin;

impl Plugin for KeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<KeyboardEvent>()
          .add_systems(Update, keyboard_input_system);
    }
}

/// System that produces keyboard events to the message bus
pub fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: MessageWriter<KeyboardEvent>,
) {
    // Fire only on just_pressed and just_released for clean events
    for key_code in keyboard.get_just_pressed() {
        let modifiers = extract_modifiers(&keyboard);
        events.write(KeyboardEvent {
            code: *key_code,
            modifiers,
            pressed: true,
        });
    }
    for key_code in keyboard.get_just_released() {
        let modifiers = extract_modifiers(&keyboard);
        events.write(KeyboardEvent {
            code: *key_code,
            modifiers,
            pressed: false,
        });
    }
}

/// Extracts modifier key states from the keyboard input
fn extract_modifiers(keyboard: &ButtonInput<KeyCode>) -> ModifierKeys {
    let mut modifiers = ModifierKeys::default();
    
    modifiers.shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    modifiers.ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    modifiers.alt = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    modifiers.win = keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight);
    modifiers.fn_lock = false; // Fn key is handled differently in Bevy 0.17
    modifiers.caps_lock = keyboard.pressed(KeyCode::CapsLock);
    modifiers.scroll_lock = keyboard.pressed(KeyCode::ScrollLock);
    
    modifiers
}

/// Trait for components that respond to keyboard events
/// Consumes the event and performs its own logic
pub trait KeyboardListener {
    fn handle(&mut self, event: &KeyboardEvent, transform: &mut Transform);
}


