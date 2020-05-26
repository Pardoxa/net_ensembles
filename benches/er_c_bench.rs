use criterion::{criterion_group, criterion_main, Criterion};
use net_ensembles::{ErEnsembleC, EmptyNode, rand::SeedableRng};
use rand_pcg::Pcg64;
use core::time::Duration;
mod common;
use common::*;

const SIZE: usize = 50;
const C: f64 = 4.0;
const SEED: u64 = 123_239_010;
const M_STEPSIZE: usize = 100;

fn init() -> ErEnsembleC<EmptyNode, Pcg64> {
    let rng = Pcg64::seed_from_u64(SEED);
    ErEnsembleC::<EmptyNode, _>::new(
        SIZE,
        C,
        rng
    )
}

pub fn er_steps_bench(c: &mut Criterion) {
    generic_steps_bench(c, "er_c", M_STEPSIZE, init);
}

pub fn bench_m(c: &mut Criterion) {
    generic_measure_bench(c, "er_c", M_STEPSIZE, init);
}

pub fn bench_s_m(c: &mut Criterion) {
    generic_simple_measure_bench(c, "er_c", init);
}

pub fn bench_iterator(c: &mut Criterion) {
    generic_iter_bench(c, "er_c", init);
}

criterion_group!{
    name = s_measure_er_c;
    config = Criterion::default()
        .sample_size(200)
        .warm_up_time( Duration::new(1, 1))
        .measurement_time(Duration::new(30, 0));
    targets = bench_s_m
}

criterion_group!{
    name = measure_er_c;
    config = Criterion::default()
        .sample_size(200)
        .warm_up_time( Duration::new(1, 1))
        .measurement_time(Duration::new(30, 0));
    targets = bench_m
}


criterion_group!{
    name = er_c_steps;
    config = Criterion::default().sample_size(200);
    targets = er_steps_bench
}

criterion_group!{
    name = benches_iter;
    config = Criterion::default().sample_size(200);
    targets = bench_iterator
}

criterion_main!(s_measure_er_c, measure_er_c, benches_iter, er_c_steps);
