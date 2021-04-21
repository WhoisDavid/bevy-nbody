use rand::Rng;
use rand_distr::{Distribution, UnitSphere};

mod nbody;
pub mod plugins;

use bevy::prelude::*;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::AmbientLight,
};

use nbody::{BodyBundle, Gravity, NBody};
use plugins::pan_orbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2.0,
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PanOrbitCameraPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(add_starry_background.system())
        // .add_startup_system(random_bodies.system())
        // .add_startup_system(figure8_bodies.system())
        .add_startup_system(solar_system.system())
        .add_plugin(NBody { speed_factor: 10.0 })
        .run()
}

fn spawn_z_camera(commands: &mut Commands, z: f32) {
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0.0, 0.0, z).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(PanOrbitCamera {
            radius: z,
            ..Default::default()
        });
}

fn spawn_z_light(commands: &mut Commands, z: f32, intensity: f32, range: f32) {
    commands.spawn_bundle(LightBundle {
        light: Light {
            intensity,
            range,
            ..Default::default()
        },
        // Slightly offset
        transform: Transform::from_xyz(0.0, z, z),
        ..Default::default()
    });
}

fn add_starry_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    let stars: Vec<Vec3> = UnitSphere
        .sample_iter(&mut rng)
        .take(1000)
        .map(|xyz| 800.0 * Vec3::new(xyz[0], xyz[1], xyz[2]))
        .collect();

    stars.into_iter().for_each(|s| {
        commands.spawn_bundle(PbrBundle {
            transform: Transform::from_translation(s),
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 2,
            })),
            material: materials.add(Color::WHITE.into()),
            ..Default::default()
        });
        // .insert(Light::default());
    })
}

/// Figure-8 solution
/// See: https://en.wikipedia.org/wiki/Three-body_problem#cite_note-11
/// ```
/// m = 1.0 / G
/// Pos: x1 = -x2 = 0.970004360 - 0.24308753i ; x3 = 0
/// Vel: vx3 = -2vx1= -2vx2 = - 0.93240737 - 0.86473146i
/// ```
pub fn figure8_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut g: ResMut<Gravity>,
) {
    // Set G = 1.0
    g.0 = 1.0;

    // For simplicity, we just accept the loss of precision for f32
    let pos = Vec3::new(0.970_004_4, -0.243_087_5, 0.0);
    let vel = Vec3::new(0.932_407_4, 0.864_731_5, 0.0);

    let bodies: Vec<(Vec3, Vec3, Color)> = vec![
        (pos, vel / 2.0, Color::BLUE),
        (-pos, vel / 2.0, Color::GREEN),
        (Vec3::ZERO, -vel, Color::RED),
    ];

    for (pos, vel, col) in bodies.into_iter() {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.1,
                    subdivisions: 5,
                })),
                material: materials.add(col.into()),
                ..Default::default()
            })
            .insert_bundle(BodyBundle::new(1.0, pos, vel));
    }

    spawn_z_camera(&mut commands, 5.0);
    spawn_z_light(&mut commands, 5.0, 200.0, 20.0);
}

pub fn random_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut g: ResMut<Gravity>,
) {
    // Set G = 1.0
    g.0 = 1.0;

    let mut rng = rand::thread_rng();

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 5,
            })),
            material: materials.add(Color::YELLOW.into()),
            ..Default::default()
        })
        .insert_bundle(BodyBundle::new(10_000.0, Vec3::ZERO, Vec3::ZERO));
    // .insert(Light {
    //     color: Color::ORANGE_RED,
    //     ..Default::default()
    // });

    (0..10).for_each(|_| {
        let pos = Vec3::new(
            rng.gen_range(-10.0..10.0),
            rng.gen_range(-10.0..10.0),
            rng.gen_range(-10.0..10.0),
        );

        let vel = Vec3::new(
            rng.gen_range(-10.0..50.0),
            rng.gen_range(-10.0..50.0),
            rng.gen_range(-10.0..50.0),
        );

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 0.5,
                    subdivisions: 5,
                })),
                material: materials.add(
                    Color::rgb(
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                    )
                    .into(),
                ),
                ..Default::default()
            })
            .insert_bundle(BodyBundle::new(1.0, pos, vel));
    });

    spawn_z_camera(&mut commands, 50.0);
    spawn_z_light(&mut commands, 10.0, 2000.0, 50.0);
}

