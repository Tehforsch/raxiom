use bevy::prelude::Commands;
use bevy::prelude::Res;
use bevy::prelude::*;
use mpi::Rank;

use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::position::Position;
use crate::units::f32::kilograms;
use crate::units::f32::second;
use crate::units::vec2::meter;
use crate::velocity::Velocity;

pub struct InitialConditionsPlugin;

impl Plugin for InitialConditionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_particles_system);
    }
}

fn spawn_particles_system(mut commands: Commands, rank: Res<Rank>) {
    if *rank != 0 {
        return;
    }
    let upper_left = meter(Vec2::new(0.0, 0.0));
    let lower_right = meter(Vec2::new(0.5, 0.5));
    for i in 0..200 {
        let pos = upper_left + (lower_right - upper_left) * i as f32;
        let vel = pos / second(1.0);
        let vel = crate::units::vec2::Velocity::new(vel.y(), -vel.x());
        commands.spawn().insert_bundle(LocalParticleBundle::new(
            Position(pos),
            Velocity(vel),
            Mass(kilograms(1.0)),
        ));
    }
}
