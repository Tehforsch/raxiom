#![allow(incomplete_features)]
#![feature(generic_const_exprs, adt_const_params, hash_drain_filter)]
#![feature(const_fn_floating_point_arithmetic)]
// Some or our '*_system' functions have a large number of arguments.
// That is not necessarily a bad thing, as they are auto-provided by bevy.
#![allow(clippy::too_many_arguments)]
// Some of the Query<…> types appear rather complex to clippy, but are actually
// perfectly readable.
#![allow(clippy::type_complexity)]

pub mod app_builder;
pub(crate) mod command_line_options;
pub mod communication;
pub(crate) mod config;
pub(crate) mod domain;
pub(crate) mod initial_conditions;
pub(crate) mod io;
pub(crate) mod mass;
pub(crate) mod named;
pub(crate) mod parameters;
pub(crate) mod particle;
pub(crate) mod physics;
pub(crate) mod plugin_utils;
pub(crate) mod position;
pub(crate) mod quadtree;
pub(crate) mod stages;
pub mod units;
pub(crate) mod velocity;
pub(crate) mod visualization;

#[cfg(feature = "mpi")]
pub mod mpi_log;
