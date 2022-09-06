use std::marker::PhantomData;

use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::NonSendMut;
use bevy::prelude::ParallelSystemDescriptorCoercion;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use mpi::traits::Equivalence;
use mpi::traits::MatchesRaw;

use super::DomainDecompositionStages;
use crate::communication::CommunicationPlugin;
use crate::communication::CommunicationType;
use crate::communication::DataByRank;
use crate::communication::ExchangeCommunicator;
use crate::communication::Rank;
use crate::communication::SizedCommunicator;
use crate::communication::WorldRank;
use crate::communication::WorldSize;
use crate::physics::LocalParticle;

struct ExchangePluginExists;

#[derive(Default)]
pub(super) struct OutgoingEntities(DataByRank<Vec<Entity>>);

impl OutgoingEntities {
    pub fn add(&mut self, rank: Rank, entity: Entity) {
        self.0[rank].push(entity);
    }
}

#[derive(Default)]
struct SpawnedEntities(DataByRank<Vec<Entity>>);

struct ExchangeBuffers<T>(DataByRank<Vec<T>>);

pub struct ExchangeDataPlugin<T> {
    _marker: PhantomData<T>,
}

#[derive(Equivalence)]
struct NumEntities(usize);

impl<T> Default for ExchangeDataPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData::default(),
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> Plugin for ExchangeDataPlugin<T>
where
    <T as Equivalence>::Out: MatchesRaw,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        let exists = app.world.get_resource_mut::<ExchangePluginExists>();
        let first = exists.is_none();
        let rank = app.world.get_resource::<WorldRank>().unwrap().0;
        let size = app.world.get_resource::<WorldSize>().unwrap().0;
        if first {
            app.insert_resource(ExchangePluginExists)
                .insert_resource(OutgoingEntities(DataByRank::from_size_and_rank(size, rank)))
                .insert_resource(SpawnedEntities(DataByRank::from_size_and_rank(size, rank)))
                .add_plugin(CommunicationPlugin::<NumEntities>::new(
                    CommunicationType::Exchange,
                ));
        }
        app.insert_resource(ExchangeBuffers::<T>(DataByRank::from_size_and_rank(
            size, rank,
        )))
        .add_plugin(CommunicationPlugin::<T>::new(CommunicationType::Exchange))
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::fill_buffers_system,
        )
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::send_buffers_system
                .after(Self::fill_buffers_system)
                .before(reset_outgoing_entities_system)
                .before(schedule_system),
        )
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::receive_buffers_system
                .after(Self::send_buffers_system)
                .after(spawn_incoming_entities_system)
                .after(schedule_system),
        )
        .add_system_to_stage(
            DomainDecompositionStages::Exchange,
            Self::reset_buffers_system.after(Self::receive_buffers_system),
        );
        if first {
            app.add_system_to_stage(
                DomainDecompositionStages::Exchange,
                send_num_outgoing_entities_system,
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                despawn_outgoing_entities_system,
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                reset_outgoing_entities_system.after(send_num_outgoing_entities_system),
            )
            .add_system_to_stage(
                DomainDecompositionStages::Exchange,
                spawn_incoming_entities_system.after(send_num_outgoing_entities_system),
            )
            .add_system_to_stage(DomainDecompositionStages::Exchange, schedule_system);
        }
    }
}

impl<T: Sync + Send + 'static + Component + Clone + Equivalence> ExchangeDataPlugin<T> {
    fn fill_buffers_system(
        entity_exchange: Res<OutgoingEntities>,
        query: Query<&T>,
        mut buffer: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, entities) in entity_exchange.0.iter() {
            // This allocates a new buffer every time. An alternative would be
            // to keep this at maximum size, trading performance for memory overhead
            buffer.0.insert(
                *rank,
                entities
                    .iter()
                    .map(|entity| query.get(*entity).unwrap().clone())
                    .collect(),
            );
        }
    }

    fn send_buffers_system(
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        mut buffers: ResMut<ExchangeBuffers<T>>,
    ) {
        for (rank, data) in buffers.0.drain_all() {
            communicator.send_vec(rank, data);
        }
    }

    fn receive_buffers_system(
        mut commands: Commands,
        mut communicator: NonSendMut<ExchangeCommunicator<T>>,
        spawned_entities: Res<SpawnedEntities>,
    ) {
        for (rank, data) in communicator.receive_vec() {
            let spawned_entities = spawned_entities.0[rank].clone();
            for (entity, component) in spawned_entities.iter().zip(data.into_iter()) {
                commands.entity(*entity).insert(component);
            }
        }
    }

    fn reset_buffers_system(
        mut buffers: ResMut<ExchangeBuffers<T>>,
        size: Res<WorldSize>,
        rank: Res<WorldRank>,
    ) {
        *buffers = ExchangeBuffers(DataByRank::from_size_and_rank(size.0, rank.0));
    }
}

