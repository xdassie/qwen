# Camera Message System Plan

## Problem
Camera controls are not working. Need to implement keyboard message consumption following the same pattern as `transform.rs`.

## Architecture

### Message Flow
```
keyboard.rs::keyboard_input_system (Producer)
  ↓ writes KeyboardEvent messages
  ↓
Message Bus
  ↓
keyboard.rs::keyboard_input_system continues reading (doesn't consume)
  ↓
transform.rs::camera_message_listener (Consumer)
  ↓
cameras.rs::camera_message_listener (Consumer)
```

### Key Design Decisions
1. **Messages are shared**: Multiple consumers can read the same batch of events
2. **Pattern match**: Follow `transform.rs` exactly for consistency
3. **Modifier keys**: Use Shift/Ctrl/Alt to control camera behavior
   - Arrow keys: Move/rotate camera based on modifiers
   - Shift + Arrow: Rotate camera view (yaw/pitch)
   - Ctrl + Arrow: Pan camera position
   - Alt + Arrow: Zoom in/out (radius adjustment)

## Implementation Steps

### Step 1: Fix `cameras.rs` - Add CameraListener component
```rust
use crate::input::keyboard::{KeyboardEvent, KeyboardListener};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CameraListener;

impl KeyboardListener<EditorCamera> for CameraListener {
    fn handle(&mut self, event: &KeyboardEvent, camera: &mut EditorCamera) {
        // Implement camera controls with modifier key support
        // - Arrow keys: Pan/rotate/zoom
        // - Modifiers: Change behavior
    }
}

// Add camera_message_listener system
pub fn camera_message_listener(
    mut reader: MessageReader<KeyboardEvent>,
    mut listeners: Query<(Entity, &mut Transform, &mut EditorCamera, &mut CameraListener), With<CameraListener>>,
) {
    for (_entity, mut transform, mut camera, mut listener) in listeners.iter_mut() {
        for event in reader.read() {
            listener.handle(&event, &mut *camera);
        }
    }
}
```

### Step 2: Update `CameraPlugin`
```rust
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Only register camera_message_listener
        // Remove keyboard_input_system - it duplicates keyboard.rs work
        app.add_systems(Update, camera_message_listener);
    }
}
```

### Step 3: Add CameraListener to camera in `bevy.rs::setup()`
```rust
commands.spawn((
    Camera3d { ..default() },
    Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    EditorCamera::default(),
    CameraListener,  // Add this component
));
```

### Step 4: Implement CameraListener::handle() with modifier support
```rust
impl KeyboardListener<EditorCamera> for CameraListener {
    fn handle(&mut self, event: &KeyboardEvent, camera: &mut EditorCamera) {
        // Get dt from a Res<Time> or use fixed delta
        let dt = 0.016; // ~60fps
        
        match event.code {
            // Shift + Arrow: Rotate camera view
            KeyCode::ArrowUp if event.modifiers.shift => {
                camera.target_pitch -= 2.0 * dt;
            }
            KeyCode::ArrowDown if event.modifiers.shift => {
                camera.target_pitch += 2.0 * dt;
            }
            KeyCode::ArrowLeft if event.modifiers.shift => {
                camera.target_yaw -= 2.0 * dt;
            }
            KeyCode::ArrowRight if event.modifiers.shift => {
                camera.target_yaw += 2.0 * dt;
            }
            
            // Ctrl + Arrow: Pan camera position
            KeyCode::ArrowUp if event.modifiers.ctrl => {
                let dir = camera.target_focus.normalize_or_zero();
                camera.target_focus.y += 50.0 * dt;
            }
            KeyCode::ArrowDown if event.modifiers.ctrl => {
                let dir = camera.target_focus.normalize_or_zero();
                camera.target_focus.y -= 50.0 * dt;
            }
            KeyCode::ArrowLeft if event.modifiers.ctrl => {
                let dir = camera.target_focus.normalize_or_zero();
                camera.target_focus -= dir * 50.0 * dt;
            }
            KeyCode::ArrowRight if event.modifiers.ctrl => {
                let dir = camera.target_focus.normalize_or_zero();
                camera.target_focus += dir * 50.0 * dt;
            }
            
            // Alt + Arrow: Zoom in/out
            KeyCode::ArrowUp if event.modifiers.alt => {
                camera.radius *= 1.1;
            }
            KeyCode::ArrowDown if event.modifiers.alt => {
                camera.radius *= 0.9;
            }
            
            // Alt alone: Reset zoom
            KeyCode::PageUp if event.modifiers.alt => {
                camera.radius = 10.0;
            }
            
            _ => {}
        }
    }
}
```

