use rand_pcg::Pcg64;
use rand::SeedableRng;
use net_ensembles::*;
mod common;
use common::{equal_graphs};

#[test]
fn step_test() {
    let rng = Pcg64::seed_from_u64(7567526);
    let rng2 = Pcg64::seed_from_u64(7566);
    let e = ErEnsembleC::<EmptyNode, Pcg64>::new(500, 2.7, rng);
    let degree_dist: Vec<usize> = e.graph().container_iter().map(|c| c.degree()).collect();
    let mut config = ConfigurationModel::<EmptyNode, Pcg64>::from_vec(degree_dist, rng2);
    let mut config_change = config.clone();
    config.sort_adj();

    for i in 0..=200 {
        let steps = config_change.m_steps(i);
        println!("{:?}", steps);
        config_change.undo_steps_quiet(steps);

        config_change.sort_adj();
        equal_graphs(&config.graph(), &config_change.graph());
    }

}