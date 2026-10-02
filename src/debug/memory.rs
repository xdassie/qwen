use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::mesh::{Mesh, VertexAttributeValues};
use std::collections::HashMap;
use std::fs;

#[derive(Resource, Default)]
pub struct MemoryTracker {
    pub gpu_memory: GpuMemory,
    pub asset_memory: AssetMemory,
    pub ecs_memory: EcsMemory,
    pub process_memory: ProcessMemory,
    top_allocations: HashMap<String, usize>,
}

#[derive(Default, Debug, Clone)]
pub struct ProcessMemory {
    pub rss_kb: u64,
    pub virtual_kb: u64,
    pub peak_kb: u64,
}

#[derive(Default, Debug, Clone)]
pub struct GpuMemory {
    pub adapter_name: String,
    pub total_memory_bytes: u64,
    pub dedicated_memory_bytes: u64,
    pub buffer_count: usize,
    pub texture_count: usize,
    pub estimated_usage_bytes: u64,
}

#[derive(Default, Debug, Clone)]
pub struct AssetMemory {
    pub mesh_count: usize,
    pub texture_count: usize,
    pub scene_count: usize,
    pub total_bytes: u64,
}

#[derive(Default, Debug, Clone)]
pub struct EcsMemory {
    pub entity_count: usize,
    pub component_type_count: usize,
    pub estimated_bytes: u64,
}

