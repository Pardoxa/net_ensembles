
use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::{equal_graphs, PhaseNode};
use net_ensembles::sampling::Metropolis;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;

#[cfg(feature = "serde_support")]
use serde_json;

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = ErEnsembleC::<EmptyNode, Pcg64>::new(500, 2.7, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    let mut steps = Vec::new();
    for i in 0..=200 {
        e.m_steps(i, &mut steps);
        e.undo_steps_quiet(&steps);

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
    let ensemble_clone = ensemble.clone();
    let mut large_deviation_metropolis = String::new();

    // now perform large deviation simulation
    let metropolis_rng = Pcg64::seed_from_u64(77526);

    let current_energy = ensemble.diameter().unwrap() as f64;
    let mut metropolis = Metropolis::new_from_temperature(
        metropolis_rng,
        ensemble,
        current_energy,
        -10.0,
        30
    ).ok().unwrap();

    metropolis.metropolis_while(
        |ensemble| ensemble.diameter().map(|val| val as f64), // energy
        |metr|   writeln!(large_deviation_metropolis, "{}, {}, {}", metr.counter(), metr.energy(), metr.ensemble().leaf_count()).unwrap(), // measure
        |metr| metr.counter() < 20
    );

    metropolis.metropolis_while(
        |ensemble| ensemble.diameter().map(|val| val as f64), // energy
        |metr|   writeln!(large_deviation_metropolis, "{}, {}, {}", metr.counter(), metr.energy(), metr.ensemble().leaf_count()).unwrap(), // measure
        |metr| metr.counter() < 40
    );




    // alternative simulation
    let mut large_deviation_metropolis_clone = String::new();

    let metropolis_rng2 = Pcg64::seed_from_u64(77526);

    let mut metropolis2 = Metropolis::new_from_temperature(
        metropolis_rng2,
        ensemble_clone,
        current_energy,
        -10.0,
        30
    ).ok().unwrap();

    metropolis2.metropolis_while(
        |ensemble| ensemble.diameter().map(|val| val as f64), // energy
        |metr|   writeln!(large_deviation_metropolis_clone, "{}, {}, {}", metr.counter(), metr.energy(), metr.ensemble().leaf_count()).unwrap(), // measure
        |metr| metr.counter() < 40
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

#[cfg(feature = "serde_support")]
#[test]
fn unchanging_graph_construction()
{
    
    let rng = Pcg64::seed_from_u64(123929);
    let er: ErEnsembleC<EmptyNode, _> = ErEnsembleC::new(123, 2.0, rng);

    let read = File::open("TestData/unchaning_erc_1.json")
        .expect("Unable to open file");
    let bufr1 = BufReader::new(read);

    let unchaning_1_load: ErEnsembleC<EmptyNode, Pcg64> = serde_json::from_reader(bufr1).unwrap();

    equal_graphs(er.graph(), unchaning_1_load.graph());

    let rng = Pcg64::seed_from_u64(1929);
    let er2: ErEnsembleC<EmptyNode, _> = ErEnsembleC::new(233, 10.0, rng);
    

    let read2 = File::open("TestData/unchaning_erc_2.json")
        .expect("Unable to open file");
    let bufr2 = BufReader::new(read2);

    let unchaning_2_load: ErEnsembleC<EmptyNode, Pcg64> = serde_json::from_reader(bufr2).unwrap();

    equal_graphs(er2.graph(), unchaning_2_load.graph());

}