/// Add the sun and all the planets of the Solar system (+ Pluto)
/// Units are scales:
/// Mass = 10^24 kg
/// Distance = AU (= 1.5 x 10^11 m)
/// Velocity = AU / Day
/// Acceleration = AU / DAY^2
pub fn solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut g: ResMut<Gravity>,
) {
    // Scale for rendering: 1 unit = 0.1 AU
    const AU_TO_UNIT_SCALE: f32 = 10.0;
    const DAY: f32 = 86_400.0;

    // Scale the gravitational constant accordingly to account for the units scaling
    // ```
    // G = m^3 / kg / s^2
    // G = (1.5^3 * 10^11 / 10 m)^3 / 10^24 kg / Day^2
    // G' = G * Day^2 * 10-6 / 1.5^3
    // ```
    g.0 *= DAY * DAY * 10.0f32.powi(-6) / 1.5f32.powi(3);

    let sun = BodyBundle::new(1_988_500.0, Vec3::ZERO, Vec3::ZERO);
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 2.8,
                subdivisions: 10,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::YELLOW.into(),
                roughness: 0.6,
                emissive: Color::YELLOW,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert_bundle(sun)
        .insert(Light {
            color: Color::WHITE,
            intensity: 50_000.0,
            range: 2000.0,
            ..Default::default()
        });

    macro_rules! spawn_planet {
    ($name:ident, m=$mass:literal, pos=($($pos:literal),+), vel=($($vel:literal),+), r=$radius:literal, col=$col:expr $(,)?) => {
        let $name = BodyBundle::new($mass, AU_TO_UNIT_SCALE * Vec3::new($($pos),+), AU_TO_UNIT_SCALE * Vec3::new($($vel),+));
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: $radius / 10_000.0,
                    subdivisions: 5,
                })),
                material: materials.add(StandardMaterial {
                    base_color: $col.into(),
                    roughness: 0.6,
                    reflectance: 0.1,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .insert_bundle($name);

    };
}
    // Data pulled from JPL Horizons as of 2021-04-18
    // https://ssd.jpl.nasa.gov/horizons.cgi
    #[rustfmt::skip]
    spawn_planet!(
        mercury,
        m=0.3302,
        pos=(3.044170697902298E-01, 1.295114876282963E-01, -1.734104195212369E-02),
        vel=(-1.648628006573339E-02, 2.713585294570181E-02, 3.729745700066048E-03),
        r=2440.0,
        col=Color::ORANGE_RED,
    );

    #[rustfmt::skip]
    spawn_planet!(
        venus,
        m=4.868,
        pos=(5.387247476293335E-01, 4.820230339302334E-01, -2.447215630265642E-02),
        vel=(-1.354845714410186E-02, 1.498631588335955E-02, 9.874886299710420E-04),
        r=6051.84,
        col=Color::ORANGE,
    );

    #[rustfmt::skip]
    spawn_planet!(
        earth,
        m=5.97219,
        pos=(-8.873674344461769E-01, -4.697992257377307E-01, 2.381003809013169E-05),
        vel=(7.775921491692710E-03, -1.526923260035268E-02, 1.329236295796724E-07),
        r=6371.01,
        col=Color::BLUE,
    );
    #[rustfmt::skip]
    spawn_planet!(
        mars,
        m=0.64171 ,
        pos=(-7.669365607923907E-01, 1.437715683938847E+00, 4.894216325150345E-02),
        vel=(-1.181841087219943E-02, -5.396860897762226E-03, 1.768153357356463E-04),
        r=3389.92,
        col=Color::RED,
    );
    #[rustfmt::skip]
    spawn_planet!(
        jupiter,
        m=1898.187,
        pos=(3.638338491378654E+00, -3.517196054099748E+00, -6.679350348303023E-02),
        vel=(5.159638546395391E-03, 5.787459942412818E-03, -1.394560955359292E-04),
        r=69911.0,
        col=Color::BISQUE,
    );
    #[rustfmt::skip]
    spawn_planet!(
        saturn,
        m=568.34,
        pos=(5.946821461107053E+00, -8.000786524501104E+00, -9.757186586148088E-02),
        vel=(4.173453543382942E-03, 3.320093983241896E-03, -2.235785645393874E-04),
        r=58232.0,
        col=Color::GOLD,
    );
    #[rustfmt::skip]
    spawn_planet!(
        uranus,
        m=86.813,
        pos=(1.507889019392361E+01, 1.276651492152234E+01, -1.479475386482554E-01),
        vel=(-2.565701401124483E-03, 2.824133197172000E-03, 4.363663945419187E-05),
        r=25362.0,
        col=Color::AQUAMARINE,
    );
    #[rustfmt::skip]
    spawn_planet!(
        neptune,
        m=102.4126,
        pos=(2.951580077181258E+01, -4.898113153026739E+00, -5.794227616270428E-01),
        vel=(4.988324362083494E-04, 3.122660147661985E-03, -7.542919141146281E-05),
        r=24622.0,
        col=Color::BLUE
    );
    #[rustfmt::skip]
    spawn_planet!(
        pluto,
        m=0.013030,
        pos=(1.437474170944128E+01, -3.109027718169479E+01, -8.297576366914019E-01),
        vel=(2.929346098298212E-03, 6.560315763737425E-04, -9.025427350060328E-04),
        r=11880.3,
        col=Color::GRAY,
    );

    spawn_z_camera(&mut commands, 500.0);
}
