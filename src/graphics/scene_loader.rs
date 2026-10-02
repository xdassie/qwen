use bevy::prelude::*;
use bevy::pbr::StandardMaterial;
use std::fs;
use crate::input::mouse::MouseClickReceiver;

#[derive(Resource, Clone, Default)]
pub struct GltfLoadingState {
    pub loading: bool,
    pub loaded: bool,
    pub error: Option<String>,
    pub scene_handle: Handle<Scene>,
}

#[derive(Resource)]
pub struct TextureLoadInfo {
    pub loaded: bool,
}

impl Default for TextureLoadInfo {
    fn default() -> Self {
        TextureLoadInfo { loaded: false }
    }
}

pub trait SceneLoader: Resource {
    fn load_glb(loader: &Self, asset_server: &Res<AssetServer>, state: ResMut<GltfLoadingState>) where Self: 'static;
    
    fn on_glb_loaded(loader: &Self, scenes: &Res<Assets<Scene>>, state: ResMut<GltfLoadingState>, commands: &mut Commands, time: &Res<Time>) where Self: 'static;
}

pub fn add_click_receptors_to_scene_children(
    mut commands: Commands,
    children: Query<&Children>,
    scene_roots: Query<(Entity, &SceneRoot), Added<SceneRoot>>,
) {
    for (root_entity, _scene_root) in &scene_roots {
        let mut to_process = Vec::new();
        if let Ok(children_ref) = children.get(root_entity) {
            to_process.extend(children_ref.iter());
        }
        
        while let Some(entity) = to_process.pop() {
            commands.entity(entity).insert(MouseClickReceiver::default());
            if let Ok(children_ref) = children.get(entity) {
                to_process.extend(children_ref.iter());
            }
        }
    }
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
        
        if state.scene_handle != Handle::default() {
            info!("Scene already loaded: {}", model_path);
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
                return;
            }
            
            let scene_opt = scenes.get(&state.scene_handle);
            
            if scene_opt.is_none() {
                return;
            }
            
            state.loaded = true;
            state.loading = false;
            state.error = None;
            
            info!("Scene loaded successfully, spawning");
            
            commands.spawn((
                SceneRoot(state.scene_handle.clone()),
                Transform::default(),
                Visibility::Visible,
                MouseClickReceiver::default(),
            ));
            
            commands.init_resource::<TextureLoadInfo>();
        }
    }
}

pub fn log_textures_at_load(
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    mut load_info: Option<ResMut<TextureLoadInfo>>,
    query: Query<(Entity, Option<&MeshMaterial3d<StandardMaterial>>)>,
) {
    if let Some(mut info) = load_info {
        if info.loaded {
            return;
        }
        info.loaded = true;
    } else {
        return;
    }
    
    info!("========== TEXTURE LOAD LOG ==========");
    info!("Total images in asset server: {}", images.len());
    
    let mut texture_count = 0;
    let mut materials_with_textures = Vec::new();
    
    for (entity, material) in &query {
        if let Some(material_handle) = material {
            if let Some(mat) = materials.get(&material_handle.0) {
                let mut has_texture = false;
                let mut textures = Vec::new();
                
                if let Some(tex) = &mat.base_color_texture {
                    has_texture = true;
                    if let Some(img) = images.get(tex) {
                        textures.push(format!("base_color: {}x{}", img.texture_descriptor.size.width, img.texture_descriptor.size.height));
                    } else {
                        textures.push("base_color: (image not found)".to_string());
                    }
                }
                
                if let Some(tex) = &mat.metallic_roughness_texture {
                    has_texture = true;
                    if let Some(img) = images.get(tex) {
                        textures.push(format!("metallic_roughness: {}x{}", img.texture_descriptor.size.width, img.texture_descriptor.size.height));
                    } else {
                        textures.push("metallic_roughness: (image not found)".to_string());
                    }
                }
                
                if let Some(tex) = &mat.normal_map_texture {
                    has_texture = true;
                    if let Some(img) = images.get(tex) {
                        textures.push(format!("normal_map: {}x{}", img.texture_descriptor.size.width, img.texture_descriptor.size.height));
                    } else {
                        textures.push("normal_map: (image not found)".to_string());
                    }
                }
                
                if let Some(tex) = &mat.emissive_texture {
                    has_texture = true;
                    if let Some(img) = images.get(tex) {
                        textures.push(format!("emissive: {}x{}", img.texture_descriptor.size.width, img.texture_descriptor.size.height));
                    } else {
                        textures.push("emissive: (image not found)".to_string());
                    }
                }
                
                if has_texture {
                    materials_with_textures.push((entity, textures));
                    texture_count += 1;
                }
            }
        }
    }
    
    info!("Materials with textures: {}", texture_count);
    for (i, (entity, textures)) in materials_with_textures.iter().enumerate() {
        info!("  Material {}: {:?}", i, textures.join(", "));
    }
    info!("=====================================");
}
