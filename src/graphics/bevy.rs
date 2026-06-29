use bevy::prelude::*;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy! Spinning Cube".into(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, spin_cube)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add ambient light for base visibility
    commands.spawn(AmbientLight {
        color: Color::srgb(0.5, 0.5, 0.5),
        brightness: 500.0,
        ..default()
    });

    // Add directional light
    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 1000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn the cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.8))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add camera with proper 3D configuration
    commands.spawn((
        Camera3d {
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spin_cube(
    time: Res<Time>,
    mut cube_query: Query<&mut Transform, With<Mesh3d>>,
) {
    for mut transform in cube_query.iter_mut() {
        transform.rotate_z(time.delta_secs() * 2.0);
    }
}
