use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::equal_graphs;

#[test]
fn randomize() {
    let rng = Pcg64::seed_from_u64(8763545);
    let e = ErEnsembleM::<TestNode, Pcg64>::new(20, 70, rng);
    assert_eq!(e.graph().edge_count(), 70);
    assert_eq!(20, e.graph().vertex_count());
}
