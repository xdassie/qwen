use crate::input::keyboard::KeyboardEvent;
use crate::input::mouse::MouseClickEvent;
use bevy::prelude::*;
use std::collections::HashSet;

pub mod simplify;
pub use simplify::*;

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
            .add_plugins(SimplifyPlugin)
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

#[derive(Component)]
struct MeshStats;

#[derive(Component)]
struct MemoryStats;

fn setup_editor_ui(mut commands: Commands) {
    let ui_color = Color::srgba(0.2, 0.2, 0.2, 0.8);
    
    let panel_entity = commands.spawn((
        Name::new("EditorPanel"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            column_gap: Val::Px(5.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..Default::default()
        },
        BackgroundColor(ui_color),
    )).id();
    
    commands.entity(panel_entity).with_child((
        Name::new("SelectionCount"),
        SelectionCount,
        Text::new("0 selected"),
        TextFont { font_size: 14.0, ..Default::default() },
        TextColor(Color::WHITE),
    ));
    
    commands.entity(panel_entity).with_child((
        Name::new("MeshStats"),
        MeshStats,
        Text::new("Vertices: 0 | Faces: 0"),
        TextFont { font_size: 14.0, ..Default::default() },
        TextColor(Color::WHITE),
    ));
    
    commands.entity(panel_entity).with_child((
        Name::new("MemoryStats"),
        MemoryStats,
        Text::new("Memory: 0 MB"),
        TextFont { font_size: 14.0, ..Default::default() },
        TextColor(Color::WHITE),
    ));
    
    commands.entity(panel_entity).with_child((
        Name::new("EntityList"),
        EntityList,
        Text::new(""),
        TextFont { font_size: 14.0, ..Default::default() },
        TextColor(Color::WHITE),
    ));
}

fn refresh_scene_entities(
    mut scene: ResMut<SceneEntities>,
    query: Query<Entity, With<SceneRoot>>,
    children: Query<&Children>,
) {
    let mut all_entities = Vec::new();
    let mut to_process = query.iter().collect::<Vec<_>>();
    
    while let Some(entity) = to_process.pop() {
        all_entities.push(entity);
        if let Ok(children_ref) = children.get(entity) {
            to_process.extend(children_ref.iter());
        }
    }
    
    scene.entities = all_entities;
}

fn handle_click_events(
    mut clicks: MessageReader<MouseClickEvent>,
    scene: Res<SceneEntities>,
    mut selection: ResMut<SelectionState>,
    meshes: Query<&Mesh3d>,
) {
    let clicked_events: Vec<_> = clicks.read().collect();
    if clicked_events.is_empty() {
        return;
    }
    
    // Find the entity with Mesh3d (actual mesh, not transform node)
    let entity = clicked_events.iter()
        .find(|click| meshes.get(click.entity).is_ok())
        .map(|click| click.entity)
        .or_else(|| clicked_events.first().map(|click| click.entity));
    
    if let Some(entity) = entity {
        if !scene.entities.contains(&entity) {
            return;
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
        Query<&mut Text, With<MeshStats>>,
        Query<&mut Text, With<MemoryStats>>,
    )>,
    all_meshes: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    children: Query<&Children>,
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

    let mut total_vertices: usize = 0;
    let mut total_faces: usize = 0;
    let mut total_memory: usize = 0;
    
    let mut visited = HashSet::new();
    let mut to_process: Vec<Entity> = scene.entities.clone();
    
    while let Some(entity) = to_process.pop() {
        if !visited.insert(entity) {
            continue;
        }
        
        if let Ok(mesh3d) = all_meshes.get(entity) {
            if let Some(mesh) = meshes.get(&mesh3d.0) {
                let vertex_buffer = mesh.attribute(bevy_mesh::Mesh::ATTRIBUTE_POSITION);
                let index_buffer = mesh.indices();
                
                if let Some(vertices) = vertex_buffer {
                    total_vertices += vertices.len();
                    total_memory += vertices.len() * 12;
                }
                
                if let Some(indices) = index_buffer {
                    let index_count = match indices {
                        bevy_mesh::Indices::U16(buf) => {
                            total_memory += buf.len() * 2;
                            buf.len()
                        },
                        bevy_mesh::Indices::U32(buf) => {
                            total_memory += buf.len() * 4;
                            buf.len()
                        },
                    };
                    total_faces += index_count / 3;
                }
            }
        }
        
        if let Ok(children_ref) = children.get(entity) {
            for &child in children_ref {
                to_process.push(child);
            }
        }
    }
    
    if let Ok(mut stats) = queries.p2().single_mut() {
        *stats = Text::from(format!("Vertices: {} | Faces: {}", total_vertices, total_faces));
    }
    
    if let Ok(mut mem) = queries.p3().single_mut() {
        let mb = total_memory as f64 / (1024.0 * 1024.0);
        *mem = Text::from(format!("Memory: {:.2} MB", mb));
    }
}


