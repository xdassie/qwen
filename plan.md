# Level of Detail (LOD) Implementation Plan

## Overview
Implement a **2-level LOD system** with:
- **Mesh simplification** (high/low detail) via custom decimation algorithm
- **Fixed LOD level** controlled by command line flag (`--lod=low` or `--lod=high`)
- **Texture LOD** using mipmap levels based on the LOD setting

---

## Requirements (from Session ses_012b)

### User Specifications
1. **Mesh simplification approach**: Custom simplification algorithm (not pre-built loaders)
2. **LOD levels**: 2 levels (high/low binary switch)
3. **File format**: glTF/GLB files
4. **Texture LOD**: Fixed level of detail based on command line (not distance-based)
5. **Key constraint**: Simplify meshes **before loading** to avoid memory consumption

---

## Implementation Plan

### Phase 1: Implement Mesh Simplification (LOWEST RISK)

**New File**: `src/graphics/mesh_simplifier.rs`

**Purpose**: Simplify mesh geometry before asset loading to reduce memory usage

**Algorithm**: Distance-based mesh decimation

**Key Components**:
1. **Input Mesh Structure**:
   - Vertices array (3D positions)
   - Indices array (triangle connectivity)
   - Normals (for lighting)
   - UV coordinates (for textures)

2. **Simplification Strategy**:
   - Calculate mesh bounding box
   - Determine simplification ratio based on LOD level:
     - Low: ~50% vertex reduction
     - High: No reduction or minimal
   - Apply decimation algorithm:
     - Remove vertices with minimal impact on shape
     - Preserve edge quality
     - Maintain manifold properties

3. **Distance Field Mesh Simplification**:
   - Create distance field from mesh surface
   - Marching cubes or similar to extract simplified geometry
   - Iterate until target vertex count reached

4. **Output**:
   - Simplified mesh with reduced vertex/index count
   - Preserved normals and UVs
   - Same topology structure

**Functions to Implement**:
```rust
pub fn simplify_mesh(input_mesh: &Mesh, lod_level: LODLevel) -> Mesh {
    // Simplify based on LOD level
}

pub fn simplify_mesh_vertices(vertices: &[Vec3], indices: &[u32], 
                               lod_level: LODLevel) -> (Vec<Vec3>, Vec<u32>) {
    // Return simplified vertex/index arrays
}
```

**Micro-Iteration 1.3**: Implement index reassignment
- **Confidence**: Low (3/10)
- Complex algorithm, high risk of bugs
- 1. Add logic to rebuild index array after vertex removal
- 2. Ensure triangle connectivity is preserved
- 3. Run `cargo check`
- 4. If fail: verify index type (u32), check reindexing algorithm

**Micro-Iteration 1.1**: Create skeleton mesh_simplifier.rs
- **Confidence**: Medium (5/10)
- Requires understanding of Bevy mesh structures
- 1. Add basic function signatures with placeholder logic
- 2. Add necessary imports (Vec3, Mesh, etc.)
- 3. Run `cargo check`
- 4. If fail: verify imports are available, check function signature compatibility

**Micro-Iteration 1.2**: Implement vertex selection logic
- **Confidence**: Medium (5/10)
- Math-heavy, requires geometric understanding
- 1. Add helper to calculate vertex importance (distance to mesh center)
- 2. Add function to select vertices to remove
- 3. Run `cargo check`
- 4. If fail: verify Vec3 operations, check array indexing logic

**Micro-Iteration 1.4**: Add LOD level branching
- **Confidence**: High (8/10)
- Simple conditional logic
- 1. Implement conditional logic for high/low LOD
- 2. Apply different reduction ratios per LOD level
- 3. Run `cargo check`
- 4. If fail: verify LODLevel comparison, check conditional branches
- **Status**: ✅ COMPLETE

---

## Phase 1: Mesh Simplification - COMPLETE

---

### Phase 2: Texture LOD Integration

**New File**: `src/graphics/texture_lod.rs`

**Purpose**: Handle texture quality based on LOD setting

**Approach**: Mipmap levels + texture resolution switching

