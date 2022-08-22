use mpi::traits::Equivalence;

use super::peano_hilbert::PeanoHilbertKey;
use super::ParticleData;

/// A segment of peano hilbert keys corresponding to
/// the interval including `start` but excluding `end`
#[derive(Debug, Equivalence, Clone)]
pub struct Segment {
    start: PeanoHilbertKey,
    end: PeanoHilbertKey,
    num_particles: usize,
}

fn num_contained_particles(
    particles: &[ParticleData],
    start: PeanoHilbertKey,
    end: PeanoHilbertKey,
) -> usize {
    let start_index = particles.binary_search_by_key(&start, |p: &ParticleData| p.key);
    let end_index = particles.binary_search_by_key(&end, |p: &ParticleData| p.key);
    let start_index = start_index.unwrap_or_else(|not_found| not_found);
    let end_index = end_index.unwrap_or_else(|not_found| not_found);
    end_index - start_index
}

impl Segment {
    fn new(particles: &[ParticleData], start: PeanoHilbertKey, end: PeanoHilbertKey) -> Self {
        debug_assert!(start <= end);
        Self {
            start,
            end,
            num_particles: num_contained_particles(particles, start, end),
        }
    }

    fn split_into(
        self,
        segments: &mut Vec<Segment>,
        particles: &[ParticleData],
        desired_segment_size: usize,
    ) {
        if self.num_particles > desired_segment_size {
            let half = PeanoHilbertKey((self.end.0 + self.start.0) / 2);
            let left = Segment::new(particles, self.start, half);
            let right = Segment::new(particles, half, self.end);
            left.split_into(segments, particles, desired_segment_size);
            right.split_into(segments, particles, desired_segment_size);
        } else if self.num_particles > 0 {
            segments.push(self);
        }
    }
}

pub(super) fn get_segments(
    particles: &[ParticleData],
    desired_segment_size: usize,
) -> Vec<Segment> {
    if particles.len() == 0 {
        return vec![];
    }
    if particles.len() == 1 {
        return vec![Segment {
            start: particles[0].key,
            end: particles[0].key,
            num_particles: 1,
        }];
    }
    let segment = Segment {
        start: particles[0].key,
        end: particles.last().unwrap().key.next(),
        num_particles: particles.len(),
    };
    let mut segments = vec![];
    segment.split_into(&mut segments, &particles, desired_segment_size);
    segments
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Entity;

    use crate::domain::peano_hilbert::PeanoHilbertKey;
    use crate::domain::segment::Segment;
    use crate::domain::ParticleData;

    fn get_particles() -> Vec<ParticleData> {
        (0..10)
            .chain(30..40)
            .map(|i| ParticleData {
                key: PeanoHilbertKey(i),
                entity: Entity::from_raw(i as u32),
            })
            .collect()
    }

    #[test]
    fn num_contained_particles() {
        let particles = get_particles();
        let check_num = |start: usize, end: usize, num: usize| {
            assert_eq!(
                Segment::new(
                    &particles,
                    PeanoHilbertKey(start as u64),
                    PeanoHilbertKey(end as u64)
                )
                .num_particles,
                num
            );
        };
        check_num(0, 0, 0); // empty
        check_num(0, 1, 1); // contains: 0
        check_num(38, 40, 2); // contains: 38, 39

        check_num(20, 20, 0); // empty
        check_num(20, 25, 0); // empty

        check_num(9, 10, 1); // contains: 9
        check_num(9, 11, 1); // contains: 9

        check_num(25, 32, 2); // contains: 30, 31
    }

    #[test]
    fn get_segments_reaches_desired_size() {
        let particles = get_particles();
        let desired_size = 4;
        let segments = super::get_segments(&particles, desired_size);
        for segment in segments.iter() {
            assert!(segment.num_particles <= desired_size);
        }
    }

    #[test]
    fn get_segments_has_correct_total_number_of_particles() {
        let particles = get_particles();
        let desired_size = 4;
        let segments = super::get_segments(&particles, desired_size);
        assert_eq!(
            segments
                .iter()
                .map(|segment| segment.num_particles)
                .sum::<usize>(),
            particles.len()
        );
    }

    #[test]
    fn get_segments_does_not_fail_with_empty_list() {
        let particles = vec![];
        super::get_segments(&particles, 3);
    }

    #[test]
    fn get_segments_does_not_fail_with_single_particle() {
        let particles = vec![get_particles().remove(0)];
        super::get_segments(&particles, 3);
    }
}
