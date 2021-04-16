pub mod plugins;

use bevy::core::FixedTimestep;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

use plugins::pan_orbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

pub const G: f32 = 6.67430e-11_f32;
const TIMESTEP: f32 = 0.01;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(add_bodies.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIMESTEP as f64))
                .with_system(
                    update_acceleration
                        .system()
                        .label(PhysicsSystem::UpdateAcceleration),
                )
                .with_system(
                    update_velocity
                        .system()
                        .label(PhysicsSystem::UpdateVelocity)
                        .after(PhysicsSystem::UpdateAcceleration),
                )
                .with_system(
                    movement
                        .system()
                        .label(PhysicsSystem::Movement)
                        .after(PhysicsSystem::UpdateVelocity),
                ),
        )
        .run()
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum PhysicsSystem {
    UpdateAcceleration,
    UpdateVelocity,
    Movement,
}
struct Body {
    mass: f32,
}

#[derive(Default)]
struct Position(Vec3);

#[derive(Default)]
struct Velocity(Vec3);
#[derive(Default)]
struct Acceleration(Vec3);

#[derive(Bundle)]
struct BodyBundle {
    _b: Body,
    pos: Position,
    vel: Velocity,
    acc: Acceleration,

    #[bundle]
    pbr: PbrBundle,
}

fn add_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut spawn_body = |pos: Vec3, vel: Vec3, col: Color| {
        commands.spawn_bundle(BodyBundle {
            _b: Body { mass: 1.0 / G },
            pos: Position(pos),
            vel: Velocity(vel),
            acc: Acceleration::default(),
            pbr: PbrBundle {
                transform: Transform::from_translation(pos),
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.1,
                    subdivisions: 5,
                })),
                material: materials.add(col.into()),
                ..Default::default()
            },
        });
    };

    /*
    Figure-8 solution
    See: https://en.wikipedia.org/wiki/Three-body_problem#cite_note-11
    m = 1.0 / G
    Pos: x1 = -x2 = 0.970004360 - 0.24308753i ; x3 = 0
    Vel: vx3 = -2vx1= -2vx2 = - 0.93240737 - 0.86473146i
    */

    // We just accept the loss of precision for f32
    let pos = Vec3::new(0.970_004_4, -0.243_087_5, 0.0);
    let vel = Vec3::new(0.932_407_4, 0.864_731_5, 0.0);

    spawn_body(pos, vel / 2.0, Color::GREEN);
    spawn_body(-pos, vel / 2.0, Color::RED);
    spawn_body(Vec3::ZERO, -vel, Color::BLUE);

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(0.0, 5.0, 5.0),
        ..Default::default()
    });

    // Fix camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PanOrbitCamera::default());
}

/// Newton's law of universal gravitation
/// ```
/// F = G*m1*m2/r^2
/// ```
/// where:
/// - `F` is the gravitational force acting between two objects
/// - `G` is the gravitational constant
/// - `m1` and `m2` are the masses of the objects
/// - `r` is the distance between the centers of their masses
fn update_acceleration(mut query: Query<(&Body, &Position, &mut Acceleration)>) {
    let mut bodies: Vec<(&Body, &Position, Mut<Acceleration>)> = Vec::new();
    for (body, pos, mut acc) in query.iter_mut() {
        acc.0 = Vec3::ZERO;
        for (other_body, other_pos, other_acc) in bodies.iter_mut() {
            if let Some(mut force) = (other_pos.0 - pos.0).try_normalize() {
                let magnitude =
                    G * body.mass * other_body.mass / (pos.0 - other_pos.0).length_squared();
                force *= magnitude;
                acc.0 += force;
                other_acc.0 -= force;
            }
        }
        bodies.push((body, pos, acc));
    }

    // Newton's second law of motion: `F = ma => a = F/m`
    for (body, _, acc) in bodies.iter_mut() {
        acc.0 /= body.mass;
    }
}

fn update_velocity(time: Res<Time>, mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut vel, acc) in query.iter_mut() {
        vel.0 += acc.0 * time.delta_seconds().min(TIMESTEP); // (TIMESTEP as f32);
    }
}

fn movement(time: Res<Time>, mut query: Query<(&mut Position, &mut Transform, &Velocity)>) {
    for (mut pos, mut transform, vel) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds().min(TIMESTEP);
        transform.translation = pos.0;
    }
}
