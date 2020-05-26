
use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::{equal_graphs, PhaseNode};
use std::fmt::Write;

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = ErEnsembleC::<EmptyNode, Pcg64>::new(500, 2.7, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=200 {
        let steps = e.m_steps(i);
        e.undo_steps_quiet(steps);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }

}

#[test]
fn monte_continue(){
    // first init an ensemble, which implements MarkovChain
    let rng = Pcg64::seed_from_u64(7567526);
    let mut ensemble = ErEnsembleC::<EmptyNode, _>::new(300, 4.0, rng);
    while !ensemble.is_connected().unwrap() {
        ensemble.randomize();
    }
    let mut ensemble_clone = ensemble.clone();
    let mut large_deviation_metropolis = String::new();

    // now perform large deviation simulation
    let metropolis_rng = Pcg64::seed_from_u64(77526);
    let state = ensemble.metropolis_while(
        metropolis_rng, // rng
        -10.0,          // temperature
        30,             // stepsize
        100,            // steps
        |ensemble| ensemble.is_connected().unwrap(),    // valid_self
        |ensemble| ensemble.diameter().unwrap() as f64, // energy
        |ensemble, counter, energy, rejected| {         // measure
            // of cause, you can store it in a file instead
            writeln!(large_deviation_metropolis, "{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count())
            .unwrap();
        },
        |_, counter| counter == 20,                     // break_if
    );

    // resume the simulation
    ensemble.continue_metropolis_while(
        state,
        false,
        |ensemble| ensemble.is_connected().unwrap(),    // valid_self
        |ensemble| ensemble.diameter().unwrap() as f64, // energy
        |ensemble, counter, energy, rejected| {         // measure
            // of cause, you can store it in a file instead
            writeln!(large_deviation_metropolis, "{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count())
            .unwrap();
        },
        |_, counter| counter == 20,                     // break_if
    );

    // alternative simulation
    let mut large_deviation_metropolis_clone = String::new();

    let metropolis_rng = Pcg64::seed_from_u64(77526);

    ensemble_clone.metropolis_while(
        metropolis_rng, // rng
        -10.0,          // temperature
        30,             // stepsize
        100,            // steps
        |ensemble| ensemble.is_connected().unwrap(),    // valid_self
        |ensemble| ensemble.diameter().unwrap() as f64, // energy
        |ensemble, counter, energy, rejected| {         // measure
            // of cause, you can store it in a file instead
            writeln!(large_deviation_metropolis_clone, "{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count())
            .unwrap();
        },
        |_, _| false,                     // break_if
    );

    assert_eq!(
        large_deviation_metropolis,
        large_deviation_metropolis_clone
    );
}

#[test]
fn test_graph_construction() {
    let rng = Pcg64::seed_from_u64(76);
    let e = ErEnsembleC::<EmptyNode, Pcg64>::new(20, 2.7, rng);
    assert_eq!(e.graph().edge_count(), 28);
    assert_eq!(20, e.graph().vertex_count());
}

#[test]
fn test_complete_graph() {
    let rng = Pcg64::seed_from_u64(76);
    let e = ErEnsembleC::<EmptyNode, Pcg64>::new(20, 19.0, rng);
    assert_eq!(20, e.graph().vertex_count());
    assert_eq!(190, e.graph().edge_count());
    assert!(e.graph().is_connected().expect("test_complete_graph error"));
}

#[test]
fn iter_optimization_nth() {
    let size = 50;
    let rng = Pcg64::seed_from_u64(489);
    let e = ErEnsembleC::<PhaseNode, Pcg64>::new(size, 6.0, rng);

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
}
