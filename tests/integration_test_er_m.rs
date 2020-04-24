use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;

#[cfg(feature = "serde_support")]
use serde_json;

#[test]
fn randomize() {
    let rng = Pcg64::seed_from_u64(8763545);
    let e = ErEnsembleM::<EmptyNode, Pcg64>::new(20, 70, rng);
    assert_eq!(e.graph().edge_count(), 70);
    assert_eq!(20, e.graph().vertex_count());
}

#[cfg(feature = "serde_support")]
#[test]
fn serde_json_test() {
    let rng = Pcg64::seed_from_u64(8745);
    let mut e = ErEnsembleM::<EmptyNode, Pcg64>::new(30, 70, rng);
    let serialized = serde_json::to_string(&e).unwrap();
    println!("{}", serialized);

    let mut e2: ErEnsembleM::<EmptyNode, Pcg64> = serde_json::from_str(&serialized).unwrap();

    equal_graphs(e.graph(), e2.graph());

    e.m_steps(300);
    e2.m_steps(300);
    equal_graphs(e.graph(), e2.graph());
}

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = ErEnsembleM::<EmptyNode, Pcg64>::new(500, 3000, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=200 {
        let steps = e.m_steps(i);
        e.undo_steps_quiet(steps);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }

}