impl MemoryTracker {
    pub fn log_summary(&self) {
        info!("=== Memory Summary ===");
        
        info!("Process Memory:");
        info!("  RSS: {:.2} MB", self.process_memory.rss_kb as f64 / 1024.0);
        info!("  Virtual: {:.2} MB", self.process_memory.virtual_kb as f64 / 1024.0);
        info!("  Peak: {:.2} MB", self.process_memory.peak_kb as f64 / 1024.0);
        
        info!("GPU Memory ({}):", self.gpu_memory.adapter_name);
        info!("  Total: {:.2} GB", self.gpu_memory.total_memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
        info!("  Dedicated: {:.2} GB", self.gpu_memory.dedicated_memory_bytes as f64 / 1024.0 / 1024.0 / 1024.0);
        info!("  Buffers: {}", self.gpu_memory.buffer_count);
        info!("  Textures: {}", self.gpu_memory.texture_count);
        info!("  Estimated Usage: {:.2} MB", self.gpu_memory.estimated_usage_bytes as f64 / 1024.0 / 1024.0);
        
        info!("Asset Memory:");
        info!("  Meshes: {}", self.asset_memory.mesh_count);
        info!("  Textures: {}", self.asset_memory.texture_count);
        info!("  Scenes: {}", self.asset_memory.scene_count);
        info!("  Total: {:.2} MB", self.asset_memory.total_bytes as f64 / 1024.0 / 1024.0);
        
        info!("ECS Memory:");
        info!("  Entities: {}", self.ecs_memory.entity_count);
        info!("  Component Types: {}", self.ecs_memory.component_type_count);
        info!("  Estimated: {:.2} MB", self.ecs_memory.estimated_bytes as f64 / 1024.0 / 1024.0);
        
        if !self.top_allocations.is_empty() {
            info!("Top Allocations:");
            let mut sorted: Vec<_> = self.top_allocations.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            for (name, size) in sorted.iter().take(10) {
                info!("  {}: {:.2} MB", name, **size as f64 / 1024.0 / 1024.0);
            }
        }
    }
}

pub fn query_gpu_memory(adapter_info: &RenderAdapterInfo) -> GpuMemory {
    let info = &adapter_info.0;
    
    GpuMemory {
        adapter_name: info.name.clone(),
        total_memory_bytes: 0,
        dedicated_memory_bytes: 0,
        buffer_count: 0,
        texture_count: 0,
        estimated_usage_bytes: 0,
    }
}

fn vertex_attribute_size(values: &VertexAttributeValues) -> usize {
    match values {
        VertexAttributeValues::Float32(v) => v.len() * std::mem::size_of::<f32>(),
        VertexAttributeValues::Sint32(v) => v.len() * std::mem::size_of::<i32>(),
        VertexAttributeValues::Uint32(v) => v.len() * std::mem::size_of::<u32>(),
        VertexAttributeValues::Float32x2(v) => v.len() * std::mem::size_of::<[f32; 2]>(),
        VertexAttributeValues::Sint32x2(v) => v.len() * std::mem::size_of::<[i32; 2]>(),
        VertexAttributeValues::Uint32x2(v) => v.len() * std::mem::size_of::<[u32; 2]>(),
        VertexAttributeValues::Float32x3(v) => v.len() * std::mem::size_of::<[f32; 3]>(),
        VertexAttributeValues::Sint32x3(v) => v.len() * std::mem::size_of::<[i32; 3]>(),
        VertexAttributeValues::Uint32x3(v) => v.len() * std::mem::size_of::<[u32; 3]>(),
        VertexAttributeValues::Float32x4(v) => v.len() * std::mem::size_of::<[f32; 4]>(),
        VertexAttributeValues::Sint32x4(v) => v.len() * std::mem::size_of::<[i32; 4]>(),
        VertexAttributeValues::Uint32x4(v) => v.len() * std::mem::size_of::<[u32; 4]>(),
        VertexAttributeValues::Sint16x2(v) => v.len() * std::mem::size_of::<[i16; 2]>(),
        VertexAttributeValues::Snorm16x2(v) => v.len() * std::mem::size_of::<[i16; 2]>(),
        VertexAttributeValues::Uint16x2(v) => v.len() * std::mem::size_of::<[u16; 2]>(),
        VertexAttributeValues::Unorm16x2(v) => v.len() * std::mem::size_of::<[u16; 2]>(),
        VertexAttributeValues::Sint16x4(v) => v.len() * std::mem::size_of::<[i16; 4]>(),
        VertexAttributeValues::Snorm16x4(v) => v.len() * std::mem::size_of::<[i16; 4]>(),
        VertexAttributeValues::Uint16x4(v) => v.len() * std::mem::size_of::<[u16; 4]>(),
        VertexAttributeValues::Unorm16x4(v) => v.len() * std::mem::size_of::<[u16; 4]>(),
        VertexAttributeValues::Sint8x2(v) => v.len() * std::mem::size_of::<[i8; 2]>(),
        VertexAttributeValues::Snorm8x2(v) => v.len() * std::mem::size_of::<[i8; 2]>(),
        VertexAttributeValues::Uint8x2(v) => v.len() * std::mem::size_of::<[u8; 2]>(),
        VertexAttributeValues::Unorm8x2(v) => v.len() * std::mem::size_of::<[u8; 2]>(),
        VertexAttributeValues::Sint8x4(v) => v.len() * std::mem::size_of::<[i8; 4]>(),
        VertexAttributeValues::Snorm8x4(v) => v.len() * std::mem::size_of::<[i8; 4]>(),
        VertexAttributeValues::Uint8x4(v) => v.len() * std::mem::size_of::<[u8; 4]>(),
        VertexAttributeValues::Unorm8x4(v) => v.len() * std::mem::size_of::<[u8; 4]>(),
    }
}

pub fn query_asset_memory(meshes: &Assets<Mesh>, textures: &Assets<Image>) -> AssetMemory {
    let mut total_bytes = 0u64;
    
    for (_id, mesh) in meshes.iter() {
        for (_, values) in mesh.attributes() {
            total_bytes += vertex_attribute_size(values) as u64;
        }
        
        if let Some(indices) = mesh.indices() {
            match indices {
                bevy::mesh::Indices::U16(v) => total_bytes += v.len() as u64 * std::mem::size_of::<u16>() as u64,
                bevy::mesh::Indices::U32(v) => total_bytes += v.len() as u64 * std::mem::size_of::<u32>() as u64,
            }
        }
    }
    
    for (_id, texture) in textures.iter() {
        if let Some(data) = &texture.data {
            total_bytes += data.len() as u64;
        }
    }
    
    AssetMemory {
        mesh_count: meshes.len(),
        texture_count: textures.len(),
        scene_count: 0,
        total_bytes,
    }
}

pub fn query_ecs_memory(world: &mut World) -> EcsMemory {
    let entity_count = world.query::<EntityRef>().iter(world).count();
    
    EcsMemory {
        entity_count,
        component_type_count: 0,
        estimated_bytes: entity_count as u64 * 100,
    }
}

#[derive(Resource)]
pub struct AllocationTracker {
    allocations: HashMap<String, usize>,
}

impl Default for AllocationTracker {
    fn default() -> Self {
        Self {
            allocations: HashMap::new(),
        }
    }
}

impl AllocationTracker {
    pub fn track(&mut self, name: &str, size: usize) {
        *self.allocations.entry(name.to_string()).or_insert(0) += size;
    }
    
    pub fn get_top_allocations(&self, count: usize) -> Vec<(&String, &usize)> {
        let mut sorted: Vec<_> = self.allocations.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        sorted.truncate(count);
        sorted
    }
}

pub fn query_process_memory() -> ProcessMemory {
    let pid = std::process::id();
    let mut process_memory = ProcessMemory::default();
    
    if let Ok(status) = fs::read_to_string(format!("/proc/{}/status", pid)) {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    process_memory.rss_kb = parts[1].parse().ok().unwrap_or(0);
                }
            } else if line.starts_with("VmSize:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    process_memory.virtual_kb = parts[1].parse().ok().unwrap_or(0);
                }
            } else if line.starts_with("VmPeak:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    process_memory.peak_kb = parts[1].parse().ok().unwrap_or(0);
                }
            }
        }
    }
    
    process_memory
}