**Key Components**:
1. **Texture Quality Settings**:
   - Low LOD: Use lower resolution textures (e.g., 512x512)
   - High LOD: Use full resolution textures (e.g., 2048x2048)
   - Generate mipmaps for all textures

2. **Implementation Strategy**:
   - Load textures at appropriate resolution based on `LODLevel`
   - Enable mipmap generation in Bevy texture settings
   - Use `TextureDescriptor` to control resolution

3. **Functions to Implement**:
```rust
pub fn get_texture_resolution(lod_level: LODLevel) -> (u32, u32) {
    // Return appropriate texture size
}

pub fn load_texture_with_lod(path: &str, lod_level: LODLevel) -> Handle<Sampler> {
    // Load texture at LOD-appropriate resolution
}
```

**Micro-Iteration 2.1**: Create skeleton texture_lod.rs
- **Confidence**: Medium (5/10)
- Requires understanding of Bevy texture API
- 1. Add basic texture resolution functions
- 2. Add texture loading helpers
- 3. Run `cargo check`
- 4. If fail: verify imports, check Bevy texture API

**Micro-Iteration 2.3**: Integrate with scene loader
- **Confidence**: Medium (6/10)
- Requires modifying existing scene_loader
- 1. Modify scene_loader to use texture LOD functions
- 2. Pass LOD level to texture loading
- 3. Run `cargo check`
- 4. If fail: verify function integration, check parameter passing

**Micro-Iteration 2.2**: Implement resolution mapping
- **Confidence**: High (8/10)
- Simple lookup table logic
- 1. Create resolution lookup table for LOD levels
- 2. Add function to return texture size
- 3. Run `cargo check`
- 4. If fail: verify constant definitions, check tuple returns

---

### Phase 3: Modify Scene Loading

**File**: `src/graphics/scene_loader.rs`

**Changes**:
1. Add `LODLevel` resource to imports
2. In `load_glb()`:
   - Check `LODLevel` before loading
   - If LOW LOD: simplify meshes before asset loading
   - Load with simplified geometry
3. Store LOD mode in scene metadata

**Integration Points**:
```rust
impl SceneLoader for ModelSceneLoader {
    fn load_glb(
        // ... existing params ...
        lod_level: Res<LODLevel>,
    ) {
        // Load mesh data
        // If low LOD, apply simplification
        // Load simplified mesh to asset server
    }
}
```

**Micro-Iteration 3.2**: Add simplification call site
- **Confidence**: Medium (6/10)
- Requires understanding of existing loading flow
- 1. Insert call to mesh simplifier before loading
- 2. Handle both high and low LOD paths
- 3. Run `cargo check`
- 4. If fail: verify simplifier function is accessible, check return type

**Micro-Iteration 3.3**: Integrate with asset server
- **Confidence**: Medium (6/10)
- Depends on Bevy asset API familiarity
- 1. Load simplified mesh data
- 2. Create proper asset handles
- 3. Run `cargo check`
- 4. If fail: verify asset_server API, check handle creation

**Micro-Iteration 3.1**: Add imports and signature updates
- **Confidence**: High (8/10)
- Straightforward import and signature change
- 1. Add `LODLevel` to imports
- 2. Update function signature to accept LOD level
- 3. Run `cargo check`
- 4. If fail: verify import path, check function parameter types

---

### Phase 4: Add LOD Level Configuration

**File**: `src/graphics/bevy.rs`

**Command line usage**:
- `--lod=high` (default)
- `--lod=low`

**Micro-Iteration 4.2**: Parse `--lod` command line argument
- **Confidence**: Medium (6/10)
- Depends on existing argument parsing pattern
- 1. Add argument parsing in `get_model_path()` or new `parse_lod_level()` function
- 2. Use existing `args` pattern or add new clap argument
- 3. Run `cargo check`
- 4. If fail: verify argument parser is imported, check type compatibility

**Micro-Iteration 4.1**: Add `LODLevel` enum
- **Confidence**: High (8/10)
- Simple enum with derive attributes, minimal complexity
- 1. Add enum definition near other resource definitions
- 2. Add `#[derive(Resource)]` attribute
- 3. Run `cargo check`
- 4. If fail: check error message, verify derive attributes match Bevy resource requirements