### Step 5: Keep existing camera_system for interpolation
The existing `camera_system` that lerps values should remain:
```rust
pub fn camera_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EditorCamera)>,
) {
    let dt = time.delta_secs();
    let factor = 10.0 * dt;
    
    for (_entity, mut transform, mut cam) in &mut query {
        // Lerp current values to target values
        cam.focus = cam.focus.lerp(cam.target_focus, factor);
        cam.yaw = cam.yaw.lerp(cam.target_yaw, factor);
        cam.pitch = cam.pitch.lerp(cam.target_pitch, factor);
        
        // Update transform
        let rotation = Quat::from_axis_angle(Vec3::Y, cam.yaw) 
                     * Quat::from_axis_angle(Vec3::X, cam.pitch);
        let direction_vec = rotation * Vec3::Z;
        
        transform.translation = cam.focus + direction_vec * cam.radius;
        transform.look_at(cam.focus, Vec3::Y);
    }
}
```

## Camera Control Summary

| Modifier | Key | Action |
|----------|-----|--------|
| None | Arrow Up | Pan camera up |
| None | Arrow Down | Pan camera down |
| None | Arrow Left | Pan camera left |
| None | Arrow Right | Pan camera right |
| Shift | Arrow Up/Down | Pitch camera (look up/down) |
| Shift | Arrow Left/Right | Yaw camera (rotate view) |
| Ctrl | Arrow Up/Down | Move focus Y position |
| Ctrl | Arrow Left/Right | Move focus X/Z position |
| Alt | Arrow Up/Down | Zoom in/out (radius) |
| Alt | Page Up | Reset zoom |

## Files to Modify

1. **src/graphics/cameras.rs**
   - Add `CameraListener` component
   - Implement `KeyboardListener<EditorCamera>` trait
   - Add `camera_message_listener` system
   - Keep existing `camera_system` for interpolation

2. **src/graphics/bevy.rs**
   - Add `CameraListener` component to camera spawn in `setup()`

3. **src/graphics/cameras.rs** (remove from CameraPlugin)
   - Remove `keyboard_input_system` from `CameraPlugin::build()`
   - Only register `camera_message_listener`

## Expected Behavior

1. Pressing arrow keys without modifiers should pan camera
2. Pressing Shift+Arrow should rotate camera view
3. Pressing Ctrl+Arrow should move camera focus position
4. Pressing Alt+Arrow should zoom in/out
5. Camera should smoothly interpolate (lerp) between positions
6. TransformListener and CameraListener should both receive events (no consumption)

## Testing Checklist

- [ ] Arrow keys pan camera position
- [ ] Shift+Arrow rotates camera view (yaw/pitch)
- [ ] Ctrl+Arrow moves camera focus
- [ ] Alt+Arrow zooms in/out
- [ ] Camera smoothly interpolates to target positions
- [ ] Both transform and camera respond to keyboard
- [ ] No compilation errors
- [ ] No duplicate event handling

## Next Steps

1. Write and implement `CameraListener`
2. Update `CameraPlugin` to only register `camera_message_listener`
3. Add `CameraListener` to camera in `setup()`
4. Build and test with arrow keys
5. Verify modifier key combinations work
6. Ensure both transform and camera receive events
