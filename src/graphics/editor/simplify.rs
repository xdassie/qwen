use bevy::prelude::*;
use bevy_asset::RenderAssetUsages;
use bevy_mesh::{Mesh, Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::pbr::StandardMaterial;
use crate::input::keyboard::KeyboardEvent;
use crate::graphics::editor::{SelectionState, SceneEntities};

pub struct SimplifyPlugin;

impl Plugin for SimplifyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, simplify_selected_meshes);
    }
}

fn simplify_selected_meshes(
    mut keys: MessageReader<KeyboardEvent>,
    selection: Res<SelectionState>,
    scene: Res<SceneEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mesh_query: Query<(Entity, &mut Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>)>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
) {
    for key in keys.read() {
        if key.code == KeyCode::KeyS && key.pressed {
            println!("S key pressed, selected entities: {:?}", selection.selected);
            for &entity in &selection.selected {
                if !scene.entities.contains(&entity) {
                    continue;
                }
                
                simplify_entity(entity, &mut meshes, &mut mesh_query, &parents, &children);
            }
        }
    }
}

fn simplify_entity(
    entity: Entity,
    meshes: &mut Assets<Mesh>,
    mesh_query: &mut Query<(Entity, &mut Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>)>,
    parents: &Query<&ChildOf>,
    children: &Query<&Children>,
) {
    println!("Simplifying entity: {:?}", entity);
    
    let mut to_process: Vec<Entity> = vec![entity];
    
    while let Some(current) = to_process.pop() {
        if let Ok(children_ref) = children.get(current) {
            for &child in children_ref {
                to_process.push(child);
            }
        }
        
        if let Ok((_child_entity, mut mesh3d, material)) = mesh_query.get_mut(current) {
            println!("  Found mesh: {:?}, material: {:?}", current, material.is_some());
            if material.is_none() {
                println!("    WARNING: No material found on this entity!");
            }
            
            let mesh_handle = &mesh3d.0;
            
            let Some(mesh) = meshes.get(mesh_handle) else {
                println!("  No mesh found for handle");
                continue;
            };
            
            let Some(vertex_buffer) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
                println!("  No position attribute found");
                continue;
            };
            
            let uv_buffer = mesh.attribute(Mesh::ATTRIBUTE_UV_0).and_then(|uv| {
                if let VertexAttributeValues::Float32x2(values) = uv {
                    let uvs = values.clone();
                    println!("  Found {} UV coordinates", uvs.len());
                    Some(uvs)
                } else {
                    println!("  No UV_0 attribute found");
                    None
                }
            });
            let index_buffer = mesh.indices().cloned();
            
            if vertex_buffer.len() == 0 {
                println!("  No vertices");
                continue;
            }
            
            println!("  Original vertex count: {}", vertex_buffer.len());
            
            let cluster_size = compute_cluster_size(vertex_buffer);
            
            let simplified = cluster_vertices(vertex_buffer, uv_buffer, index_buffer, cluster_size);
            
            println!("  Simplified vertex count: {}", simplified.vertices.len());
            
            let old_handle = mesh_handle.clone();
            let new_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
            let new_handle = meshes.add(new_mesh);
            
            let new_mesh_mut = meshes.get_mut(&new_handle).unwrap();
            
            new_mesh_mut.insert_attribute(Mesh::ATTRIBUTE_POSITION, simplified.vertices);
            new_mesh_mut.insert_attribute(Mesh::ATTRIBUTE_NORMAL, simplified.normals);
            
            if let Some(uvs) = simplified.uvs {
                new_mesh_mut.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            }
            
            if !simplified.indices.is_empty() {
                new_mesh_mut.insert_indices(Indices::U32(simplified.indices));
            }
            
            mesh3d.0 = new_handle;
            meshes.remove(&old_handle);
            println!("  Updated Mesh3d, removed old mesh");
        }
    }
}

struct SimplifiedMesh {
    vertices: Vec<[f32; 3]>,
    uvs: Option<Vec<[f32; 2]>>,
    indices: Vec<u32>,
    normals: Vec<[f32; 3]>,
}

fn compute_cluster_size(vertices: &VertexAttributeValues) -> f32 {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    
    if let VertexAttributeValues::Float32x3(values) = vertices {
        for [x, y, z] in values {
            min = min.min(Vec3::new(*x, *y, *z));
            max = max.max(Vec3::new(*x, *y, *z));
        }
    }
    
    let size = (max - min).length();
    size * 0.01
}

