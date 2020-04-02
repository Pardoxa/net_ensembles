use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;


#[test]
fn show() {
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
