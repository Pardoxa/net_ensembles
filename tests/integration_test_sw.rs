use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;


#[test]
fn show() {
    let size = 300;
    let rng = Pcg64::seed_from_u64(75675026);
    let mut e = SwEnsemble::<EmptyNode, Pcg64>::new(size, 0.15, rng);
    
    for _ in 0..200 {
        e.randomize();
        for i in 0..size {
            assert_eq!(2, e.graph().container(i as usize).count_root());
        }
    }
}