fn send_num_outgoing_entities_system(
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    num_outgoing: Res<OutgoingEntities>,
) {
    for rank in communicator.other_ranks() {
        communicator.send(rank, NumEntities(num_outgoing.0.get(&rank).unwrap().len()));
    }
}

fn spawn_incoming_entities_system(
    mut commands: Commands,
    mut communicator: NonSendMut<ExchangeCommunicator<NumEntities>>,
    mut spawned_entities: ResMut<SpawnedEntities>,
) {
    for (rank, num_incoming) in communicator.receive() {
        spawned_entities.0.insert(
            rank,
            (0..num_incoming.0)
                .map(|_| {
                    let id = commands.spawn().insert(LocalParticle).id();
                    id
                })
                .collect(),
        );
    }
}

fn reset_outgoing_entities_system(
    mut outgoing: ResMut<OutgoingEntities>,
    size: Res<WorldSize>,
    rank: Res<WorldRank>,
) {
    *outgoing = OutgoingEntities(DataByRank::from_size_and_rank(size.0, rank.0));
}

fn despawn_outgoing_entities_system(
    mut commands: Commands,
    entity_exchange: Res<OutgoingEntities>,
) {
    for (_, entities) in entity_exchange.0.iter() {
        for entity in entities {
            commands.entity(*entity).despawn();
        }
    }
}

fn schedule_system() {}

#[cfg(test)]
#[cfg(feature = "local")]
mod tests {
    use bevy::prelude::App;
    use bevy::prelude::Component;
    use bevy::prelude::CoreStage;
    use bevy::prelude::SystemStage;
    use mpi::traits::Equivalence;

    use crate::communication::build_local_communication_app_with_custom_logic;
    use crate::communication::BaseCommunicationPlugin;
    use crate::communication::WorldRank;
    use crate::domain::exchange_data_plugin::ExchangeDataPlugin;
    use crate::domain::exchange_data_plugin::OutgoingEntities;
    use crate::domain::DomainDecompositionStages;

    #[derive(Clone, Equivalence, Component)]
    struct A {
        x: i32,
        y: f32,
    }
    #[derive(Clone, Equivalence, Component)]
    struct B {
        x: i64,
        y: bool,
    }

    fn check_received(mut app: App) {
        let is_main = app.world.get_resource::<WorldRank>().unwrap().is_main();
        let mut entities = vec![];
        if is_main {
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 0, y: 5.0 })
                    .insert(B { x: 0, y: false })
                    .id(),
            );
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 1, y: 10.0 })
                    .insert(B { x: 1, y: true })
                    .id(),
            );
            entities.push(
                app.world
                    .spawn()
                    .insert(A { x: 2, y: 20.0 })
                    .insert(B { x: 2, y: false })
                    .id(),
            );
        }
        let check_num_entities = |app: &mut App, rank_0_count: usize, rank_1_count: usize| {
            let mut query = app.world.query::<&mut A>();
            let count = query.iter(&app.world).count();
            if is_main {
                assert_eq!(count, rank_0_count);
            } else {
                assert_eq!(count, rank_1_count);
            }
        };
        let mut exchange_first_entity = |app: &mut App| {
            if is_main {
                let mut outgoing = app.world.get_resource_mut::<OutgoingEntities>().unwrap();
                outgoing.add(1, entities.remove(0));
            }
        };
        check_num_entities(&mut app, 3, 0);
        exchange_first_entity(&mut app);
        app.update();
        check_num_entities(&mut app, 2, 1);
        app.update();
        check_num_entities(&mut app, 2, 1);
        exchange_first_entity(&mut app);
        exchange_first_entity(&mut app);
        app.update();
        check_num_entities(&mut app, 0, 3);
    }

    #[test]
    fn exchange_data_plugin() {
        build_local_communication_app_with_custom_logic(
            |app, size, rank| build_app(app, size, rank),
            check_received,
            2,
        );
    }

    fn build_app(app: &mut App, size: usize, rank: i32) {
        app.add_stage_after(
            CoreStage::Update,
            DomainDecompositionStages::Exchange,
            SystemStage::parallel(),
        )
        .add_plugin(BaseCommunicationPlugin::new(size, rank))
        .add_plugin(ExchangeDataPlugin::<A>::default())
        .add_plugin(ExchangeDataPlugin::<B>::default());
    }
}