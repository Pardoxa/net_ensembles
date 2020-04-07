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
        e.mc_step();
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
        let _steps = e.mc_steps(i);
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
        let _steps = e.mc_steps(i);
        //println!("do: {:?}", _steps);
        let _undo_step = e.undo_steps_quiet(_steps);
        //println!("undo: {:?}", _undo_step);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }
}

//#[test]
//fn simple_sample() {
//    use net_ensembles::{SwEnsemble, EmptyNode};
//    use net_ensembles::traits::*; // I recommend always using this
//    use rand_pcg::Pcg64; //or whatever you want to use as rng
//    use rand::SeedableRng; // I use this to seed my rng, but you can use whatever
//    use std::fs::File;
//    use std::io::{BufWriter, Write};
//
//    let rng = Pcg64::seed_from_u64(122);
//
//    // now create small-world ensemble with 200 nodes and a rewiring probability of 0.3 for each edge
//    let mut sw_ensemble = SwEnsemble::<EmptyNode, Pcg64>::new(100, 0.3, rng);
//
//    // setup file for writing
//    let f = File::create("simple_sample_sw.dat")
//        .expect("Unable to create file");
//    let mut f = BufWriter::new(f);
//    f.write_all(b"#diameter bi_connect_max\n")
//        .unwrap();
//
//    // simple sample for 10 steps
//    sw_ensemble.simple_sample(10,
//        |ensemble|
//        {
//            let diameter = ensemble.graph()
//                .diameter()
//                .unwrap();
//
//            let bi_connect_max = ensemble.graph()
//                .clone()
//                .vertex_biconnected_components(false)[0];
//
//            write!(f, "{} {}\n", diameter, bi_connect_max)
//                .unwrap();
//        }
//    )
//}
