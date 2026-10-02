use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::input::keyboard::KeyCode;
use std::env;
use crate::debug::memory::{MemoryTracker, query_gpu_memory, query_asset_memory, EcsMemory, query_process_memory};
use crate::input::keyboard::KeyboardEvent;
use crate::graphics::cameras::{CameraPlugin, EditorCamera, CameraListener};
use crate::graphics::editor::EditorPlugin;
use crate::graphics::recorder::{FrameCount, Recorder, ScreenshotRecorder};
use crate::graphics::scene_loader::{SceneLoader, ModelSceneLoader, GltfLoadingState, TextureLoadInfo, log_textures_at_load};
use crate::graphics::scene_saver::{SceneSaverPlugin, ExportOnLoad};
use crate::graphics::transform::{TransformListener, TransformKeyboardPlugin};
use crate::input::keyboard::KeyboardPlugin;
use crate::input::mouse::MouseClickPlugin;

pub fn get_model_path() -> String {
    let args: Vec<String> = env::args().collect();
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with("--model=") {
            return arg.trim_start_matches("--model=").to_string();
        } else if arg == "--model" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    String::new()
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let limit = if let Some(arg) = args.iter().find(|a| a.starts_with("--limit=")) {
        arg.trim_start_matches("--limit=").parse::<usize>().unwrap_or(0)
    } else if let Some(arg) = args.iter().find(|a| *a == "--limit") {
        arg.trim().parse::<usize>().unwrap_or(0)
    } else {
        0
    };
    
    let export_on_load = args.iter().any(|a| a == "--export" || a.starts_with("--export="));
    
    let loader = ModelSceneLoader::new(&get_model_path());
    let recorder = ScreenshotRecorder::new(limit as u64);
    
    let mut app = App::new();
    app.init_resource::<FrameCount>();
    app.init_resource::<GltfLoadingState>();
    app.init_resource::<MemoryTracker>();
    
    app.insert_resource(FrameCount(0));
    app.insert_resource(loader);
    app.insert_resource(recorder);
    app.insert_resource(ExportOnLoad { enabled: export_on_load, done: false });
    
    app.add_systems(Startup, setup)
        .add_systems(Startup, |mut tracker: ResMut<MemoryTracker>, adapter_info: Res<RenderAdapterInfo>| {
            tracker.gpu_memory = query_gpu_memory(&adapter_info);
        })
        .add_systems(Startup, |loader: Res<ModelSceneLoader>, asset_server: Res<AssetServer>, state: ResMut<GltfLoadingState>| {
            ModelSceneLoader::load_glb(&loader, &asset_server, state);
        })
        .add_systems(Update, |scenes: Res<Assets<Scene>>, mut commands: Commands, time: Res<Time>, loader: Res<ModelSceneLoader>, state: ResMut<GltfLoadingState>| {
            ModelSceneLoader::on_glb_loaded(&loader, &scenes, state, &mut commands, &time);
        })
        .add_systems(Update, log_textures_at_load)
        .add_systems(Update, |entities: Query<Entity, With<SceneRoot>>, mut commands: Commands, has_listener: Query<Entity, (With<TransformListener>, With<SceneRoot>)>| {
            for entity in entities.iter() {
                if !has_listener.iter().any(|e| e == entity) {
                    commands.entity(entity).insert(TransformListener::default());
                }
            }
        })
        .add_systems(Update, |mut frame_count: ResMut<FrameCount>, recorder: Res<ScreenshotRecorder>, commands: Commands, window_query: Query<Entity, With<Window>>| {
            frame_count.0 += 1;
            let current_frame = frame_count.0;
            let limit = recorder.get_screenshot_limit();
            if limit > 0 && recorder.limit_reached(current_frame) {
                println!("Screenshot limit reached: {} screenshots captured", limit);
                std::process::exit(0);
            }
            if limit > 0 {
                <ScreenshotRecorder as Recorder>::capture_screenshot(&*recorder, current_frame, commands, window_query);
            }
        })
        .add_systems(Update, memory_dump_system);
    
    app.add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy! GLB Loader".into(),
                    resolution: (800, 600).into(),
                    visible: true,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            })
        )
        .add_plugins(KeyboardPlugin)
        .add_plugins(TransformKeyboardPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(MouseClickPlugin)
        .add_plugins(EditorPlugin)
        .add_plugins(SceneSaverPlugin)
        .run();
}

fn setup(mut commands: Commands) {    
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d { ..default() },
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        EditorCamera::default(),
        CameraListener,
        MeshPickingCamera,
    ));

        commands.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 1000.0,
            range: 50.0,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn memory_dump_system(
    mut events: MessageReader<KeyboardEvent>,
    mut tracker: ResMut<MemoryTracker>,
    meshes: Res<Assets<Mesh>>,
    textures: Res<Assets<Image>>,
    adapter_info: Res<RenderAdapterInfo>,
    entity_query: Query<Entity>,
    transform_query: Query<&Transform>,
) {
    for event in events.read() {
        if event.code == KeyCode::KeyM 
            && event.modifiers.ctrl 
            && event.pressed {
            
            tracker.process_memory = query_process_memory();
            tracker.gpu_memory = query_gpu_memory(&adapter_info);
            tracker.asset_memory = query_asset_memory(&meshes, &textures);
            
            let entity_count = entity_query.iter().len();
            let transform_count = transform_query.iter().len();
            let transform_size = entity_count as u64 * std::mem::size_of::<Transform>() as u64;
            
            tracker.ecs_memory = EcsMemory {
                entity_count,
                component_type_count: 1,
                estimated_bytes: transform_size,
            };
            
            tracker.log_summary();
        }
    }
}