fn cluster_vertices(vertices: &VertexAttributeValues, uvs: Option<Vec<[f32; 2]>>, indices: Option<Indices>, cluster_size: f32) -> SimplifiedMesh {
    let indices = match indices {
        Some(Indices::U16(buf)) => buf.iter().map(|&x| x as u32).collect(),
        Some(Indices::U32(buf)) => buf,
        None => (0..vertices.len() as u32).collect(),
    };
    
    let positions = match vertices {
        VertexAttributeValues::Float32x3(values) => values,
        _ => &Vec::new(),
    };
    
    let mut clusters: Vec<[f32; 3]> = Vec::new();
    let mut cluster_uvs: Vec<[f32; 2]> = Vec::new();
    let mut vertex_map: Vec<usize> = vec![0; positions.len()];
    
    for (i, pos) in positions.iter().enumerate() {
        let pos = *pos;
        let mut found_cluster = false;
        
        for (j, cluster_center) in clusters.iter().enumerate() {
            let dist = distance(&pos, cluster_center);
            if dist < cluster_size {
                vertex_map[i] = j;
                let weight = 1.0 / dist.max(0.001);
                
                for k in 0..3 {
                    clusters[j][k] = (clusters[j][k] * j as f32 + pos[k] * weight) / (j as f32 + weight);
                }
                
                if let Some(ref uvs) = uvs {
                    if i < uvs.len() {
                        let uv = uvs[i];
                        for k in 0..2 {
                            cluster_uvs[j][k] = (cluster_uvs[j][k] * j as f32 + uv[k] * weight) / (j as f32 + weight);
                        }
                    }
                }
                
                found_cluster = true;
                break;
            }
        }
        
        if !found_cluster {
            vertex_map[i] = clusters.len();
            clusters.push(pos);
            if let Some(ref uvs) = uvs {
                if i < uvs.len() {
                    cluster_uvs.push(uvs[i]);
                } else {
                    cluster_uvs.push([0.0, 0.0]);
                }
            }
        }
    }
    
    let mut new_indices = Vec::new();
    let mut triangle_set: std::collections::HashSet<(u32, u32, u32)> = std::collections::HashSet::new();
    
    let num_clusters = clusters.len();
    let mut normals: Vec<[f32; 3]> = vec![[0.0; 3]; num_clusters];
    
    for i in (0..indices.len()).step_by(3) {
        if i + 2 >= indices.len() {
            break;
        }
        
        let idx0 = indices[i] as usize;
        let idx1 = indices[i + 1] as usize;
        let idx2 = indices[i + 2] as usize;
        
        if idx0 >= vertex_map.len() || idx1 >= vertex_map.len() || idx2 >= vertex_map.len() {
            continue;
        }
        
        let v0 = vertex_map[idx0];
        let v1 = vertex_map[idx1];
        let v2 = vertex_map[idx2];
        
        if v0 == v1 || v1 == v2 || v0 == v2 {
            continue;
        }
        
        let triangle = if v0 < v1 {
            if v1 < v2 { (v0 as u32, v1 as u32, v2 as u32) }
            else if v0 < v2 { (v0 as u32, v2 as u32, v1 as u32) }
            else { (v2 as u32, v0 as u32, v1 as u32) }
        } else {
            if v0 < v2 { (v1 as u32, v0 as u32, v2 as u32) }
            else if v1 < v2 { (v1 as u32, v2 as u32, v0 as u32) }
            else { (v2 as u32, v1 as u32, v0 as u32) }
        };
        
        if !triangle_set.contains(&triangle) {
            triangle_set.insert(triangle);
            new_indices.push(triangle.0);
            new_indices.push(triangle.1);
            new_indices.push(triangle.2);
            
            let p0 = clusters[v0];
            let p1 = clusters[v1];
            let p2 = clusters[v2];
            
            let edge1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let edge2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            
            let cross = [
                edge1[1] * edge2[2] - edge1[2] * edge2[1],
                edge1[2] * edge2[0] - edge1[0] * edge2[2],
                edge1[0] * edge2[1] - edge1[1] * edge2[0],
            ];
            
            let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            if len > 0.0001 {
                let inv_len = 1.0 / len;
                for k in 0..3 {
                    normals[v0][k] += cross[k] * inv_len;
                    normals[v1][k] += cross[k] * inv_len;
                    normals[v2][k] += cross[k] * inv_len;
                }
            }
        }
    }
    
    for normal in normals.iter_mut() {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        if len > 0.0001 {
            let inv_len = 1.0 / len;
            normal[0] *= inv_len;
            normal[1] *= inv_len;
            normal[2] *= inv_len;
        } else {
            *normal = [0.0, 1.0, 0.0];
        }
    }
    
    SimplifiedMesh {
        vertices: clusters,
        uvs: if uvs.is_some() { Some(cluster_uvs) } else { None },
        indices: new_indices,
        normals,
    }
}

fn distance(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}
