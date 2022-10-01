use serde::Deserialize;

use crate::named::Named;
use crate::quadtree::QuadTreeConfig;
use crate::units::Length;

/// Parameters for hydrodynamics. Only needed if the
/// [HydrodynamicsPlugin](crate::physics::hydrodynamics::HydrodynamicsPlugin)
/// is added to the simulation
#[derive(Deserialize, Named)]
#[name = "hydrodynamics"]
#[serde(deny_unknown_fields)]
pub struct HydrodynamicsParameters {
    pub smoothing_length: Length,
    /// Parameters of the tree used for the neighbour search in the
    /// hydrodynamic density and force calculation. See
    /// [QuadTreeConfig](crate::quadtree::QuadTreeConfig)
    #[serde(default = "default_hydro_tree")]
    pub tree: QuadTreeConfig,
}

fn default_hydro_tree() -> QuadTreeConfig {
    QuadTreeConfig {
        min_depth: 0,
        max_depth: 20,
        max_num_particles_per_leaf: 30,
    }
}
