use bevy::prelude::*;
use std::fs;

#[derive(Resource, Clone, Default)]
pub struct GltfLoadingState {
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
    pub scene_handle: Handle<Scene>,
}

pub trait SceneLoader: Resource {
    fn load_glb(loader: &Self, asset_server: &Res<AssetServer>, state: ResMut<GltfLoadingState>) where Self: 'static;
    
    fn on_glb_loaded(loader: &Self, scenes: &Res<Assets<Scene>>, state: ResMut<GltfLoadingState>, commands: &mut Commands, time: &Res<Time>) where Self: 'static;
}

#[derive(Resource, Clone)]
pub struct ModelSceneLoader {
    model_path: String,
}

impl ModelSceneLoader {
    pub fn new(model_path: &str) -> Self {
        ModelSceneLoader {
            model_path: model_path.to_string(),
        }
    }
}

impl SceneLoader for ModelSceneLoader {
    fn load_glb(
        loader: &ModelSceneLoader,
        asset_server: &Res<AssetServer>,
        mut state: ResMut<GltfLoadingState>,
    ) {
        let model_path = loader.model_path.clone();
        
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
        _loader: &ModelSceneLoader,
        scenes: &Res<Assets<Scene>>,
        mut state: ResMut<GltfLoadingState>,
        commands: &mut Commands,
        time: &Res<Time>,
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
            return;
        }
    }
}
