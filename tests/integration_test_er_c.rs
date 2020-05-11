
use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::{equal_graphs, PhaseNode};
use serde_json;
use std::fs::File;
use net_ensembles::monte_carlo::MetropolisState;
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
fn monte_save(){
    // first init an ensemble, which implements MarkovChain
    let rng = Pcg64::seed_from_u64(7567526);
    let mut ensemble = ErEnsembleC::<EmptyNode, _>::new(300, 4.0, rng);
    // ensure that inital state is valid
    while !ensemble.is_connected().unwrap() {
        ensemble.randomize();
    }

    // now perform large deviation simulation
    // in this example the simulation will be interrupted, when the counter hits 20:
    // break_if = |_, counter| counter == 20
    let metropolis_rng = Pcg64::seed_from_u64(77526);
    let state = ensemble.monte_carlo_metropolis_while(
        metropolis_rng, // rng
        -10.0,          // temperature
        30,             // stepsize
        100,            // steps
        |ensemble| ensemble.is_connected().unwrap(),    // valid_self
        |ensemble| ensemble.diameter().unwrap() as f64, // energy
        |ensemble, counter, energy, rejected| {         // measure
            // of cause, you can store it in a file instead
            println!("{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count());
        },
        |_, counter| counter == 20,                     // break_if
    );

    // NOTE: You will likely not need the cfg part
    // I only need it, because the example has to work with and without serde_support
    #[cfg(feature = "serde_support")]
    {
        // saving
        let save_ensemble = File::create("metropolis_ensemble.save")
        .expect("Unable to create file");
        serde_json::to_writer_pretty(save_ensemble, &ensemble).unwrap();

        let save_state = File::create("metropolis_state.save")
        .expect("Unable to create file");
        serde_json::to_writer_pretty(save_state, &state).unwrap();



        // loading
        let ensemble_reader = File::open("metropolis_ensemble.save")
        .expect("Unable to open file");

        let mut loaded_ensemble: ErEnsembleC::<EmptyNode, Pcg64>
        = serde_json::from_reader(ensemble_reader).unwrap();

        let state_reader = File::open("metropolis_state.save")
        .expect("Unable to open file");

        let loaded_state: MetropolisState::<Pcg64>
        = serde_json::from_reader(state_reader).unwrap();



        // resume the simulation
        loaded_ensemble.monte_carlo_metropolis_while_resume(
            loaded_state,
            false,
            |ensemble| ensemble.is_connected().unwrap(),    // valid_self
            |ensemble| ensemble.diameter().unwrap() as f64, // energy
            |ensemble, counter, energy, rejected| {         // measure
                // of cause, you can store it in a file instead
                println!("{}, {}, {}, {}", counter, rejected, energy, ensemble.leaf_count());
            },
            |_, _| false,                     // break_if
        );
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
    let state = ensemble.monte_carlo_metropolis_while(
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
    ensemble.monte_carlo_metropolis_while_resume(
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

    ensemble_clone.monte_carlo_metropolis_while(
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
