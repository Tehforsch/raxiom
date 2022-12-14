use std::marker::PhantomData;

use bevy::ecs::schedule::SystemDescriptor;
use bevy::prelude::*;

use super::close_file_system;
use super::make_output_dirs_system;
use super::open_file_system;
use super::parameters::OutputParameters;
use super::timer::Timer;
use super::write_used_parameters_system;
use super::OutputFile;
use super::OutputStages;
use crate::named::Named;
use crate::prelude::Simulation;
use crate::simulation::RaxiomPlugin;

pub(crate) trait IntoOutputSystem {
    fn system() -> SystemDescriptor;
}

#[derive(SystemLabel)]
struct OutputSystemLabel;

#[derive(Named)]
pub struct OutputPlugin<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for OutputPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T> RaxiomPlugin for OutputPlugin<T>
where
    T: IntoOutputSystem + Named,
{
    fn allow_adding_twice(&self) -> bool {
        true
    }

    fn should_build(&self, sim: &Simulation) -> bool {
        sim.write_output
    }

    fn build_once_on_main_rank(&self, sim: &mut Simulation) {
        sim.add_startup_system(make_output_dirs_system)
            .add_startup_system(write_used_parameters_system.after(make_output_dirs_system));
    }

    fn build_once_everywhere(&self, sim: &mut Simulation) {
        sim.add_parameter_type::<OutputParameters>()
            .insert_resource(OutputFile::default())
            .add_startup_system(Timer::initialize_system)
            .add_system_to_stage(
                OutputStages::Output,
                open_file_system.with_run_criteria(Timer::run_criterion),
            )
            .add_system_to_stage(
                OutputStages::Output,
                close_file_system
                    .after(open_file_system)
                    .with_run_criteria(Timer::run_criterion),
            )
            .add_system_to_stage(
                OutputStages::Output,
                Timer::update_system
                    .after(close_file_system)
                    .with_run_criteria(Timer::run_criterion),
            );
    }

    fn build_everywhere(&self, sim: &mut Simulation) {
        if OutputParameters::is_desired_field::<T>(sim) {
            sim.add_system_to_stage(
                OutputStages::Output,
                T::system()
                    .after(open_file_system)
                    .before(close_file_system)
                    .with_run_criteria(Timer::run_criterion)
                    .label(OutputSystemLabel)
                    .ambiguous_with(OutputSystemLabel),
            );
        }
    }
}
