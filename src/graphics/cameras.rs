use bevy::prelude::*;

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
        let mut store = CameraViewStore::default();
        store.views.push(CameraViewpoint::default());
        store.views.push(CameraViewpoint::default());
        app.init_resource::<CameraViewStore>();
        app.insert_resource(store);
        app.add_systems(Update, (orbit_camera_system, apply_camera_viewpoints).chain());
    }
}

#[allow(dead_code)]
fn collect_descendants(entity: Entity, children_query: &Query<&Children>, out: &mut Vec<Entity>) {
    if let Ok(children) = children_query.get(entity) {
        for &child in children {
            out.push(child);
            collect_descendants(child, children_query, out);
        }
    }
}

#[allow(dead_code)]
pub fn orbit_camera_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut view_store: ResMut<CameraViewStore>,
    mut query: Query<(Entity, &mut Transform, &mut EditorCamera, &CameraViewSlot)>,
) {
    let dt = time.delta_secs();
    let turn_speed = 2.0;
    let move_speed = 10.0;

    for (_cam_entity, mut transform, mut cam, slot) in &mut query {
        if keys.pressed(KeyCode::ArrowRight) {
            cam.target_yaw -= turn_speed * dt;
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            cam.target_yaw += turn_speed * dt;
        }
        if keys.pressed(KeyCode::ArrowUp) {
            let dir = cam.target_focus.normalize_or_zero();
            cam.target_focus -= dir * move_speed * dt;
        }
        if keys.pressed(KeyCode::ArrowDown) {
            let dir = cam.target_focus.normalize_or_zero();
            cam.target_focus += dir * move_speed * dt;
        }
        if keys.pressed(KeyCode::PageUp) {
            cam.target_focus.y += move_speed * dt;
        }
        if keys.pressed(KeyCode::PageDown) {
            cam.target_focus.y -= move_speed * dt;
        }

        let factor = 10.0 * dt;
        cam.focus = cam.focus.lerp(cam.target_focus, factor);
        cam.yaw = cam.yaw + (cam.target_yaw - cam.yaw) * factor;
        cam.pitch = cam.pitch + (cam.target_pitch - cam.pitch) * factor;

        if let Some(view) = view_store.views.get_mut(slot.0) {
            view.focus = cam.focus;
            view.target_focus = cam.target_focus;
            view.radius = cam.radius;
            view.yaw = cam.yaw;
            view.target_yaw = cam.target_yaw;
            view.pitch = cam.pitch;
            view.target_pitch = cam.target_pitch;
        }

        let rotation = Quat::from_axis_angle(Vec3::Y, cam.yaw) * Quat::from_axis_angle(Vec3::X, cam.pitch);
        let direction_vec = rotation * Vec3::Z;

        transform.translation = cam.focus + direction_vec * cam.radius;
        transform.look_at(cam.focus, Vec3::Y);
    }
}

#[allow(dead_code)]
pub fn apply_camera_viewpoints(
    view_store: Res<CameraViewStore>,
    mut query: Query<(&mut Transform, &CameraViewSlot), Without<EditorCamera>>,
) {
    for (mut transform, slot) in &mut query {
        if let Some(view) = view_store.views.get(slot.0) {
            let rotation = Quat::from_axis_angle(Vec3::Y, view.yaw) * Quat::from_axis_angle(Vec3::X, view.pitch);
            let direction_vec = rotation * Vec3::Z;

            transform.translation = view.focus + direction_vec * view.radius;
            transform.rotation = rotation;
        }
    }
}
