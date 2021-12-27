use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;
use common::PhaseNode;

use std::fs::File;
use std::io::BufReader;


#[cfg(feature = "serde_support")]
use serde_json;

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
            assert_eq!(2, e.graph().container(i).count_root());
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
            assert_eq!(2, e.graph().container(i).count_root());
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

    let mut steps = Vec::with_capacity(501);
    let mut undo_step = Vec::new();
    for i in 0..=500 {
        e.m_steps(i, &mut steps);
        //println!("do: {:?}", _steps);
        let _undo_step = e.undo_steps(&steps, &mut undo_step);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(e_0.graph(), e.graph());
    }
}

#[test]
fn step_quiet_test() {
    let size = 500;
    let rng = Pcg64::seed_from_u64(960448);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.3, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    let mut steps = Vec::with_capacity(501);
    for i in 0..=500 {
        e.m_steps(i, &mut steps);
        //println!("do: {:?}", _steps);
        e.undo_steps_quiet(&steps);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(e_0.graph(), e.graph());
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



#[cfg(feature = "serde_support")]
#[test]
fn unchanging_graph_construction()
{
    let rng = Pcg64::seed_from_u64(123929);
    let sw1: SwEnsemble<EmptyNode, _> = SwEnsemble::new(123, 0.1, rng);

    let read = File::open("TestData/unchaning_sw_1.json")
        .expect("Unable to open file");
    let bufr1 = BufReader::new(read);

    let unchaning_1_load: SwEnsemble<EmptyNode, Pcg64> = serde_json::from_reader(bufr1).unwrap();

    equal_graphs(sw1.graph(), unchaning_1_load.graph());


    let rng = Pcg64::seed_from_u64(1929);
    let sw2: SwEnsemble<EmptyNode, _> = SwEnsemble::new(133, 0.3, rng);
    

    let read2 = File::open("TestData/unchaning_sw_2.json")
        .expect("Unable to open file");
    let bufr2 = BufReader::new(read2);

    let unchaning_2_load: SwEnsemble<EmptyNode, Pcg64> = serde_json::from_reader(bufr2).unwrap();

    equal_graphs(sw2.graph(), unchaning_2_load.graph());


    let rng = Pcg64::seed_from_u64(1229);
    let sw3: SwEnsemble<EmptyNode, _> = SwEnsemble::new(123, 0.0, rng);


    let read3 = File::open("TestData/unchaning_sw_3.json")
        .expect("Unable to open file");
    let bufr3 = BufReader::new(read3);

    let unchaning_1_load: SwEnsemble<EmptyNode, Pcg64> = serde_json::from_reader(bufr3).unwrap();

    equal_graphs(sw3.graph(), unchaning_1_load.graph());


    let rng = Pcg64::seed_from_u64(1922239);
    let sw4: SwEnsemble<EmptyNode, _> = SwEnsemble::new(133, 1.0, rng);

    let read4 = File::open("TestData/unchaning_sw_4.json")
        .expect("Unable to open file");
    let bufr4 = BufReader::new(read4);

    let unchaning_2_load: SwEnsemble<EmptyNode, Pcg64> = serde_json::from_reader(bufr4).unwrap();

    equal_graphs(sw4.graph(), unchaning_2_load.graph());

}