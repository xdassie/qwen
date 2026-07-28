# Plan: Fix check_clicks Function

## Current Issues
1. **Line 377**: Always logs the same message format regardless of click location
2. **Lines 361-377**: No distinction between clicking an object vs empty space
3. **Line 377**: When `clicked_entity` is None, logs `clicked marker #0` which is misleading

## Required Changes

### Change 1: Differentiate Click Messages (Line 377)
**Before:**
```rust
println!("Click {} - clicked marker #{} at position ({:.2}, {:.2}, {:.2}) (marker #{})", 
         mode.click_count, clicked_marker, spawn_pos.x, spawn_pos.y, spawn_pos.z, marker_count + 1);
```

**After:**
```rust
if let Some(entity) = clicked_entity {
    println!("Click {} - hit object at marker #{} (marker #{}) position ({:.2}, {:.2}, {:.2})", 
             mode.click_count, clicked_marker, marker_count + 1, spawn_pos.x, spawn_pos.y, spawn_pos.z);
} else {
    println!("Click {} - empty space (no object hit)", mode.click_count);
}
```

### Change 2: Fix Marker Counting Logic (Lines 373-377)
**Current Issue:** When clicking empty space, `Marker(marker_count + 1)` is still spawned, which is incorrect.

**Fix:** Only spawn marker when object is clicked:
```rust
if let Some(entity) = clicked_entity {
    let Ok((_, transform, _)) = markers.get(entity) else {
        return;
    };
    let spawn_pos = transform.translation;
    
    commands.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(marker_material),
        Marker(marker_count + 1),
        Transform::from_translation(spawn_pos),
    ));
    
    println!("Click {} - hit object at marker #{} (marker #{}) position ({:.2}, {:.2}, {:.2})", 
             mode.click_count, clicked_marker, marker_count + 1, spawn_pos.x, spawn_pos.y, spawn_pos.z);
} else {
    println!("Click {} - empty space (no object hit)", mode.click_count);
}
```

## Implementation Steps
1. Remove the else branch that spawns marker at Vec3::ZERO (lines 367-369)
2. Wrap the existing spawn code in an `if let Some(entity) = clicked_entity` block
3. Add an `else` branch for empty space clicks
4. Update the println! to reflect the conditional logic

## Expected Behavior After Fix
- **Object click**: "Click 1 - hit object at marker #0 (marker #1) position (0.00, 0.00, 0.00)"
- **Empty space click**: "Click 1 - empty space (no object hit)"
- No marker spawned on empty space clicks
