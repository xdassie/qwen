use bevy::prelude::*;
use crate::input::keyboard::{KeyboardEvent, KeyboardListener};

#[allow(dead_code)]
#[derive(Component, Clone, Copy)]
pub struct EditorCamera {
    pub focus: Vec3,
    pub target_focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub target_yaw: f32,
    pub pitch: f32,
    pub target_pitch: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CameraViewpoint {
    pub focus: Vec3,
    pub target_focus: Vec3,
    pub radius: f32,
    pub yaw: f32,
    pub target_yaw: f32,
    pub pitch: f32,
    pub target_pitch: f32,
}

impl Default for CameraViewpoint {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            target_focus: Vec3::ZERO,
            radius: 10.0,
            yaw: 0.0,
            target_yaw: 0.0,
            pitch: 0.0,
            target_pitch: 0.0,
        }
    }
}

#[allow(dead_code)]
#[derive(Resource, Default)]
pub struct CameraViewStore {
    pub views: Vec<CameraViewpoint>,
}

#[allow(dead_code)]
#[derive(Component, Clone, Copy, Debug)]
pub struct CameraViewSlot(pub usize);

#[allow(dead_code)]
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CameraListener;

impl KeyboardListener<EditorCamera> for CameraListener {
    fn handle(&mut self, event: &KeyboardEvent, camera: &mut EditorCamera) {
        let dt = 0.016;
        
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

impl Default for EditorCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            target_focus: Vec3::ZERO,
            radius: 10.0,
            yaw: 0.0,
            target_yaw: 0.0,
            pitch: 0.0,
            target_pitch: 0.0,
        }
    }
}

#[allow(dead_code)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (camera_message_listener, camera_system));
    }
}

pub fn camera_message_listener(
    mut reader: MessageReader<KeyboardEvent>,
    mut listeners: Query<(Entity, &mut Transform, &mut EditorCamera, &mut CameraListener), With<CameraListener>>,
) {
    for (_entity, mut transform, mut camera, mut listener) in listeners.iter_mut() {
        // Only read once per entity so TransformListener can also receive events
        if let Some(event) = reader.read().next() {
            listener.handle(event, &mut *camera);
        }
    }
}

#[allow(dead_code)]
pub fn camera_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut EditorCamera)>,
) {
    let dt = time.delta_secs();
    let factor = 10.0 * dt;
    
    for (entity, mut transform, mut cam) in &mut query {
        cam.focus = cam.focus.lerp(cam.target_focus, factor);
        cam.yaw = cam.yaw.lerp(cam.target_yaw, factor);
        cam.pitch = cam.pitch.lerp(cam.target_pitch, factor);

        let rotation = Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);
        let direction_vec = rotation * Vec3::Z;

        transform.translation = cam.focus + direction_vec * cam.radius;
        transform.rotation = rotation;
        transform.look_at(cam.focus, Vec3::Y);
    }
}
