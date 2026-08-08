use bevy::prelude::*;
use bevy::input::keyboard::KeyCode;
use crate::input::keyboard::{KeyboardEvent, KeyboardListener};

/// Plugin for transform keyboard listener
pub struct TransformKeyboardPlugin;

impl Plugin for TransformKeyboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, keyboard_message_listener);
    }
}

/// System that reads keyboard messages and delivers them to listeners
pub fn keyboard_message_listener(
    mut reader: MessageReader<KeyboardEvent>,
    mut listeners: Query<(Entity, &mut Transform, &mut TransformListener), With<TransformListener>>,
) {
    for (_entity, mut transform, mut listener) in listeners.iter_mut() {
        for event in reader.read() {
            listener.handle(&event, &mut *transform);
        }
    }
}

/// Component that listens for keyboard events to transform the model
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TransformListener;

impl KeyboardListener for TransformListener {
    fn handle(&mut self, event: &KeyboardEvent, transform: &mut Transform) {
        match event.code {
            KeyCode::ArrowUp => {
                transform.rotate_x(0.01);
            }
            KeyCode::ArrowDown => {
                transform.rotate_x(-0.01);
            }
            KeyCode::ArrowLeft => {
                transform.rotate_z(0.01);
            }
            KeyCode::ArrowRight => {
                transform.rotate_z(-0.01);
            }
            _ => {}
        }
    }
}
