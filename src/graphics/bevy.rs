use bevy::prelude::*;
use std::env;
use crate::graphics::recorder::{FrameCount, Recorder, ScreenshotRecorder};
use crate::graphics::scene_loader::{SceneLoader, ModelSceneLoader, GltfLoadingState};

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
        arg.trim_start_matches("--limit=").parse::<usize>().unwrap_or(100)
    } else if let Some(arg) = args.iter().find(|a| *a == "--limit") {
        arg.trim().parse::<usize>().unwrap_or(100)
    } else {
        100
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
        });
    
    app.add_systems(Update, spin_model)
        .add_systems(Update, |scenes: Res<Assets<Scene>>, mut commands: Commands, time: Res<Time>, loader: Res<ModelSceneLoader>, state: ResMut<GltfLoadingState>| {
            ModelSceneLoader::on_glb_loaded(&loader, &scenes, state, &mut commands, &time);
        })
        .add_systems(Update, |mut frame_count: ResMut<FrameCount>, recorder: Res<ScreenshotRecorder>, commands: Commands, window_query: Query<Entity, With<Window>>| {
            frame_count.0 += 1;
            let current_frame = frame_count.0;
            if recorder.limit_reached(current_frame) {
                println!("Screenshot limit reached: {} screenshots captured", recorder.get_screenshot_limit());
                std::process::exit(0);
            }
            <ScreenshotRecorder as Recorder>::capture_screenshot(&*recorder, current_frame, commands, window_query);
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

pub fn spin_model(
    time: Res<Time>,
    mut model_query: Query<&mut Transform, (With<SceneRoot>, Without<Camera3d>)>,
) {
    for mut transform in model_query.iter_mut() {
        transform.rotate_z(time.delta_secs() * 2.0);
    }
}