**Micro-Iteration 4.3**: Make `LODLevel` a Bevy resource
- **Confidence**: High (8/10)
- Straightforward resource initialization
- 1. Initialize resource in `run()` function
- 2. Pass to scene loader systems
- 3. Run `cargo check`
- 4. If fail: verify resource is properly inserted, check system ordering

---

### Phase 5: System Integration

**File**: `src/graphics/bevy.rs`

**Changes**:
1. Add `LODLevel` resource initialization
2. Pass LOD level to scene loader systems
3. Add command line parsing for `--lod` flag
4. Integrate with existing texture quality system

**System Updates**:
```rust
fn run() {
    let lod_level = parse_lod_level();
    
    app.init_resource::<LODLevel>();
    app.insert_resource(lod_level);
    
    // Pass to scene loader
    // Pass to texture loader
}
```

---

## File Structure

```
src/graphics/
├── bevy.rs                    # Main app setup (MODIFY)
├── scene_loader.rs            # Scene loading (MODIFY)
├── mesh_simplifier.rs         # NEW - Mesh simplification
├── texture_lod.rs             # NEW - Texture LOD handling
├── mod.rs                     # Module exports
├── cameras.rs
├── recorder.rs
└── transform.rs
```

---

## Data Structures

### LODLevel Enum
```rust
#[derive(Resource, Clone, Copy, Debug, PartialEq, Default)]
pub enum LODLevel {
    #[default]
    High,
    Low,
}
```

### Mesh Simplification Parameters
```rust
struct MeshSimplificationParams {
    pub target_vertex_ratio: f32,  // 0.0 to 1.0
    pub preserve_normals: bool,
    pub preserve_uvs: bool,
    pub maintain_topology: bool,
}
```

### Texture Resolution Map
```rust
const TEXTURE_RESOLUTIONS: [(LODLevel, u32, u32); 2] = [
    (LODLevel::High, 2048, 2048),
    (LODLevel::Low, 512, 512),
];
```

---

## Testing Strategy

1. **Unit Tests**:
   - Mesh simplification preserves topology
   - Vertex count reduction matches expectations
   - Normals/UVs preserved after simplification

2. **Integration Tests**:
   - Scene loads successfully at both LOD levels
   - Memory usage is lower at low LOD
   - Textures load at correct resolutions

3. **Command Line Tests**:
   - `--lod=high` loads high detail
   - `--lod=low` loads low detail
   - Default behavior (high LOD) works

---

## Success Criteria

- ✅ Meshes are simplified before loading (not at runtime)
- ✅ Memory consumption is reduced at low LOD
- ✅ Command line controls LOD level
- ✅ Textures match LOD level resolution
- ✅ Visual quality is acceptable at both levels
- ✅ No performance degradation from simplification logic

## Confidence Metric Guide

- **High (8-10/10)**: Simple, straightforward implementation, low risk
- **Medium (5-7/10)**: Moderate complexity, some unknowns
- **Low (1-4/10)**: Complex, high risk of bugs or API issues

## Cargo Check Failure Recovery

When `cargo check` fails:

1. **Read error message carefully**
   - Look for the specific line/column
   - Identify: missing import, type mismatch, API change

2. **Check imports**
   - Verify all required types are imported
   - Add `use bevy::...` as needed

3. **Verify API compatibility**
   - Check Bevy docs for correct function signatures
   - Ensure derive macros match trait requirements

4. **Type compatibility**
   - Confirm return types match function declarations
   - Check parameter types are correct

5. **If stuck, isolate**
   - Comment out recent changes
   - Binary search to find problematic code
   - Revert and re-add incrementally

6. **Documentation**
   - Check Bevy's `cargo doc` output
   - Review relevant trait definitions

---

## Notes

- **Memory Strategy**: Pre-simplify during loading to avoid runtime overhead
- **Quality Tradeoff**: Low LOD prioritizes memory over visual fidelity
- **Algorithm Choice**: Distance-based decimation for quality preservation
- **Future Enhancement**: Could add more LOD levels or distance-based switching
