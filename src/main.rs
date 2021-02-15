use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use rand::Rng;
use bevy::input::mouse::{MouseButtonInput, MouseMotion};
use bevy::input::ElementState;
use bevy::render::camera::{ActiveCameras, PerspectiveProjection};
use std::ops::Mul;
use bevy::input::keyboard::KeyboardInput;

use bevy::reflect::TypeUuid;

pub const BULLET_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 13148362314412771389);

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "CS434 lab1".to_string(),
            cursor_visible: false,
            cursor_locked: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_system(fan_rotation_system.system())
        .add_system(mouse_fin_destruction_system.system())
        .run();
}

struct Windmill {
    state: usize,
    fins: [Option<Entity>; 3]
}
struct WindmillFin {
    index: usize,
}
struct Bullet {
    dir: Vec3
};

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // add entities to the world

    let mut rng = rand::thread_rng();
    let windmill_rod_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.1,
        rings: 5,
        depth: 5.0,
        latitudes: 16,
        longitudes: 32,
        uv_profile: Default::default()
    }));
    let windmill_fan_mesh = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.05,
        rings: 5,
        depth: 1.0,
        latitudes: 16,
        longitudes: 32,
        uv_profile: Default::default()
    }));

    meshes.set_untracked(BULLET_MESH_HANDLE, Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 16 }));
    for i in 0..10 {
        let x = rng.gen_range(-15.0..15.0);
        let z = rng.gen_range(-15.0..15.0);
        let mut fins: [Option<Entity>; 3] = [None; 3];
        for i in 0..3 {
            commands.spawn(PbrBundle {
                mesh: windmill_fan_mesh.clone(),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(x, 2.0, z),
                ..Default::default()
            })
                .with(WindmillFin {
                    index: i,
                });;
            fins[i] = commands.current_entity();
        }
        commands.spawn(PbrBundle {
            mesh: windmill_rod_mesh.clone(),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(x, 2.0, z),
            ..Default::default()
        })
            .with(Windmill {
                state: 0,
                fins
            });
    }
    commands
        // plane
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        // light
        .spawn(LightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        // camera
        .spawn(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::default(), Vec3::unit_y()),
            ..Default::default()
        })
        .with(FlyCamera::default());
}

fn fan_rotation_system(
    commands: &mut Commands,
    time: Res<Time>,
    windmill_query: Query<(&Windmill, &Transform)>,
    mut windmill_fins_query: Query<(&WindmillFin, &mut Transform)>
) {
    for (windmill, windmill_transform) in windmill_query.iter() {
       for fin_entity in windmill.fins.iter() {
           if let Some(entity) = fin_entity {
               let (fin, mut fin_transform) = windmill_fins_query.get_mut(*entity).unwrap();
               let angle = time.seconds_since_startup() as f32 + (fin.index as f32 * std::f32::consts::FRAC_PI_3 * 2.0);
               fin_transform.rotation = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), angle);
               fin_transform.translation = windmill_transform.translation;
               fin_transform.translation.y += 2.5;
               fin_transform.translation.x -= angle.sin() * 0.5;
               fin_transform.translation.y += angle.cos() * 0.5;
           }
       }
    }
}

fn mouse_fin_destruction_system(
    mut commands: &mut Commands,
    mut windows: ResMut<Windows>,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    active_cameras: Res<ActiveCameras>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<(&Transform), With<PerspectiveProjection>>,
    mut windmill_query: Query<(Entity, &mut Windmill, &Transform)>,
) {
    let window = windows.get_primary_mut().unwrap();
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::Escape {
                window.set_cursor_lock_mode(false);
                window.set_cursor_visibility(true);
            }
        }
    }

    let camera = if let Some(camera) = active_cameras.get("Camera3d") {
        camera
    } else {
        return;
    };



    // Calculate bomb location
    let camera_transform = camera_query.get(camera).unwrap();
    let ray = camera_transform.rotation.mul(Vec3::new(0.0, 0.0, 1.0));

    commands.spawn(PbrBundle {
        mesh: BULLET_MESH_HANDLE.typed(),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        ..Default::default()
    });


    for event in mouse_button_input_events.iter() {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
        match event {
            MouseButtonInput {
                button: MouseButton::Left,
                state: ElementState::Pressed,
            } => {
                for (entity, mut windmill, transform) in windmill_query.iter_mut() {
                    if (transform.translation.x - x).abs() < 0.5 && (transform.translation.z - z).abs() < 0.5 {
                        // hit!
                        let fin_to_destroy_index = windmill.state;
                        if fin_to_destroy_index == 3 {
                            commands.despawn(entity);
                            break;
                        }
                        let fin_to_destroy = windmill.fins[fin_to_destroy_index].take().unwrap();
                        windmill.state += 1;
                        commands.despawn(fin_to_destroy);
                        break;
                    }
                }
            },
            _ => (),
        }
    }

}
