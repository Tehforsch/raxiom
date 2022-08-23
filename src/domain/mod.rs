use bevy::prelude::*;
use mpi::traits::Equivalence;

mod extent;
mod peano_hilbert;
pub mod quadtree;
pub mod segment;

use self::extent::Extent;
use self::peano_hilbert::PeanoHilbertKey;
use self::segment::get_segments;
use self::segment::Segment;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::domain::segment::sort_and_merge_segments;
use crate::mass::Mass;
use crate::particle::LocalParticleBundle;
use crate::physics::LocalParticle;
use crate::position::Position;
use crate::units::Length;
use crate::units::VecLength;
use crate::velocity::Velocity;

#[derive(StageLabel)]
pub enum DomainDecompositionStages {
    Decomposition,
}

pub struct DomainDecompositionPlugin;

impl Plugin for DomainDecompositionPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Decomposition,
            SystemStage::parallel(),
        );
        let extent = Extent::new(
            Length::meter(-100.0),
            Length::meter(100.0),
            Length::meter(-100.0),
            Length::meter(100.0),
        );
        app.insert_resource(GlobalExtent(extent));
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            determine_global_extent_system,
        );
        app.add_system_to_stage(
            DomainDecompositionStages::Decomposition,
            domain_decomposition_system.after(determine_global_extent_system),
        );
    }
}

struct GlobalExtent(Extent);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ParticleData {
    key: PeanoHilbertKey,
    entity: Entity,
}

impl ParticleData {
    fn key(&self) -> PeanoHilbertKey {
        self.key
    }
}

fn determine_global_extent_system(// mut commands: Commands,
    // particles: Query<&Position, With<LocalParticle>>,
) {
    debug!("TODO: Determine global extent");
}

fn domain_decomposition_system(
    mut commands: Commands,
    rank: Res<Rank>,
    extent: Res<GlobalExtent>,
    particles: Query<(Entity, &Position), With<LocalParticle>>,
    full_particles: Query<(Entity, &Position, &Velocity, &Mass), With<LocalParticle>>,
    all_particles: Query<(Entity, &Position)>,
    mut comm: NonSendMut<ExchangeCommunicator<Segment>>,
    mut exchange_comm: NonSendMut<ExchangeCommunicator<ParticleExchangeData>>,
) {
    let mut particles: Vec<_> = particles
        .iter()
        .map(|(entity, pos)| ParticleData {
            entity,
            key: PeanoHilbertKey::new(&extent.0, &pos.0),
        })
        .collect();
    particles.sort();
    const NUM_DESIRED_SEGMENTS_PER_RANK: usize = 10;
    let num_desired_particles_per_segment = particles.len() / NUM_DESIRED_SEGMENTS_PER_RANK;
    let segments = get_segments(&particles, num_desired_particles_per_segment);
    for rank in comm.other_ranks() {
        comm.send_vec(rank, segments.clone());
    }
    let mut all_segments = comm.receive_vec();
    let received_all_segments = all_segments.clone();
    all_segments.insert(*rank, segments);
    let total_load: usize = all_segments
        .iter()
        .map(|(_, segments)| segments.iter().map(|s| s.num_particles).sum::<usize>())
        .sum();
    let all_segments = sort_and_merge_segments(all_segments);
    let seggis = all_segments.clone();
    let load_per_rank = total_load / comm.size();
    let mut load = 0;
    let mut key_cutoffs_by_rank = vec![];
    for segment in all_segments.into_iter() {
        load += segment.num_particles;
        if load >= load_per_rank {
            key_cutoffs_by_rank.push(segment.end());
            if key_cutoffs_by_rank.len() == comm.size() - 1 {
                break;
            }
            load = 0;
        }
    }

    let target_rank = |pos: &VecLength| {
        let key = PeanoHilbertKey::new(&extent.0, &pos);
        key_cutoffs_by_rank
            .binary_search(&key)
            .unwrap_or_else(|e| e) as Rank
    };
    let mut counts = vec![0, 0, 0, 0];
    for (_, part) in all_particles.iter() {
        counts[target_rank(&part.0) as usize] += 1
    }
    if *rank == 0 {
        if counts[2] > 100 {
            dbg!(received_all_segments);
            println!(
                "{} {} {}",
                key_cutoffs_by_rank[0].0, key_cutoffs_by_rank[1].0, key_cutoffs_by_rank[2].0
            );
            println!("---");
            let mut total = 0;
            for seg in seggis.iter() {
                total += seg.num_particles;
                let r = key_cutoffs_by_rank
                    .binary_search(&seg.start())
                    .unwrap_or_else(|e| e) as Rank;
                println!(
                    "{} {:03} {:02} {} {}",
                    r,
                    total,
                    seg.num_particles,
                    seg.start().0,
                    seg.end().0
                );
            }
        }
    }
    for (entity, pos, vel, mass) in full_particles.iter() {
        let target_rank = target_rank(&pos.0);
        if target_rank != *rank {
            commands.entity(entity).despawn();
            exchange_comm.send(
                target_rank,
                ParticleExchangeData {
                    pos: pos.clone(),
                    vel: vel.clone(),
                    mass: mass.clone(),
                },
            );
        }
    }

    for (_, moved_to_own_domain) in exchange_comm.receive_vec().into_iter() {
        for data in moved_to_own_domain.into_iter() {
            commands
                .spawn()
                .insert_bundle(LocalParticleBundle::new(data.pos, data.vel, data.mass));
        }
    }
}

#[derive(Equivalence, Clone)]
pub struct ParticleExchangeData {
    vel: Velocity,
    pos: Position,
    mass: Mass,
}
