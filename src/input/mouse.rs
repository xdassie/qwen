use bevy::prelude::*;
use bevy::picking::prelude::*;
use bevy::input::mouse::MouseButton;

/// Component to receive mouse click events
#[derive(Component, Default, Debug, Clone)]
pub struct MouseClickReceiver {
    pub click_count: u32,
}

/// Mouse click event for the message bus
#[derive(Message, Clone, Debug)]
pub struct MouseClickEvent {
    pub entity: Entity,
    pub mouse_button: MouseButton,
    pub position: Vec2,
    pub click_count: u32,
}

/// Plugin for mouse click detection
pub struct MouseClickPlugin;

impl Plugin for MouseClickPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_message::<MouseClickEvent>()
            .add_systems(Update, mouse_click_system)
            .add_observer(mouse_click_observer);
    }
}

fn mouse_click_observer(
    click: On<Pointer<Click>>,
    mut events: MessageWriter<MouseClickEvent>,
) {
    // The Pointer<Click> event already contains the clicked entity and pointer location
    let loc = click.pointer_location.clone();
    events.write(MouseClickEvent {
        entity: click.entity,
        mouse_button: MouseButton::Left,
        position: Vec2::new(loc.position.x, loc.position.y),
        click_count: 1,
    });

    info!("Object clicked! Entity: {:?}, Position: {:?}", click.entity, loc);
}

fn mouse_click_system(
    mut query: Query<(Entity, &mut MouseClickReceiver)>,
) {
    // Process all entities with MouseClickReceiver
    for (entity, mut receiver) in query.iter_mut() {
        receiver.click_count += 1;
    }
}
