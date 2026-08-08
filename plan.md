# Plan: Fix Module Separation - Keyboard Cannot Depend on Transform

## Problem
The `KeyboardListener` trait is generic (keyboard input handling) but lives in `src/graphics/transform.rs` (transform-specific location). Additionally, `keyboard.rs` imports `TransformListener` for the `keyboard_message_listener` system, creating a dependency where input module depends on graphics module.

## Solution
Move the `KeyboardListener` trait to `src/input/keyboard.rs` where it belongs. Move the `keyboard_message_listener` system to `src/graphics/transform.rs` so the input module has NO dependencies on the transform module.

## Architecture
- **keyboard.rs** (producer): Contains `KeyboardEvent`, `KeyboardListener` trait, `keyboard_input_system` only. No imports from transform.
- **transform.rs** (consumer): Contains `TransformListener` component, `impl KeyboardListener`, `keyboard_message_listener` system. Imports `KeyboardListener` from input.

## Changes

### 1. src/input/keyboard.rs
- Keep `KeyboardEvent`, `ModifierKeys`, `KeyboardListener` trait, `keyboard_input_system`
- **Remove** `keyboard_message_listener` system (moves to transform module)
- No imports from `crate::graphics::*`

### 2. src/graphics/transform.rs
- Add `use crate::input::keyboard::KeyboardListener;` import
- Keep `TransformListener` component
- Keep `impl KeyboardListener for TransformListener`
- **Add** `keyboard_message_listener` system here (consumer of keyboard events)

### 3. src/input/mod.rs
- Add `pub use keyboard::KeyboardListener;` to exports

### 4. src/graphics/mod.rs
- Remove `pub use transform::KeyboardListener;` (trait no longer exported from graphics)
- Keep `pub use transform::TransformListener;` (component stays in graphics)

## Result
- `KeyboardListener` trait: lives in `src/input/keyboard.rs`, generic name matches location
- `TransformListener` component: lives in `src/graphics/transform.rs`, specific name matches location
- **NO dependencies**: `keyboard.rs` does not import from `transform.rs`
- Dependency flow: transform module depends on input module (transform imports KeyboardListener from input)
