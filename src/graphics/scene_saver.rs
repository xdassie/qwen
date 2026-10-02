use bevy::prelude::*;
use bevy_mesh::Mesh;
use gltf_json::{accessor, Asset, Buffer, Index, Root, Scene as GltfScene, mesh, scene as gltf_scene, buffer, accessor::ComponentType, buffer::Target, mesh::Mode, validation::Checked, mesh::Semantic};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use crate::input::keyboard::KeyboardEvent;

#[derive(Resource, Default)]
pub struct ExportOnLoad {
    pub enabled: bool,
    pub done: bool,
}

pub struct SceneSaverPlugin;

impl Plugin for SceneSaverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExportOnLoad>();
        app.add_systems(Update, save_scene_on_ctrl_s);
        app.add_systems(Update, export_on_load_if_enabled);
    }
}

fn export_on_load_if_enabled(
    export_flag: Res<ExportOnLoad>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&Children>)>,
) {
    if !export_flag.enabled || export_flag.done {
        return;
    }
    
    info!("Auto-export triggered on load");
    
    let output_path = "scene_export.glb";
    let export_result = export_scene(&query, &meshes, &materials, &images, output_path);
    
    match export_result {
        Ok(_) => {
            info!("Scene auto-exported successfully to {}", output_path);
        },
        Err(e) => error!("Failed to auto-export scene: {}", e),
    }
}

fn save_scene_on_ctrl_s(
    mut keys: MessageReader<KeyboardEvent>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    query: Query<(Entity, &Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&Children>)>,
) {
    for key in keys.read() {
        if key.code == KeyCode::KeyS 
            && key.pressed 
            && key.modifiers.ctrl {
            
            info!("CTRL+S pressed - saving scene");
            
            let output_path = "scene_export.glb";
            
            let export_result = export_scene(&query, &meshes, &materials, &images, output_path);
            
            match export_result {
                Ok(_) => {
                    info!("Scene saved successfully to {}", output_path);
                },
                Err(e) => error!("Failed to save scene: {}", e),
            }
        }
    }
}

