
use rand_pcg::Pcg64;
use net_ensembles::ER;
use net_ensembles::TestNode;
use rand::SeedableRng;
mod common;
use common::equal_graphs;

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = ER::<TestNode, Pcg64>::new(500, 2.7, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=200 {
        let steps = e.random_steps(i);
        e.undo_steps(steps).unwrap();

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }

}


#[test]
fn test_graph_construction() {
    let rng = Pcg64::seed_from_u64(76);
    let e = ER::<TestNode, Pcg64>::new(20, 2.7, rng);
    assert_eq!(e.graph().edge_count(), 28);
    assert_eq!(20, e.graph().vertex_count());
}

#[test]
fn test_complete_graph() {
    let rng = Pcg64::seed_from_u64(76);
    let e = ER::<TestNode, Pcg64>::new(20, 19.0, rng);
    assert_eq!(20, e.graph().vertex_count());
    assert_eq!(190, e.graph().edge_count());
    assert!(e.graph().is_connected().expect("test_complete_graph error"));
}
