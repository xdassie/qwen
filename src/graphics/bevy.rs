use bevy::prelude::*;
use std::env;
use std::fs;

/// Trait for recording screenshots, always active and writes to /tmp each frame
pub trait Recorder {
    fn record(&self, frame_count: u64);
    fn limit_reached(&self, frame_count: u64) -> bool;
    fn capture_screenshot(&self, frame_count: u64, commands: Commands, window_query: Query<Entity, With<Window>>);
    fn get_screenshot_limit(&self) -> u64;
}

#[derive(Resource, Clone)]
pub struct ScreenshotRecorder {
    screenshot_limit: u64,
}

impl ScreenshotRecorder {
    pub fn new(limit: u64) -> Self {
        Self { screenshot_limit: limit }
    }
}

impl Recorder for ScreenshotRecorder {
    fn record(&self, frame_count: u64) {
        println!("DEBUG: Record frame {}", frame_count);
    }
    
    fn limit_reached(&self, frame_count: u64) -> bool {
        frame_count >= self.screenshot_limit
    }
    
    fn capture_screenshot(
        &self,
        frame_count: u64,
        mut commands: Commands,
        window_query: Query<Entity, With<Window>>,
    ) {
        self.record(frame_count);
        
        println!("DEBUG: Capturing screenshot {}", frame_count);
        
        // Spawn screenshot capture on the first window
        for _window_entity in window_query.iter() {
            let filename = format!("/tmp/screenshot_{}.png", frame_count);
            commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(filename));
        }
    }
    
    fn get_screenshot_limit(&self) -> u64 {
        self.screenshot_limit
    }
}

pub trait SceneLoader {
    fn load_glb(
        &mut self,
        asset_server: Res<AssetServer>,
        state: ResMut<GltfLoadingState>,
    );
    
    fn on_glb_loaded(
        &mut self,
        scenes: Res<Assets<Scene>>,
        state: ResMut<GltfLoadingState>,
        commands: &mut Commands,
        time: Res<Time>,
    );
}

#[derive(Resource, Clone)]
pub struct ModelSceneLoader {
    model_path: String,
    screenshot_limit: u64,
}

impl ModelSceneLoader {
    pub fn new(model_path: &str, screenshot_limit: u64) -> Self {
        ModelSceneLoader {
            model_path: model_path.to_string(),
            screenshot_limit,
        }
    }
}

impl SceneLoader for ModelSceneLoader {
    fn load_glb(
        &mut self,
        asset_server: Res<AssetServer>,
        mut state: ResMut<GltfLoadingState>,
    ) {
        let model_path = self.model_path.clone();
        
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
        
        info!("Loading scene: {}", model_path);
        
        // Check if file exists before loading
        if !fs::metadata(&model_path).is_ok() {
            state.loading = false;
            state.loaded = false;
            state.error = Some(format!("Model file not found: {}", model_path));
            error!("Failed to load model: {}", model_path);
            return;
        }
        
        let scene_handle = asset_server.load(
            GltfAssetLabel::Scene(0).from_asset(model_path)
        );
        
        state.scene_handle = scene_handle.clone();
        info!("Loaded asset handle: {:?}", scene_handle);
    }
    
    fn on_glb_loaded(
        &mut self,
        scenes: Res<Assets<Scene>>,
        mut state: ResMut<GltfLoadingState>,
        commands: &mut Commands,
        time: Res<Time>,
    ) {
        if state.loading && !state.loaded {
            if time.elapsed_secs() > 10.0 {
                state.error = Some("Scene loading timed out".to_string());
                error!("Scene loading timed out: {:?}", state.scene_handle);
                state.loading = false;
            }
        }
        
        let scene_opt = scenes.get(&state.scene_handle);
        
        if scene_opt.is_none() {
            state.loaded = false;
            state.error = Some("Scene not found in assets".to_string());
            error!("Scene not found: {:?}", state.scene_handle);
        }
        
        if !state.loaded {
            state.loaded = true;
            state.loading = false;
            state.error = None;
            
            info!("Scene loaded successfully, spawning");
            
            commands.spawn((
                SceneRoot(state.scene_handle.clone()),
                Transform::default(),
                Visibility::Visible,
            ));
        }
    }
}

#[derive(Resource, Default)]
pub struct GltfLoadingState {
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
    pub scene_handle: Handle<Scene>,
}

#[derive(Resource, Default, Clone)]
struct FrameCount(u64);

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
    
    let loader = ModelSceneLoader::new(&get_model_path(), limit as u64);
    let recorder = ScreenshotRecorder::new(limit as u64);
    
    let mut app = App::new();
    app.init_resource::<GltfLoadingState>();
    app.init_resource::<FrameCount>();
    app.insert_resource(loader.clone());
    app.insert_resource(recorder.clone());
    app.add_systems(Startup, setup);
    app.add_systems(Startup, update_loader);
    
    if limit > 0 {
        app.add_systems(Update, (update_scene, spin_model, capture_screenshot));
    }
    
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



fn setup(
    mut commands: Commands,
) {
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

fn update_loader(
    mut loader: ResMut<ModelSceneLoader>,
    asset_server: Res<AssetServer>,
    state: ResMut<GltfLoadingState>,
) {
    loader.load_glb(asset_server, state);
}


fn update_scene(
    mut loader: ResMut<ModelSceneLoader>,
    scenes: Res<Assets<Scene>>,
    state: ResMut<GltfLoadingState>,
    mut commands: Commands,
    time: Res<Time>,
) {
    loader.on_glb_loaded(scenes, state, &mut commands, time);
}



fn spin_model(
    time: Res<Time>,
    mut model_query: Query<&mut Transform, (With<SceneRoot>, Without<Camera3d>)>,
) {
    for mut transform in model_query.iter_mut() {
        transform.rotate_z(time.delta_secs() * 2.0);
    }
}

fn capture_screenshot(
    commands: Commands,
    window_query: Query<Entity, With<Window>>,
    mut frame_count: ResMut<FrameCount>,
    recorder: Res<ScreenshotRecorder>,
) {
    // Increment frame count
    frame_count.0 += 1;
    let current_frame = frame_count.0;
    
    // Check if we've reached the limit using trait method
    if recorder.limit_reached(current_frame) {
        println!("Screenshot limit reached: {} screenshots captured", recorder.get_screenshot_limit());
        std::process::exit(0);
    }
    
    // Capture screenshot using the trait method
    recorder.capture_screenshot(current_frame, commands, window_query);
}


