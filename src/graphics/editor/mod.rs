use crate::input::keyboard::KeyboardEvent;
use crate::input::mouse::MouseClickEvent;
use bevy::prelude::*;

/// Selection state resource - single source of truth
#[derive(Resource, Default, Debug)]
pub struct SelectionState {
    pub selected: Vec<Entity>,
}

/// Scene entities resource (tracks SceneRoot entities only)
#[derive(Resource, Default, Debug)]
pub struct SceneEntities {
    pub entities: Vec<Entity>,
}

/// Editor UI plugin
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .init_resource::<SceneEntities>()
            .add_systems(Startup, setup_editor_ui)
            .add_systems(Update, (
                refresh_scene_entities,
                handle_click_events,
                handle_delete_events,
                render_editor_ui,
            ));
    }
}

#[derive(Component)]
struct SelectionCount;

#[derive(Component)]
struct EntityList;

fn setup_editor_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("SelectionCount"),
        SelectionCount,
        Text::new("0 selected"),
        TextFont { font_size: 14.0, ..Default::default() },
    ));

    commands.spawn((
        Name::new("EntityList"),
        EntityList,
        Text::new(""),
        TextFont { font_size: 14.0, ..Default::default() },
    ));
}

fn refresh_scene_entities(
    mut scene: ResMut<SceneEntities>,
    query: Query<Entity, With<SceneRoot>>,
) {
    scene.entities = query.iter().collect();
}

fn handle_click_events(
    mut clicks: MessageReader<MouseClickEvent>,
    scene: Res<SceneEntities>,
    mut selection: ResMut<SelectionState>,
    parents: Query<&ChildOf>,
) {
    for click in clicks.read() {
        let mut entity = click.entity;
        
        // Traverse up to find SceneRoot parent
        let mut current = entity;
        loop {
            if scene.entities.contains(&current) {
                entity = current;
                break;
            }
            if let Ok(child_of) = parents.get(current) {
                current = child_of.0;
            } else {
                break;
            }
        }
        
        if !scene.entities.contains(&entity) {
            continue;
        }

        if let Some(idx) = selection.selected.iter().position(|e| *e == entity) {
            selection.selected.remove(idx);
        } else {
            selection.selected.push(entity);
        }
    }
}

fn handle_delete_events(
    mut keys: MessageReader<KeyboardEvent>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    entities: Query<Entity>,
) {
    for key in keys.read() {
        if key.code == KeyCode::KeyD && key.pressed {
            for entity in selection.selected.drain(..) {
                if entities.contains(entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn render_editor_ui(
    selection: Res<SelectionState>,
    scene: Res<SceneEntities>,
    mut queries: ParamSet<(
        Query<&mut Text, With<SelectionCount>>,
        Query<&mut Text, With<EntityList>>,
    )>,
) {
    if let Ok(mut count) = queries.p0().single_mut() {
        *count = Text::from(format!("{} selected", selection.selected.len()));
    }

    if let Ok(mut list) = queries.p1().single_mut() {
        let lines: Vec<_> = scene.entities.iter().map(|e| {
            let marker = if selection.selected.contains(e) { "[x]" } else { "[ ]" };
            format!("{} {:?}", marker, e)
        }).collect();
        *list = Text::from(lines.join("\n"));
    }
}


