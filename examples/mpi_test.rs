// This is effectively an MPI test written as an example. This is
// unfortunate but "necessary" since support for custom test runners
// is very light at the moment.  I tried obtaining the executable
// built by cargo test and then running it with mpirun (filtering only
// for the tests that should be run with mpirun), but this doesn't
// work because of how cargo test produces a multithreaded binary -
// thread/rank 0 gets stuck at distributing work and doesn't enter the
// program.  Passing --num-threads 1 and --jobs 1 does not help

use mpi::traits::Communicator;
use mpi::Tag;
use raxiom::communication::exchange_communicator::ExchangeCommunicator;
use raxiom::communication::DataByRank;
use raxiom::communication::MpiWorld;
use raxiom::communication::SizedCommunicator;
use raxiom::communication::MPI_UNIVERSE;

pub fn main() {
    let fns: &[(&str, fn())] = &[
        ("exchange_all", exchange_all),
        ("send_receive", send_receive),
    ];
    for (name, f) in fns {
        f();
        if MPI_UNIVERSE.world().rank() == 0 {
            println!("{} ... ok", name);
        }
    }
    MPI_UNIVERSE.drop();
}

fn send_receive() {
    let mut world = MpiWorld::<i32>::new(Tag::default());
    let rank = world.rank();
    let x0: Vec<i32> = vec![1, 2, 3];
    let x1: Vec<i32> = vec![3, 2, 1];
    if rank == 0 {
        world.blocking_send_vec(1, &x0);
        assert_eq!(MpiWorld::<i32>::receive_vec(&mut world, 1), x1);
    } else if rank == 1 {
        assert_eq!(MpiWorld::<i32>::receive_vec(&mut world, 0), x0);
        world.blocking_send_vec(0, &x1);
    }
}

fn exchange_all() {
    let world = MpiWorld::<i32>::new(Tag::default());
    let rank = world.rank();
    let mut exchange_comm = ExchangeCommunicator::from(world);
    for _ in 0..100 {
        let x0: Vec<i32> = (0..100).collect();
        let x1: Vec<i32> = (0..100).rev().collect();
        if rank == 0 {
            let data = DataByRank::from_iter([(1, x0)].into_iter());
            let res = exchange_comm.exchange_all(data);
            assert_eq!(res[1], x1);
        } else if rank == 1 {
            let data = DataByRank::from_iter([(0, x1)].into_iter());
            let res = exchange_comm.exchange_all(data);
            assert_eq!(res[0], x0);
        }
    }
}
