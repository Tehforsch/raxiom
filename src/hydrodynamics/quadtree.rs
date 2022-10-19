use bevy::prelude::*;

use super::parameters::HydrodynamicsParameters;
use crate::components::Position;
use crate::components::SmoothingLength;
use crate::domain::GlobalExtent;
use crate::prelude::MVec;
use crate::prelude::Particles;
use crate::quadtree::LeafDataType;
use crate::quadtree::NodeDataType;
use crate::quadtree::{self};
use crate::units::Length;
use crate::units::VecLength;

pub type QuadTree = quadtree::QuadTree<NodeData, LeafData>;

#[derive(Debug, Clone)]
pub struct LeafData {
    pub entity: Entity,
    pub pos: VecLength,
    pub smoothing_length: Length,
}

#[derive(Debug, Default)]
pub struct NodeData {
    pub largest_smoothing_length: Length,
}

impl LeafDataType for LeafData {
    fn pos(&self) -> &VecLength {
        &self.pos
    }
}

impl NodeDataType<LeafData> for NodeData {
    fn update_with(&mut self, leaf: &LeafData) {
        self.largest_smoothing_length = self.largest_smoothing_length.max(leaf.smoothing_length);
    }
}

fn bounding_boxes_overlap(
    pos1: &VecLength,
    size1: &VecLength,
    pos2: &VecLength,
    size2: &VecLength,
) -> bool {
    (pos1.x() - pos2.x()).abs() < size1.x() + size2.x()
        && (pos1.y() - pos2.y()).abs() < size1.y() + size2.y()
}

fn add_particles_in_box<'a>(
    particles: &mut Vec<&'a LeafData>,
    tree: &'a QuadTree,
    pos: &VecLength,
    side_length: &Length,
) {
    let node_extent = tree.extent.side_lengths()
        + VecLength::from_vector_and_scale(MVec::ONE, tree.data.largest_smoothing_length);
    if bounding_boxes_overlap(
        &tree.extent.center(),
        &node_extent,
        pos,
        &VecLength::from_vector_and_scale(MVec::ONE, *side_length),
    ) {
        match &tree.node {
            quadtree::Node::Tree(tree) => {
                for child in tree.iter() {
                    add_particles_in_box(particles, child, pos, side_length);
                }
            }
            quadtree::Node::Leaf(leaf) => {
                particles.extend(leaf.iter());
            }
        }
    }
}

fn get_particles_in_box<'a>(
    tree: &'a QuadTree,
    pos: &VecLength,
    side_length: &Length,
) -> Vec<&'a LeafData> {
    let mut particles = vec![];
    add_particles_in_box(&mut particles, tree, pos, side_length);
    particles
}

pub(super) fn get_particles_in_radius<'a>(
    tree: &'a QuadTree,
    pos: &VecLength,
    radius: &Length,
) -> Vec<&'a LeafData> {
    get_particles_in_box(tree, pos, radius)
        .into_iter()
        .filter(|particle| particle.pos().distance(pos) < radius.max(particle.smoothing_length))
        .collect()
}

pub(super) fn construct_quad_tree_system(
    parameters: Res<HydrodynamicsParameters>,
    particles: Particles<(Entity, &Position, &SmoothingLength)>,
    extent: Res<GlobalExtent>,
    mut quadtree: ResMut<QuadTree>,
) {
    let particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos, smoothing_length)| LeafData {
            entity,
            pos: pos.0,
            smoothing_length: **smoothing_length,
        })
        .collect();
    *quadtree = QuadTree::new(&parameters.tree, particles, &extent);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::get_particles_in_radius;
    use super::LeafData;
    use super::QuadTree;
    use crate::domain::extent::Extent;
    use crate::gravity::tests::get_particles;
    use crate::quadtree::QuadTreeConfig;
    use crate::units::Length;
    use crate::units::VecLength;

    pub(super) fn direct_neighbour_search<'a>(
        particles: &'a [LeafData],
        pos: &VecLength,
        radius: &Length,
    ) -> Vec<&'a LeafData> {
        particles
            .iter()
            .filter(|particle| particle.pos.distance(pos) < radius.max(particle.smoothing_length))
            .collect()
    }

    #[test]
    fn radius_search() {
        let n = 12;
        let m = 12;
        let radius = Length::meters(2.0);
        let particles = get_particles(n, m);
        let particles: Vec<_> = particles
            .into_iter()
            .map(|particle| LeafData {
                entity: particle.entity,
                pos: particle.pos,
                smoothing_length: particle.pos.x() * 0.2,
            })
            .collect();
        let extent = Extent::from_positions(particles.iter().map(|leaf| &leaf.pos)).unwrap();
        let tree = QuadTree::new(&QuadTreeConfig::default(), particles.clone(), &extent);
        for particle in particles.iter() {
            let tree_neighbours = get_particles_in_radius(&tree, &particle.pos, &radius);
            let direct_neighbours = direct_neighbour_search(&particles, &particle.pos, &radius);
            let tree_entities: HashSet<_> = tree_neighbours
                .into_iter()
                .map(|particle| particle.entity)
                .collect();
            let direct_entities: HashSet<_> = direct_neighbours
                .into_iter()
                .map(|particle| particle.entity)
                .collect();
            assert_eq!(tree_entities, direct_entities);
        }
    }
}