# Plan: Fix Unused Code Warnings and Refactor Recorder Architecture

## Problem Analysis

### Unused Code Warning
- **Location**: `src/graphics/bevy.rs:18-20`
- **Issue**: `limit_reached` method defined but never called
- **Root Cause**: Inline check at line 29 bypasses trait method:
  ```rust
  let limit_reached = frame_count >= self.get_screenshot_limit();  // Line 29 - duplicates logic
  ```

### Architectural Violation
- `ModelSceneLoader` implements `Recorder` trait but shouldn't know about screenshots
- Scene loading and screenshot recording are orthogonal concerns
- Trait methods (`limit_reached`, `capture_screenshot`, `get_screenshot_limit`) are defined but bypassed

## Solution: Refactor to Separate Concerns

### 1. Keep `Recorder` Trait (src/graphics/bevy.rs:6-11)

```rust
pub trait Recorder {
    fn record(&self, frame_count: u64);
    fn limit_reached(&self, frame_count: u64) -> bool;
    fn capture_screenshot(&self, frame_count: u64, commands: Commands, window_query: Query<Entity, With<Window>>);
    fn get_screenshot_limit(&self) -> u64;
}
```

**Rationale**: 
- `record()` is called at line 30 inside `capture_screenshot` impl (even though bypassed by inline check)
- `limit_reached()` serves architectural purpose of encapsulating check logic
- `capture_screenshot()` is the core method - must be called, not bypassed
- `get_screenshot_limit()` is used by `limit_reached()` - needed for trait contract

### 2. Create New `ScreenshotRecorder` Resource

Add new struct that implements `Recorder` trait fully:

```rust
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
        println!("DEBUG: Capturing screenshot {}", frame_count);
        
        for _window_entity in window_query.iter() {
            let filename = format!("/tmp/screenshot_{}.png", frame_count);
            commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                .observe(bevy::render::view::screenshot::save_to_disk(filename));
        }
    }
    
    fn get_screenshot_limit(&self) -> u64 {
        self.screenshot_limit
    }
}
```

**Placement**: After trait definition in `src/graphics/bevy.rs`, before `SceneLoader` trait

### 3. Remove Screenshot Logic from `ModelSceneLoader`

Keep only scene-loading methods:
- `load_glb()` - loads GLB model
- `on_glb_loaded()` - callback after GLB loaded

Remove from `impl Recorder for ModelSceneLoader`:
- `record()` - moved to `ScreenshotRecorder`
- `limit_reached()` - moved to `ScreenshotRecorder`
- `capture_screenshot()` - moved to `ScreenshotRecorder`
- `get_screenshot_limit()` - moved to `ScreenshotRecorder`

### 4. Update `capture_screenshot` System Function

Current (WRONG - uses loader for screenshots):
```rust
fn capture_screenshot(
    commands: Commands,
    window_query: Query<Entity, With<Window>>,
    mut frame_count: ResMut<FrameCount>,
    loader: Res<ModelSceneLoader>,  // ❌ Wrong resource
) {
    frame_count.0 += 1;
    let current_frame = frame_count.0;
    loader.capture_screenshot(current_frame, commands, window_query);  // ❌ Wrong call
}
```

Refactored (CORRECT - uses dedicated recorder):
```rust
fn capture_screenshot(
    commands: Commands,
    window_query: Query<Entity, With<Window>>,
    mut frame_count: ResMut<FrameCount>,
    recorder: Res<ScreenshotRecorder>,  // ✅ Correct resource
) {
    frame_count.0 += 1;
    let current_frame = frame_count.0;
    
    // ✅ Call trait method - no inline duplication
    if recorder.limit_reached(current_frame) {
        println!("Screenshot limit reached: {}", recorder.get_screenshot_limit());
        std::process::exit(0);
    }
    
    recorder.capture_screenshot(current_frame, commands, window_query);  // ✅ Call trait method
}
```

### 5. Update `run()` Function

Add `ScreenshotRecorder` initialization:

```rust
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let limit = /* ... existing limit parsing logic ... */;
    
    let loader = ModelSceneLoader::new(&get_model_path(), limit as u64);
    let recorder = ScreenshotRecorder::new(limit as u64);  // ✅ NEW
    
    let mut app = App::new();
    app.init_resource::<GltfLoadingState>();
    app.init_resource::<FrameCount>();
    app.insert_resource(loader);
    app.insert_resource(recorder);  // ✅ NEW RESOURCE
    
    if limit > 0 {
        app.add_systems(Update, (update_scene, spin_model, capture_screenshot));
    }
    
    // ... rest of plugin setup ...
}
```

### 6. Update `mod.rs`

Add new struct to module exports:

```rust
pub mod bevy;
pub use bevy::*;  // This will include ScreenshotRecorder
pub mod cameras;
```

## Implementation Order

1. **Step 1**: Add `ScreenshotRecorder` struct and impl to `bevy.rs` (after trait definition)
2. **Step 2**: Remove `impl Recorder for ModelSceneLoader` entirely (keep only `SceneLoader` impl)
3. **Step 3**: Update `capture_screenshot` system function to use `ScreenshotRecorder`
4. **Step 4**: Update `run()` to insert `ScreenshotRecorder` resource
5. **Step 5**: Run `cargo build` to verify no warnings

## Expected Outcome

### Before
```
warning: method `limit_reached` is never used
  = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
```

### After
- ✅ No unused warnings
- ✅ `ModelSceneLoader` only handles scene loading
- ✅ `ScreenshotRecorder` only handles screenshots
- ✅ `Recorder` trait is the sole source of screenshot logic
- ✅ No inline duplication
- ✅ `limit_reached()` is called by system function
- ✅ All functionality preserved

## Verification Steps

After implementation:
1. `cargo build` - should compile without warnings
2. Check `bevy.rs` for:
   - `ScreenshotRecorder` struct present
   - `impl Recorder for ScreenshotRecorder` complete
   - `impl Recorder for ModelSceneLoader` removed
   - `capture_screenshot` system uses `recorder` resource
   - `run()` inserts `ScreenshotRecorder`
3. Test screenshot capture works as before
4. Verify frame count and limit logic work correctly

## Files to Modify

| File | Changes |
|------|---------|
| `src/graphics/bevy.rs` | Add `ScreenshotRecorder`, remove `impl Recorder for ModelSceneLoader`, update `capture_screenshot` system, update `run()` |
| `src/graphics/mod.rs` | No change needed (already re-exports via `pub use bevy::*`) |
| `src/main.rs` | No change needed |
| `Cargo.toml` | No change needed |
| `plan.md` | This document |
