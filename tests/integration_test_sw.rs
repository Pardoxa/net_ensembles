use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;
use common::PhaseNode;

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
        e.m_step();
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
        let _steps = e.m_steps(i);
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
        let _steps = e.m_steps(i);
        //println!("do: {:?}", _steps);
        let _undo_step = e.undo_steps_quiet(_steps);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }
}

#[test]
fn iter_optimization_nth() {
    let size = 50;
    let rng = Pcg64::seed_from_u64(4893);
    let mut e = SwEnsemble::<PhaseNode, Pcg64>::new(size, 0.3, rng);

    let mut iter = e.graph().contained_iter_neighbors(0);
    let len = iter.len();
    for i in 0..len+1 {
        let mut iter2 = e.graph().contained_iter_neighbors(0);
        let nex = iter.next();
        assert_eq!(
            nex,
            iter2.nth(i)
        );
        println!("{:?}", nex);
    }


    let mut iter = e.graph().contained_iter();
    let len = iter.len();
    for i in 0..len + 1 {
        let mut iter2 = e.graph().contained_iter();
        let nex = iter.next();
        assert_eq!(
            nex,
            iter2.nth(i)
        );
        println!("{:?}", nex);
    }


    let iter = e.contained_iter_neighbors_mut(0);
    let phase = iter.fold(10.0, |acc, data|
            {
                let p = (*data).get_phase();
                (*data).set_phase(12.0);
                acc + p
            }
        );
    e.at_mut(0).set_phase(  phase  );

    let _ = e.contained_iter();
}
