use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;

#[test]
fn sanity_check() {
    let size = 30;
    let rng = Pcg64::seed_from_u64(75675026);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.15, rng);

    for _ in 0..2000 {
        e.randomize();
        assert_eq!(e.graph().edge_count(), 2 * size);
        assert_eq!(e.graph().vertex_count(), size);
        assert_eq!(e.graph().leaf_count(), 0);
        for i in 0..size {
            assert_eq!(2, e.graph().container(i as usize).count_root());
        }
    }
}

#[test]
fn random_step() {
    let size = 30;
    let rng = Pcg64::seed_from_u64(775026);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.15, rng);

    for _ in 0..2000 {
        e.random_step();
        assert_eq!(e.graph().edge_count(), 2 * size);
        assert_eq!(e.graph().vertex_count(), size);
        assert_eq!(e.graph().leaf_count(), 0);
        for i in 0..size {
            assert_eq!(2, e.graph().container(i as usize).count_root());
        }
    }
}


#[test]
fn step_test() {
    let size = 500;
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.3, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=500 {
        let _steps = e.random_steps(i);
        //println!("do: {:?}", _steps);
        let _undo_step = e.undo_steps(_steps);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }
}

#[test]
fn step_quiet_test() {
    let size = 500;
    let rng = Pcg64::seed_from_u64(960448);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.3, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=500 {
        let _steps = e.random_steps(i);
        //println!("do: {:?}", _steps);
        let _undo_step = e.undo_steps_quiet(_steps);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }
}
