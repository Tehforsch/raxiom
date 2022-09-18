use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use rand::Rng;
use serde::Deserialize;

use crate::communication::WorldRank;
use crate::input;
use crate::parameters::ParameterPlugin;
use crate::particle::LocalParticleBundle;
use crate::plugin_utils::get_parameters;
use crate::position::Position;
use crate::units::DVec2Length;
use crate::units::DVec2Velocity;
use crate::units::Mass;
use crate::units::VecLength;
use crate::units::VecVelocity;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Parameters {
    Random(usize),
    EarthSun,
    Read(input::Parameters),
}

impl Parameters {
    pub fn unwrap_read(&self) -> &input::Parameters {
        match self {
            Self::Read(parameters) => parameters,
            _ => panic!("Called unwrap_read on other variant"),
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::Read(input::Parameters::default())
    }
}

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ParameterPlugin::<Parameters>::new("initial_conditions"));
        let parameters = get_parameters::<Parameters>(app);
        match parameters {
            Parameters::Random(_) => {
                app.add_startup_system(spawn_particles_system);
            }
            Parameters::EarthSun => {
                app.add_startup_system(spawn_solar_system_system);
            }
            Parameters::Read(_) => {}
        };
    }
}

fn spawn_particle(commands: &mut Commands, pos: VecLength, vel: VecVelocity, mass: Mass) {
    commands.spawn().insert_bundle(LocalParticleBundle::new(
        Position(pos),
        Velocity(vel),
        crate::mass::Mass(mass),
    ));
}

fn spawn_particles_system(
    mut commands: Commands,
    parameters: Res<Parameters>,
    rank: Res<WorldRank>,
) {
    if !rank.is_main() {
        return;
    }
    let num_particles = match *parameters {
        Parameters::Random(num_particles) => num_particles,
        _ => unreachable!(),
    };
    let mut rng = rand::thread_rng();
    for _ in 0..num_particles {
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let pos = 0.10 * DVec2Length::astronomical_unit(x, y);
        let x = rng.gen_range(-1.0..1.0);
        let y = rng.gen_range(-1.0..1.0);
        let vel = 0.04 * DVec2Velocity::astronomical_unit_per_day(x, y);
        spawn_particle(&mut commands, pos, vel, Mass::solar(0.01))
    }
}

fn spawn_solar_system_system(mut commands: Commands, rank: Res<WorldRank>) {
    if !rank.is_main() {
        return;
    }
    let positions: Vec<VecLength> = vec![
        VecLength::astronomical_unit(0.0, 0.0),
        VecLength::astronomical_unit(0.7, 0.7),
    ];
    let masses: Vec<Mass> = vec![Mass::solar(1.0), Mass::earth(1.0)];
    let mass_ratio = masses[0] / masses[1];
    let mass_ratio = mass_ratio.value();
    let velocity: Vec<VecVelocity> = vec![
        VecVelocity::astronomical_unit_per_day(-1e-2 / mass_ratio, 1e-2 / mass_ratio),
        VecVelocity::astronomical_unit_per_day(1e-2, -1e-2),
    ];
    for ((pos, vel), mass) in positions.into_iter().zip(velocity).zip(masses) {
        spawn_particle(&mut commands, pos, vel, mass)
    }
}
