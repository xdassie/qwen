# Plan: Move Scene Loading Functions to Trait

## Goal
Move `load_scene` and `update_scene` functions from `bevy.rs` to the `SceneLoader` trait, keeping consistency with the `capture_screenshot` pattern.

## Current Issues in bevy.rs
1. Lines 21-34: Duplicate `FrameCount` struct and `get_model_path()` function
2. Lines 58-60: Empty system closure with comment
3. Lines 66-68: Malformed `else {` block (no matching `if`)
4. Line 62-68: Duplicate `.add_systems(Update, spin_model)` call

## Minimal Changes Needed

### Step 0: Spike - Quick Fix & Verify
1. Make minimal fix to get `cargo check` passing
2. Verify the trait methods are correctly called
3. If it compiles, proceed with full cleanup
4. If it fails, update plan based on error

### Step 1: Remove duplicate code
- Delete lines 21-34 (duplicate definitions)

### Step 2: Fix system registration
Replace:
```rust
app.add_systems(Startup, setup);
app.add_systems(Startup, || {
    // load_scene is now a trait method on ModelSceneLoader
});
```

With:
```rust
app.add_systems(Startup, setup)
    .add_systems(Startup, |loader: Res<ModelSceneLoader>, asset_server: Res<AssetServer>, mut state: ResMut<GltfLoadingState>| {
        ModelSceneLoader::load_glb(&loader, &asset_server, &mut state);
    });
```

### Step 3: Fix Update system chain
Replace:
```rust
app.add_systems(Update, spin_model)
    .add_systems(Update, |scenes: Res<Assets<Scene>>, mut commands: Commands, time: Res<Time>, loader: Res<ModelSceneLoader>, state: ResMut<GltfLoadingState>| {
        // Call trait method directly
        ModelSceneLoader::on_glb_loaded(&loader, &scenes, &mut state, &mut commands, &time);
    }); else {
    app.add_systems(Update, spin_model);
}
```

With:
```rust
app.add_systems(Update, spin_model)
    .add_systems(Update, |scenes: Res<Assets<Scene>>, mut commands: Commands, time: Res<Time>, loader: Res<ModelSceneLoader>, state: ResMut<GltfLoadingState>| {
        ModelSceneLoader::on_glb_loaded(&loader, &scenes, &mut state, &mut commands, &time);
    });
```

### Step 4: Verify compilation
Run: `cargo check`

## Expected Result
- `load_scene` and `update_scene` functions removed from `bevy.rs`
- Trait methods `load_glb()` and `on_glb_loaded()` called inline
- Consistent with `capture_screenshot` pattern
- Code compiles without errors

## Files to Modify
- `/home/dave/sync/test/src/rust/qwen/src/graphics/bevy.rs`

## Verification
- `cargo check` passes
- No warnings about unused functions
- Build succeeds: `cargo build --release`