pub fn export_scene(
    query: &Query<(Entity, &Transform, Option<&Mesh3d>, Option<&MeshMaterial3d<StandardMaterial>>, Option<&Children>)>,
    meshes: &Assets<Mesh>,
    materials: &Assets<StandardMaterial>,
    images: &Assets<Image>,
    output_path: &str,
) -> Result<(), String> {
    let mut root_nodes = Vec::new();
    let mut gltf_meshes = Vec::new();
    let mut gltf_materials = Vec::new();
    let mut binary_data = Vec::new();
    let mut buffer_views = Vec::new();
    let mut accessors = Vec::new();
    let mut gltf_images: Vec<gltf_json::image::Image> = Vec::new();
    let mut gltf_textures: Vec<gltf_json::texture::Texture> = Vec::new();
    let mut gltf_samplers: Vec<gltf_json::texture::Sampler> = Vec::new();
    
    let mut mesh_id = 0;
    let mut texture_map = std::collections::HashMap::<Handle<Image>, usize>::new();
    
    let entities: Vec<_> = query.iter().collect();
    info!("Export: Found {} entities with Transform", entities.len());
    
    for (entity, transform, mesh3d, material, children) in &entities {
        info!("  Entity {:?}: Mesh3d={:?}, Children={:?}", entity, mesh3d.is_some(), children.as_ref().map(|c| c.len()).unwrap_or(0));
        
        let node_transform_translation = Some([transform.translation.x, transform.translation.y, transform.translation.z]);
        let node_transform_rotation = Some([transform.rotation.x, transform.rotation.y, transform.rotation.z, transform.rotation.w]);
        let node_transform_scale = Some([transform.scale.x, transform.scale.y, transform.scale.z]);
        
        let material_idx = if let Some(material_handle) = material {
            if let Some(mat) = materials.get(&material_handle.0) {
                let srgba = mat.base_color.to_srgba();
                
                info!("  Material: base_color_texture={:?}, metallic_roughness_texture={:?}, normal_map_texture={:?}, emissive_texture={:?}", 
                    mat.base_color_texture.is_some(), mat.metallic_roughness_texture.is_some(), mat.normal_map_texture.is_some(), mat.emissive_texture.is_some());
                
                let base_color_texture_idx = export_texture(&mat.base_color_texture, images, &mut texture_map, &mut binary_data, &mut buffer_views, &mut gltf_images, &mut gltf_textures, &mut gltf_samplers)?;
                let base_color_texture = base_color_texture_idx.map(|idx| gltf_json::texture::Info {
                    index: idx,
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                });
                
                let (metallic_roughness_texture, metallic_factor, roughness_factor) = if mat.metallic_roughness_texture.is_some() {
                    let mr_tex_idx = export_texture(&mat.metallic_roughness_texture, images, &mut texture_map, &mut binary_data, &mut buffer_views, &mut gltf_images, &mut gltf_textures, &mut gltf_samplers)?;
                    let mr_texture_info = mr_tex_idx.map(|idx| gltf_json::texture::Info {
                        index: idx,
                        tex_coord: 0,
                        extensions: None,
                        extras: Default::default(),
                    });
                    (mr_texture_info, gltf_json::material::StrengthFactor(1.0), gltf_json::material::StrengthFactor(1.0))
                } else {
                    (None, gltf_json::material::StrengthFactor(mat.metallic), gltf_json::material::StrengthFactor(mat.perceptual_roughness))
                };
                
                let normal_texture_idx = export_texture(&mat.normal_map_texture, images, &mut texture_map, &mut binary_data, &mut buffer_views, &mut gltf_images, &mut gltf_textures, &mut gltf_samplers)?;
                let normal_texture = normal_texture_idx.map(|idx| gltf_json::material::NormalTexture {
                    index: idx,
                    scale: 1.0,
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                });
                
                let emissive_texture_idx = export_texture(&mat.emissive_texture, images, &mut texture_map, &mut binary_data, &mut buffer_views, &mut gltf_images, &mut gltf_textures, &mut gltf_samplers)?;
                let emissive_texture = emissive_texture_idx.map(|idx| gltf_json::texture::Info {
                    index: idx,
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                });
                
                let gltf_mat = gltf_json::material::Material {
                    pbr_metallic_roughness: gltf_json::material::PbrMetallicRoughness {
                        base_color_factor: gltf_json::material::PbrBaseColorFactor([srgba.red, srgba.green, srgba.blue, srgba.alpha]),
                        base_color_texture,
                        metallic_factor,
                        roughness_factor,
                        metallic_roughness_texture,
                        extensions: None,
                        extras: Default::default(),
                    },
                    alpha_mode: Checked::Valid(gltf_json::material::AlphaMode::Opaque),
                    alpha_cutoff: None,
                    double_sided: true,
                    normal_texture,
                    occlusion_texture: None,
                    emissive_factor: gltf_json::material::EmissiveFactor([0.0, 0.0, 0.0]),
                    emissive_texture,
                    name: None,
                    extras: Default::default(),
                    extensions: None,
                };
                let idx = gltf_materials.len();
                gltf_materials.push(gltf_mat);
                Some(Index::new(idx as u32))
            } else {
                None
            }
        } else {
            None
        };
        
        let node_mesh = if let Some(mesh_handle) = mesh3d {
            if let Some(mesh) = meshes.get(&mesh_handle.0) {
                let mesh_result = export_mesh(mesh, &mut binary_data, &mut buffer_views, &mut accessors);
                
                if let Ok((position_accessor_idx, indices_accessor_idx, uv_accessor_idx, normal_accessor_idx)) = mesh_result {
                    let mut attributes = BTreeMap::new();
                    attributes.insert(Checked::Valid(Semantic::Positions), Index::new(position_accessor_idx as u32));
                    
                    if let Some(uv_idx) = uv_accessor_idx {
                        attributes.insert(Checked::Valid(Semantic::TexCoords(0)), Index::new(uv_idx as u32));
                    }
                    
                    if let Some(normal_idx) = normal_accessor_idx {
                        attributes.insert(Checked::Valid(Semantic::Normals), Index::new(normal_idx as u32));
                    }
                    
                    let primitive = mesh::Primitive {
                        attributes,
                        indices: Some(Index::new(indices_accessor_idx as u32)),
                        mode: Checked::Valid(Mode::Triangles),
                        material: material_idx,
                        targets: Some(Vec::new()),
                        extras: Default::default(),
                        extensions: None,
                    };
                    
                    let mesh_obj = mesh::Mesh {
                        primitives: vec![primitive],
                        weights: None,
                        name: Some(format!("mesh_{}", mesh_id)),
                        extras: Default::default(),
                        extensions: None,
                    };
                    
                    let mesh_idx = gltf_meshes.len();
                    gltf_meshes.push(mesh_obj);
                    mesh_id += 1;
                    
                    Some(Index::new(mesh_idx as u32))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        let mut node_data = gltf_scene::Node::default();
        node_data.translation = node_transform_translation;
        node_data.scale = node_transform_scale;
        node_data.mesh = node_mesh;
        root_nodes.push(node_data);
    }
    
    let buffer_data = Buffer {
        byte_length: gltf_json::validation::USize64(binary_data.len() as u64),
        uri: None,
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    
    let scene_data = GltfScene {
        nodes: root_nodes.iter().enumerate().map(|(i, _)| Index::new(i as u32)).collect(),
        extras: Default::default(),
        extensions: None,
        name: None,
    };
    
    let asset_data = Asset {
        generator: Some("Bevy Scene Exporter".to_string()),
        version: "2.0".to_string(),
        ..Default::default()
    };
    
    info!("========== TEXTURE EXPORT LOG ==========");
    info!("Total images exported: {}", gltf_images.len());
    info!("Total textures exported: {}", gltf_textures.len());
    info!("Total samplers exported: {}", gltf_samplers.len());
    info!("=======================================");
    
    let gltf_doc = Root {
        asset: asset_data,
        buffers: vec![buffer_data],
        buffer_views,
        accessors,
        meshes: gltf_meshes,
        materials: gltf_materials,
        scenes: vec![scene_data],
        scene: Some(Index::new(0)),
        nodes: root_nodes,
        cameras: Vec::new(),
        images: gltf_images,
        samplers: gltf_samplers,
        skins: Vec::new(),
        textures: gltf_textures,
        animations: Vec::new(),
        extensions: None,
        extras: Default::default(),
        extensions_used: Vec::new(),
        extensions_required: Vec::new(),
    };
    
    write_glb(&gltf_doc, &binary_data, output_path)?;
    
    Ok(())
}

fn export_mesh(
    mesh: &Mesh,
    binary_data: &mut Vec<u8>,
    buffer_views: &mut Vec<buffer::View>,
    accessors: &mut Vec<accessor::Accessor>,
) -> Result<(usize, usize, Option<usize>, Option<usize>), String> {
    let position_attr = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        .ok_or("No position attribute")?;
    
    let position_data: Vec<f32> = match position_attr {
        bevy_mesh::VertexAttributeValues::Float32x3(vals) => {
            vals.iter().flat_map(|v| v.iter().cloned()).collect()
        }
        _ => return Err("Position must be Float32x3".to_string()),
    };
    
    let uv_attr = mesh.attribute(Mesh::ATTRIBUTE_UV_0);
    let normal_attr = mesh.attribute(Mesh::ATTRIBUTE_NORMAL);
    
    info!("Export: mesh attributes - has_uv={:?}, has_normal={:?}", uv_attr.is_some(), normal_attr.is_some());
    
    let position_bytes = position_data.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>();
    let position_offset = binary_data.len();
    binary_data.extend_from_slice(&position_bytes);
    
    let position_view = buffer::View {
        buffer: Index::new(0),
        byte_offset: Some(gltf_json::validation::USize64(position_offset as u64)),
        byte_length: gltf_json::validation::USize64(position_bytes.len() as u64),
        target: Some(Checked::Valid(Target::ArrayBuffer)),
        byte_stride: None,
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    let position_view_idx = buffer_views.len();
    buffer_views.push(position_view);
    
    let (min_vals, max_vals) = calculate_min_max(&position_data, 3);
    
    let vertex_count = position_data.len() / 3;
    
    let position_accessor = accessor::Accessor {
        buffer_view: Some(Index::new(position_view_idx as u32)),
        component_type: Checked::Valid(accessor::GenericComponentType(ComponentType::F32)),
        count: gltf_json::validation::USize64(vertex_count as u64),
        type_: Checked::Valid(accessor::Type::Vec3),
        min: Some(serde_json::json!(min_vals)),
        max: Some(serde_json::json!(max_vals)),
        name: None,
        normalized: false,
        extras: Default::default(),
        extensions: None,
        byte_offset: None,
        sparse: None,
    };
    let position_accessor_idx = accessors.len();
    accessors.push(position_accessor);
    
    let indices = mesh.indices().cloned();
    let (indices_accessor_idx, _) = if let Some(indices) = indices {
        let (index_bytes, count, component_type) = match &indices {
            bevy_mesh::Indices::U16(vals) => {
                let bytes: Vec<u8> = vals.iter().flat_map(|v| v.to_le_bytes()).collect();
                (bytes, vals.len(), ComponentType::U16)
            }
            bevy_mesh::Indices::U32(vals) => {
                let bytes: Vec<u8> = vals.iter().flat_map(|v| v.to_le_bytes()).collect();
                (bytes, vals.len(), ComponentType::U32)
            }
        };
        
        let index_offset = binary_data.len();
        binary_data.extend_from_slice(&index_bytes);
        
        let index_view = buffer::View {
            buffer: Index::new(0),
            byte_offset: Some(gltf_json::validation::USize64(index_offset as u64)),
            byte_length: gltf_json::validation::USize64(index_bytes.len() as u64),
            target: Some(Checked::Valid(Target::ElementArrayBuffer)),
            byte_stride: None,
            name: None,
            extras: Default::default(),
            extensions: None,
        };
        buffer_views.push(index_view);
        
        let (min_vals, max_vals) = calculate_min_max_indices(&index_bytes, component_type);
        
        let face_count = count / 3;
        
        let index_accessor = accessor::Accessor {
            buffer_view: Some(Index::new(buffer_views.len() as u32 - 1)),
            component_type: Checked::Valid(accessor::GenericComponentType(component_type)),
            count: gltf_json::validation::USize64(count as u64),
            type_: Checked::Valid(accessor::Type::Scalar),
            min: Some(serde_json::json!(min_vals)),
            max: Some(serde_json::json!(max_vals)),
            name: None,
            normalized: false,
            extras: Default::default(),
            extensions: None,
            byte_offset: None,
            sparse: None,
        };
        let idx = accessors.len();
        accessors.push(index_accessor);
        
        info!("Export: mesh - vertices={}, faces={}", vertex_count, face_count);
        
        (idx, face_count)
    } else {
        let indices: Vec<u32> = (0..position_data.len() as u32 / 3).collect();
        let index_bytes: Vec<u8> = indices.iter().flat_map(|v| v.to_le_bytes()).collect();
        
        let index_offset = binary_data.len();
        binary_data.extend_from_slice(&index_bytes);
        
        let index_view = buffer::View {
            buffer: Index::new(0),
            byte_offset: Some(gltf_json::validation::USize64(index_offset as u64)),
            byte_length: gltf_json::validation::USize64(index_bytes.len() as u64),
            target: Some(Checked::Valid(Target::ElementArrayBuffer)),
            byte_stride: None,
            name: None,
            extras: Default::default(),
            extensions: None,
        };
        buffer_views.push(index_view);
        
        let (min_vals, max_vals) = calculate_min_max_indices(&index_bytes, ComponentType::U32);
        
        let face_count = indices.len() / 3;
        
        let index_accessor = accessor::Accessor {
            buffer_view: Some(Index::new(buffer_views.len() as u32 - 1)),
            component_type: Checked::Valid(accessor::GenericComponentType(ComponentType::U32)),
            count: gltf_json::validation::USize64(indices.len() as u64),
            type_: Checked::Valid(accessor::Type::Scalar),
            min: Some(serde_json::json!(min_vals)),
            max: Some(serde_json::json!(max_vals)),
            name: None,
            normalized: false,
            extras: Default::default(),
            extensions: None,
            byte_offset: None,
            sparse: None,
        };
        let idx = accessors.len();
        accessors.push(index_accessor);
        
        info!("Export: mesh (fallback) - vertices={}, faces={}", vertex_count, face_count);
        
        (idx, face_count)
    };
    
    let uv_accessor_idx = if let Some(uv) = uv_attr {
        match uv {
            bevy_mesh::VertexAttributeValues::Float32x2(vals) => {
                let uv_data: Vec<f32> = vals.iter().flat_map(|v| v.iter().cloned()).collect();
                let uv_bytes = uv_data.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>();
                let uv_offset = binary_data.len();
                binary_data.extend_from_slice(&uv_bytes);
                
                let uv_view = buffer::View {
                    buffer: Index::new(0),
                    byte_offset: Some(gltf_json::validation::USize64(uv_offset as u64)),
                    byte_length: gltf_json::validation::USize64(uv_bytes.len() as u64),
                    target: Some(Checked::Valid(Target::ArrayBuffer)),
                    byte_stride: None,
                    name: None,
                    extras: Default::default(),
                    extensions: None,
                };
                buffer_views.push(uv_view);
                
                let (min_vals, max_vals) = calculate_min_max(&uv_data, 2);
                
                let uv_accessor = accessor::Accessor {
                    buffer_view: Some(Index::new(buffer_views.len() as u32 - 1)),
                    component_type: Checked::Valid(accessor::GenericComponentType(ComponentType::F32)),
                    count: gltf_json::validation::USize64(vertex_count as u64),
                    type_: Checked::Valid(accessor::Type::Vec2),
                    min: Some(serde_json::json!(min_vals)),
                    max: Some(serde_json::json!(max_vals)),
                    name: None,
                    normalized: false,
                    extras: Default::default(),
                    extensions: None,
                    byte_offset: None,
                    sparse: None,
                };
                let idx = accessors.len();
                accessors.push(uv_accessor);
                Some(idx)
            }
            _ => None,
        }
    } else {
        None
    };
    
    let normal_accessor_idx = if let Some(normal) = normal_attr {
        match normal {
            bevy_mesh::VertexAttributeValues::Float32x3(vals) => {
                let normal_data: Vec<f32> = vals.iter().flat_map(|v| v.iter().cloned()).collect();
                let normal_bytes = normal_data.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<u8>>();
                let normal_offset = binary_data.len();
                binary_data.extend_from_slice(&normal_bytes);
                
                let normal_view = buffer::View {
                    buffer: Index::new(0),
                    byte_offset: Some(gltf_json::validation::USize64(normal_offset as u64)),
                    byte_length: gltf_json::validation::USize64(normal_bytes.len() as u64),
                    target: Some(Checked::Valid(Target::ArrayBuffer)),
                    byte_stride: None,
                    name: None,
                    extras: Default::default(),
                    extensions: None,
                };
                buffer_views.push(normal_view);
                
                let (min_vals, max_vals) = calculate_min_max(&normal_data, 3);
                
                let normal_accessor = accessor::Accessor {
                    buffer_view: Some(Index::new(buffer_views.len() as u32 - 1)),
                    component_type: Checked::Valid(accessor::GenericComponentType(ComponentType::F32)),
                    count: gltf_json::validation::USize64(vertex_count as u64),
                    type_: Checked::Valid(accessor::Type::Vec3),
                    min: Some(serde_json::json!(min_vals)),
                    max: Some(serde_json::json!(max_vals)),
                    name: None,
                    normalized: false,
                    extras: Default::default(),
                    extensions: None,
                    byte_offset: None,
                    sparse: None,
                };
                let idx = accessors.len();
                accessors.push(normal_accessor);
                Some(idx)
            }
            _ => None,
        }
    } else {
        None
    };
    
    Ok((position_accessor_idx, indices_accessor_idx, uv_accessor_idx, normal_accessor_idx))
}

fn calculate_min_max(data: &[f32], components: usize) -> (Vec<f32>, Vec<f32>) {
    let mut min_vals = vec![f32::INFINITY; components];
    let mut max_vals = vec![f32::NEG_INFINITY; components];
    
    for chunk in data.chunks(components) {
        for (i, &val) in chunk.iter().enumerate() {
            min_vals[i] = min_vals[i].min(val);
            max_vals[i] = max_vals[i].max(val);
        }
    }
    
    (min_vals, max_vals)
}

fn calculate_min_max_indices(data: &[u8], component_type: ComponentType) -> (Vec<f64>, Vec<f64>) {
    let mut min_val = f64::INFINITY;
    let mut max_val = f64::NEG_INFINITY;
    
    match component_type {
        ComponentType::U8 => {
            for &val in data {
                let v = val as f64;
                min_val = min_val.min(v);
                max_val = max_val.max(v);
            }
        }
        ComponentType::U16 => {
            for i in (0..data.len()).step_by(2) {
                let val = u16::from_le_bytes([data[i], data[i + 1]]) as f64;
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }
        ComponentType::U32 => {
            for i in (0..data.len()).step_by(4) {
                let val = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as f64;
                min_val = min_val.min(val);
                max_val = max_val.max(val);
            }
        }
        _ => {}
    }
    
    (vec![min_val], vec![max_val])
}

fn export_texture(
    texture_handle: &Option<Handle<Image>>,
    images: &Assets<Image>,
    texture_map: &mut std::collections::HashMap<Handle<Image>, usize>,
    binary_data: &mut Vec<u8>,
    buffer_views: &mut Vec<gltf_json::buffer::View>,
    gltf_images: &mut Vec<gltf_json::image::Image>,
    gltf_textures: &mut Vec<gltf_json::texture::Texture>,
    gltf_samplers: &mut Vec<gltf_json::texture::Sampler>,
) -> Result<Option<gltf_json::Index<gltf_json::texture::Texture>>, String> {
    let Some(handle) = texture_handle.as_ref() else {
        info!("  export_texture: handle is None");
        return Ok(None);
    };
    
    if let Some(&tex_idx) = texture_map.get(handle) {
        info!("  export_texture: cached texture idx {}", tex_idx);
        return Ok(Some(gltf_json::Index::new(tex_idx as u32)));
    }
    
    let image = images.get(handle).ok_or("Texture not found")?;
    
    info!("  export_texture: exporting texture {:?}, size: {:?}", handle, image.texture_descriptor.size);
    
    let (png_data, mime_type) = image_to_png(image)?;
    
    let image_offset = binary_data.len();
    binary_data.extend_from_slice(&png_data);
    
    let image_buffer_view_idx = buffer_views.len();
    let image_view = gltf_json::buffer::View {
        buffer: gltf_json::Index::new(0),
        byte_offset: Some(gltf_json::validation::USize64(image_offset as u64)),
        byte_length: gltf_json::validation::USize64(png_data.len() as u64),
        target: None,
        byte_stride: None,
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    buffer_views.push(image_view);
    
    let gltf_image = gltf_json::image::Image {
        buffer_view: Some(gltf_json::Index::new(image_buffer_view_idx as u32)),
        uri: None,
        mime_type: Some(gltf_json::image::MimeType(mime_type.clone())),
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    let image_idx = gltf_images.len();
    gltf_images.push(gltf_image);
    
    let sampler = gltf_json::texture::Sampler {
        mag_filter: Some(Checked::Valid(gltf_json::texture::MagFilter::Linear)),
        min_filter: Some(Checked::Valid(gltf_json::texture::MinFilter::Linear)),
        wrap_s: Checked::Valid(gltf_json::texture::WrappingMode::ClampToEdge),
        wrap_t: Checked::Valid(gltf_json::texture::WrappingMode::ClampToEdge),
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    let sampler_idx = gltf_samplers.len();
    gltf_samplers.push(sampler);
    
    let gltf_texture = gltf_json::texture::Texture {
        sampler: Some(gltf_json::Index::new(sampler_idx as u32)),
        source: gltf_json::Index::new(image_idx as u32),
        name: None,
        extras: Default::default(),
        extensions: None,
    };
    let tex_idx = gltf_textures.len();
    gltf_textures.push(gltf_texture);
    
    texture_map.insert(handle.clone(), tex_idx);
    
    Ok(Some(gltf_json::Index::new(tex_idx as u32)))
}

fn image_to_png(image: &Image) -> Result<(Vec<u8>, String), String> {
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;
    
    let bytes = match &image.data {
        Some(d) => d,
        None => return Err("Image has no data".to_string()),
    };
    
    let mut img = image::ImageBuffer::new(width as u32, height as u32);
    
    for (i, pixel) in img.pixels_mut().enumerate() {
        let byte_idx = i * 4;
        if byte_idx + 3 < bytes.len() {
            let r = bytes[byte_idx];
            let g = bytes[byte_idx + 1];
            let b = bytes[byte_idx + 2];
            let a = bytes[byte_idx + 3];
            *pixel = image::Rgba([r, g, b, a]);
        }
    }
    
    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encoding failed: {}", e))?;
    
    Ok((png_data, "image/png".to_string()))
}

fn write_glb(doc: &Root, binary_data: &[u8], output_path: &str) -> Result<(), String> {
    let json_str = serde_json::to_string(doc)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    
    let json_bytes = json_str.into_bytes();
    let json_padded_length = json_bytes.len() + (4 - (json_bytes.len() % 4)) % 4;
    
    let bin_padded_length = binary_data.len() + (4 - (binary_data.len() % 4)) % 4;
    
    let total_length = 12 + 4 + 4 + json_padded_length + 4 + 4 + bin_padded_length;
    

    
    let mut glb = Vec::with_capacity(total_length as usize);
    
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());
    
    glb.extend_from_slice(&(json_padded_length as u32).to_le_bytes());
    glb.extend_from_slice(b"JSON");
    glb.extend_from_slice(&json_bytes);
    for _ in 0..((4 - (json_bytes.len() % 4)) % 4) {
        glb.push(0x20);
    }
    
    
    glb.extend_from_slice(&(bin_padded_length as u32).to_le_bytes());
    glb.extend_from_slice(b"BIN\0");
    glb.extend_from_slice(binary_data);
    for _ in 0..((4 - (binary_data.len() % 4)) % 4) {
        glb.push(0);
    }
    
    let mut file = File::create(output_path)
        .map_err(|e| format!("Cannot create file: {}", e))?;
    
    file.write_all(&glb)
        .map_err(|e| format!("Cannot write to file: {}", e))?;
    
    Ok(())
}
