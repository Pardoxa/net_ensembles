use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;

#[test]
fn randomize() {
    let rng = Pcg64::seed_from_u64(8763545);
    let e = ErEnsembleM::<EmptyNode, Pcg64>::new(20, 70, rng);
    assert_eq!(e.graph().edge_count(), 70);
    assert_eq!(20, e.graph().vertex_count());
}

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let mut e = ErEnsembleM::<EmptyNode, Pcg64>::new(500, 3000, rng);
    let mut e_0 = e.clone();
    e_0.sort_adj();

    for i in 0..=200 {
        let steps = e.mc_steps(i);
        e.undo_steps_quiet(steps);

        e.sort_adj();
        equal_graphs(&e_0.graph(), &e.graph());
    }

}
