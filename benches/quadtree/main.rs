use bevy::prelude::Entity;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::BenchmarkId;
use criterion::Criterion;
use raxiom::hydrodynamics::quadtree::LeafData;
use raxiom::hydrodynamics::QuadTree;
use raxiom::prelude::Extent;
use raxiom::prelude::SimulationBox;
use raxiom::quadtree::QuadTreeConfig;
use raxiom::units::Length;
use raxiom::units::VecLength;

fn quadtree_radius_search(quadtree: &QuadTree, box_size: &SimulationBox) {
    quadtree.get_particles_in_radius(
        box_size,
        &VecLength::meters(0.5, 0.5, 0.5),
        &Length::meters(0.00001),
    );
}

fn get_quadtree_and_box_size(min_depth: usize) -> (QuadTree, SimulationBox) {
    let min = VecLength::meters(0.0, 0.0, 0.0);
    let max = VecLength::meters(1.0, 1.0, 1.0);
    let extent = Extent::new(min, max);
    let config = QuadTreeConfig {
        min_depth,
        ..Default::default()
    };
    let tree = QuadTree::new(&config, vec![], &extent);
    let mut particles = vec![];
    tree.depth_first_map_leaf(&mut |extent: &Extent, _| particles.push(extent.center()));
    (
        QuadTree::new(
            &config,
            particles
                .into_iter()
                .map(|pos| LeafData {
                    entity: Entity::from_raw(0),
                    pos,
                    smoothing_length: Length::meters(0.0),
                })
                .collect(),
            &extent,
        ),
        extent.into(),
    )
}

pub fn quadtree_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("quadtree");
    group.noise_threshold(0.05);
    for depth in 1..7 {
        group.bench_with_input(
            BenchmarkId::from_parameter(depth),
            &get_quadtree_and_box_size(depth),
            |b, (tree, box_size)| b.iter(|| quadtree_radius_search(tree, box_size)),
        );
    }
    group.finish();
}

criterion_group!(benches, quadtree_benchmark);
criterion_main!(benches);
