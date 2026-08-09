use bevy::prelude::*;
use std::env;
use crate::graphics::recorder::{FrameCount, Recorder, ScreenshotRecorder};
use crate::graphics::scene_loader::{SceneLoader, ModelSceneLoader, GltfLoadingState};
use crate::graphics::transform::{TransformListener, TransformKeyboardPlugin};
use crate::input::keyboard::KeyboardPlugin;


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
    
    let loader = ModelSceneLoader::new(&get_model_path());
    let recorder = ScreenshotRecorder::new(limit as u64);
    
    let mut app = App::new();
    app.init_resource::<FrameCount>();
    app.init_resource::<GltfLoadingState>();
    
    app.insert_resource(FrameCount(0));
    app.insert_resource(loader);
    app.insert_resource(recorder);
    
    app.add_systems(Startup, setup)
        .add_systems(Startup, |loader: Res<ModelSceneLoader>, asset_server: Res<AssetServer>, state: ResMut<GltfLoadingState>| {
            ModelSceneLoader::load_glb(&loader, &asset_server, state);
        })
        .add_systems(Update, |scenes: Res<Assets<Scene>>, mut commands: Commands, time: Res<Time>, loader: Res<ModelSceneLoader>, state: ResMut<GltfLoadingState>| {
            ModelSceneLoader::on_glb_loaded(&loader, &scenes, state, &mut commands, &time);
        })
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
        });
    
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
