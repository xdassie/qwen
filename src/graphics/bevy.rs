use bevy::prelude::*;
use bevy::render::view::window::screenshot::{save_to_disk, Screenshot};
use bevy::render::camera::CameraRenderGraph;
use bevy::gltf::GltfAssetLabel;
use std::env;

#[derive(Resource, Default)]
pub struct TestMode(pub bool);

#[derive(Resource, Default)]
pub struct TestModeExit(pub bool);

#[derive(Resource, Default)]
pub struct CaptureTimer(Timer);

#[derive(Resource, Default)]
pub struct GltfLoadingState {
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
    pub scene_handle: Handle<Scene>,
}

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

pub fn run(test_mode: bool) {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: if test_mode { "Bevy Test Mode" } else { "Bevy! GLB Loader" }.into(),
                    resolution: (800, 600).into(),
                    visible: test_mode,
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            })
        )
        .init_resource::<TestMode>()
        .init_resource::<TestModeExit>()
        .init_resource::<CaptureTimer>()
        .init_resource::<GltfLoadingState>()
        .insert_resource(TestMode(test_mode))
        .insert_resource(CaptureTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Startup, load_glb)
        .add_systems(Update, (
            on_glb_loaded,
            spin_model,
            spawn_screenshot,
            check_screenshot,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    _meshes: Res<Assets<Mesh>>,
    _materials: Res<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(Handle::<Mesh>::default()),
        MeshMaterial3d(Handle::<StandardMaterial>::default()),
        AmbientLight {
            color: Color::srgb(0.5, 0.5, 0.5),
            brightness: 500.0,
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 1000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d { ..default() },
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn load_glb(
    asset_server: Res<AssetServer>,
    mut state: ResMut<GltfLoadingState>,
    _test_mode: Res<TestMode>,
) {
    let model_path = get_model_path();
    
    if model_path.is_empty() {
        state.loading = false;
        state.loaded = false;
        state.error = None;
        state.scene_handle = Default::default();
        return;
    }
    
    state.loading = true;
    state.loaded = false;
    state.error = None;
    state.scene_handle = Default::default();
    
    let scene_handle = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset(model_path)
    );
    
    state.scene_handle = scene_handle;
}

fn on_glb_loaded(
    asset_server: Res<AssetServer>,
    scenes: Res<Assets<Scene>>,
    mut state: ResMut<GltfLoadingState>,
    mut commands: Commands,
) {
    info!("on_glb_loaded called - loading: {}, loaded: {}", state.loading, state.loaded);
    if state.loading && !state.loaded {
        return;
    }
    
    let scene_opt = scenes.get(&state.scene_handle);
    if scene_opt.is_none() {
        state.loaded = false;
        state.error = Some("Scene not found in assets".to_string());
        return;
    }
    
    state.loading = false;
    state.loaded = true;
    state.error = None;
    
    let scene = scene_opt.unwrap();
    commands.spawn((
        SceneRoot(state.scene_handle.clone()),
        Transform::default(),
        Visibility::Visible,
    ));
}

fn spin_model(
    time: Res<Time>,
    mut model_query: Query<&mut Transform, (With<SceneRoot>, Without<Camera3d>)>,
) {
    for mut transform in model_query.iter_mut() {
        transform.rotate_z(time.delta_secs() * 2.0);
    }
}

fn spawn_screenshot(
    time: Res<Time>,
    mut commands: Commands,
    test_mode: Res<TestMode>,
    mut capture_timer: ResMut<CaptureTimer>,
    mut test_mode_exit: ResMut<TestModeExit>,
    state: Res<GltfLoadingState>,
) {
    if !test_mode.0 {
        return;
    }
    
    if test_mode_exit.0 {
        return;
    }
    
    capture_timer.0.tick(time.delta());
    if capture_timer.0.just_finished() {
        info!("Taking screenshot - scene loaded: {}, handle: {:?}", state.loaded, state.scene_handle);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk("screenshot.png"));
        test_mode_exit.0 = true;
        capture_timer.0 = Timer::from_seconds(0.0, TimerMode::Repeating);
    }
}

fn check_screenshot(
    screenshot_entity: Query<Entity, With<Screenshot>>,
    mut test_mode_exit: ResMut<TestModeExit>,
    mut capture_timer: ResMut<CaptureTimer>,
) {
    if screenshot_entity.is_empty() {
        return;
    }
    test_mode_exit.0 = true;
    capture_timer.0 = Timer::from_seconds(0.0, TimerMode::Repeating);
}
