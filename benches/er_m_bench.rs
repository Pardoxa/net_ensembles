use criterion::{criterion_group, criterion_main, Criterion};
use net_ensembles::{ErEnsembleM, EmptyNode, rand::SeedableRng};
use rand_pcg::Pcg64;
use core::time::Duration;
mod common;
use common::*;

const SIZE: u32 = 50;
const M: usize = 100;
const SEED: u64 = 123_239_010;
const M_STEPSIZE: usize = 100;

fn init() -> ErEnsembleM<EmptyNode, Pcg64> {
    let rng = Pcg64::seed_from_u64(SEED);
    ErEnsembleM::<EmptyNode, Pcg64>::new(SIZE, M, rng)
}

pub fn er_steps_bench(c: &mut Criterion) {
    generic_steps_bench(c, "er_m", M_STEPSIZE, init);
}

pub fn bench_m(c: &mut Criterion) {
    generic_measure_bench(c, "er_m", M_STEPSIZE, init);
}

pub fn bench_iterator(c: &mut Criterion) {
    generic_iter_bench(c, "er_m", init);
}



criterion_group!{
    name = measure_er_m;
    config = Criterion::default()
        .sample_size(200)
        .warm_up_time( Duration::new(1, 1))
        .measurement_time(Duration::new(30, 0));
    targets = bench_m
}


criterion_group!{
    name = er_m_steps;
    config = Criterion::default().sample_size(200);
    targets = er_steps_bench
}

criterion_group!{
    name = benches_iter;
    config = Criterion::default().sample_size(200);
    targets = bench_iterator
}

criterion_main!(measure_er_m, benches_iter, er_m_steps);
